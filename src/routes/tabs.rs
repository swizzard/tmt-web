use crate::{
    db::{tabs, tags},
    models::{
        session::Session,
        tab::{
            NewTab, NewTabTag, NewTabWithTags, Tab, TabWithTags, UpdateTabWithTags, UserListTab,
        },
        tag::{NewTag, Tag},
    },
    types::{AppError, AppState, PaginatedResult, PaginationRequest},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};

pub fn tabs_router() -> Router<AppState> {
    Router::new()
        .route("/tabs", post(create))
        .route("/tabs/with-tags", post(create_with_tags))
        .route("/tabs/:tab_id", delete(delete_tab))
        .route("/tabs/:tab_id", get(get_tab))
        .route("/tabs/:tab_id", put(edit_tab))
        .route("/tabs/:tab_id/with-tags", get(get_tab_with_tags))
        .route("/users/tabs", get(user_tabs))
}

async fn create(
    State(st): State<AppState>,
    session: Session,
    Json(payload): Json<NewTab>,
) -> Result<impl IntoResponse, AppError> {
    if payload.user_id != session.user_id {
        Err(AppError::WrongCredentials)
    } else {
        let conn = st.conn().await?;
        let tab = tabs::new_tab(conn, payload).await?;
        Ok((StatusCode::CREATED, Json(tab)))
    }
}

async fn create_with_tags(
    State(st): State<AppState>,
    session: Session,
    Json(payload): Json<NewTabWithTags>,
) -> Result<impl IntoResponse, AppError> {
    let tab_with_user_id = NewTab {
        user_id: session.user_id.clone(),
        url: payload.tab.url,
        notes: payload.tab.notes,
    };
    let conn = st.conn().await?;
    let tab = tabs::new_tab(conn, tab_with_user_id).await?;
    let tab_id = tab.id.clone();
    let tags = payload.tags;
    let mut new: Vec<NewTag> = Vec::with_capacity(tags.len());
    let mut to_insert: Vec<NewTabTag> = Vec::with_capacity(tags.len());
    let mut tags_to_return: Vec<Tag> = Vec::with_capacity(tags.len());
    for tag in tags {
        if let Some(gid) = tag.id {
            to_insert.push(NewTabTag {
                tab_id: tab_id.clone(),
                tag_id: gid.clone(),
            });
            tags_to_return.push(Tag {
                id: gid,
                user_id: session.user_id.clone(),
                tag: tag.tag,
            })
        } else {
            new.push(NewTag {
                user_id: tag.user_id,
                tag: tag.tag,
            });
        }
    }
    let conn = st.conn().await?;
    let new_tags = tags::bulk_insert_tags(conn, new).await?;
    for tag in new_tags {
        to_insert.push(NewTabTag {
            tab_id: tab_id.clone(),
            tag_id: tag.id.clone(),
        });
        tags_to_return.push(tag);
    }
    let conn = st.conn().await?;
    tags::bulk_mk_tab_tags(conn, to_insert).await?;
    let tab_with_tags = TabWithTags {
        tab,
        tags: tags_to_return,
    };
    Ok((StatusCode::CREATED, Json(tab_with_tags)))
}

async fn get_tab(
    State(st): State<AppState>,
    session: Session,
    Path(tab_id): Path<String>,
) -> Result<Json<Tab>, AppError> {
    let conn = st.conn().await?;
    let tab = tabs::get_tab(conn, session.user_id.clone(), tab_id).await?;
    Ok(Json(tab))
}

async fn get_tab_with_tags(
    State(st): State<AppState>,
    session: Session,
    Path(tab_id): Path<String>,
) -> Result<Json<TabWithTags>, AppError> {
    let p = st.pool();
    let tab = tabs::get_tab_with_tags(p, session.user_id.clone(), tab_id).await?;
    Ok(Json(tab))
}

async fn user_tabs(
    State(st): State<AppState>,
    session: Session,
    Query(pr): Query<PaginationRequest>,
) -> Result<Json<PaginatedResult<UserListTab>>, AppError> {
    tracing::info!("getting tabs for user {} {:?}", session.user_id, pr);
    let pool = st.pool();
    let tabs = tabs::get_user_tabs(pool, session.user_id.clone(), pr).await?;
    Ok(Json(tabs))
}

async fn edit_tab(
    State(st): State<AppState>,
    session: Session,
    Path(tab_id): Path<String>,
    Json(payload): Json<UpdateTabWithTags>,
) -> Result<Json<TabWithTags>, AppError> {
    let conn = st.conn().await?;
    let updated = tabs::update_tab_and_tags(conn, tab_id, session.user_id.clone(), payload).await?;
    Ok(Json(updated))
}

async fn delete_tab(
    State(st): State<AppState>,
    session: Session,
    Path(tab_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let conn = st.conn().await?;
    tabs::delete_tab(conn, session.user_id.clone(), tab_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{
        db::{
            sessions,
            tags::bulk_mk_tab_tags,
            test_util::{bulk_create_tabs, bulk_create_tags, bulk_create_tags_from_strings},
            users,
        },
        models::{
            tab::{RequestedNewTab, UpdateTab, UpdateTags},
            tag::MaybeNewTag,
            user::NewConfirmedUser,
        },
        routes::{
            _test_utils::test_app,
            tabs::tests::{tabs::get_tab_tags, tags::get_tags_by_ids},
        },
        types::{test_pool_from_env, Claims},
    };
    use fake::{Fake, Faker};
    use http::header;

    #[test_log::test(tokio::test)]
    async fn test_create_tab_ok() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let c = pool.get().await?;

        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();
        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let url = String::from("https://example.com");
        let notes: Option<String> = Some("notes".into());
        let tab_data = NewTab {
            user_id: user_id.clone(),
            url: url.clone(),
            notes: notes.clone(),
        };
        let resp = server
            .post("/tabs")
            .json(&tab_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::CREATED);
        let tab = resp.json::<Tab>();
        assert_eq!(tab.user_id, user_id);
        assert_eq!(tab.url, url);
        assert_eq!(tab.notes, notes);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_create_tab_wrong_user_id() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let c = pool.get().await?;

        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let url = String::from("https://example.com");
        let notes: Option<String> = Some("notes".into());
        let other_user_id = Faker.fake::<String>();
        let tab_data = NewTab {
            user_id: other_user_id,
            url: url.clone(),
            notes: notes.clone(),
        };
        let resp = server
            .post("/tabs")
            .json(&tab_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::FORBIDDEN);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_get_tab_exists() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let c = pool.get().await?;

        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();
        let url = String::from("https://example.com");
        let notes: Option<String> = Some("notes".into());
        let tab_data = NewTab {
            user_id: user_id.clone(),
            url: url.clone(),
            notes: notes.clone(),
        };
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let tab_id = tab.id.clone();
        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let resp = server
            .get(&format!("/tabs/{}", tab_id))
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let gotten_tab = resp.json::<Tab>();
        assert_eq!(gotten_tab.id, tab_id.clone());
        assert_eq!(gotten_tab.user_id, user_id);
        assert_eq!(gotten_tab.url, url);
        assert_eq!(gotten_tab.notes, notes);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_get_tab_doesnt_exist() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let c = pool.get().await?;

        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();
        let url = String::from("https://example.com");
        let notes: Option<String> = Some("notes".into());
        let tab_data = NewTab {
            user_id: user_id.clone(),
            url: url.clone(),
            notes: notes.clone(),
        };
        let c = pool.get().await?;
        let _tab = tabs::new_tab(c, tab_data).await?;
        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let other_tab_id = Faker.fake::<String>();
        let resp = server
            .get(&format!("/tabs/{}", other_tab_id))
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::NOT_FOUND);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_get_tab_wrong_user_id() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let c = pool.get().await?;

        let mut u1_data = Faker.fake::<NewConfirmedUser>();
        u1_data.confirmed = true;
        let u1 = users::new_user_confirmed(c, u1_data).await?;
        let u1_id = u1.id.clone();
        let mut u2_data = Faker.fake::<NewConfirmedUser>();
        u2_data.confirmed = true;
        let c = pool.get().await?;
        let u2 = users::new_user_confirmed(c, u2_data).await?;
        let u2_id = u2.id.clone();
        let u2_email = u2.email.clone();

        let url = String::from("https://example.com");
        let notes: Option<String> = Some("notes".into());
        let tab_data = NewTab {
            user_id: u1_id.clone(),
            url: url.clone(),
            notes: notes.clone(),
        };
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;

        // "log in" as other user
        let session = sessions::new_session(pool.clone(), u2_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let resp = server
            .get(&format!("/tabs/{}", tab.id))
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, u1_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, u1_id.clone()).await?;
        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, u2_id.clone()).await?;

        resp.assert_status(StatusCode::NOT_FOUND);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_get_user_tabs_ok() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        let tabs = bulk_create_tabs(c, user_id.clone(), 5).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let pag_info = PaginationRequest {
            page: Some(1),
            page_size: Some(5),
        };
        let resp = server
            .get("/users/tabs")
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<UserListTab>>();
        assert!(!paginated_tabs.has_more);
        let gotten_tabs = paginated_tabs.results;
        assert_eq!(gotten_tabs, tabs);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_get_user_tabs_pagination_more() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        let tabs = bulk_create_tabs(c, user_id.clone(), 47).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let pag_info = PaginationRequest {
            page: Some(8),
            page_size: Some(5),
        };
        let resp = server
            .get("/users/tabs")
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<UserListTab>>();
        let offset_tabs = tabs.into_iter().skip(35).take(5).collect::<Vec<_>>();
        assert!(paginated_tabs.has_more);
        let gotten_tabs = paginated_tabs.results;
        assert_eq!(gotten_tabs, offset_tabs);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_get_user_tabs_pagination_no_more() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        let tabs = bulk_create_tabs(c, user_id.clone(), 40).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let pag_info = PaginationRequest {
            page: Some(8),
            page_size: Some(5),
        };
        let resp = server
            .get("/users/tabs")
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<UserListTab>>();
        let offset_tabs = tabs.into_iter().skip(35).take(5).collect::<Vec<_>>();
        assert!(!paginated_tabs.has_more);
        let gotten_tabs = paginated_tabs.results;
        assert_eq!(gotten_tabs, offset_tabs);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_get_user_tabs_past_end() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        let _tabs = bulk_create_tabs(c, user_id.clone(), 5).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let pag_info = PaginationRequest {
            page: Some(2),
            page_size: Some(5),
        };
        let resp = server
            .get("/users/tabs")
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<UserListTab>>();
        assert!(!paginated_tabs.has_more);
        assert!(paginated_tabs.results.is_empty());
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_get_user_tabs_no_tabs() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let pag_info = PaginationRequest {
            page: Some(2),
            page_size: Some(5),
        };
        let resp = server
            .get("/users/tabs")
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<UserListTab>>();
        assert!(!paginated_tabs.has_more);
        assert!(paginated_tabs.results.is_empty());
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_get_user_tabs_defaults() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        let tabs = bulk_create_tabs(c, user_id.clone(), 50).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let pag_info = PaginationRequest {
            page: None,
            page_size: None,
        };
        let resp = server
            .get("/users/tabs")
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<UserListTab>>();
        let expected_tabs = tabs.into_iter().take(25).collect::<Vec<_>>();
        assert!(paginated_tabs.has_more);
        assert_eq!(paginated_tabs.results, expected_tabs);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_get_tab_with_tags_ok() -> anyhow::Result<()> {
        use crate::models::tab::NewTabTag;
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();

        let c = pool.get().await?;
        let tabs = bulk_create_tabs(c, user_id.clone(), 2).await?;
        let tab = tabs.first().unwrap().clone();
        let tab_id = tab.id.clone();

        let c = pool.get().await?;
        let tags = bulk_create_tags(c, user_id.clone(), 5).await?;
        let to_attach_tags = tags.iter().take(3);
        let ntt: Vec<NewTabTag> = to_attach_tags
            .clone()
            .map(|Tag { id, .. }| NewTabTag {
                tab_id: tab_id.clone(),
                tag_id: id.clone(),
            })
            .collect();

        let c = pool.get().await?;
        bulk_mk_tab_tags(c, ntt).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let resp = server
            .get(&format!("/tabs/{}/with-tags", &tab_id))
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let tab_with_tags = resp.json::<TabWithTags>();
        assert_eq!(tab_with_tags.tab, tab);
        assert_eq!(
            tab_with_tags.tags,
            to_attach_tags.cloned().collect::<Vec<_>>()
        );

        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_create_with_tags_ok() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();

        let c = pool.get().await?;
        let tags = bulk_create_tags(c, user_id.clone(), 5).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let mut tags_data: Vec<MaybeNewTag> = tags
            .into_iter()
            .take(3)
            .map(|Tag { id, tag, .. }| MaybeNewTag {
                id: Some(id.clone()),
                user_id: user_id.clone(),
                tag: tag.clone(),
            })
            .collect();
        tags_data.push(MaybeNewTag {
            id: None,
            user_id: user_id.clone(),
            tag: Faker.fake::<String>(),
        });
        tags_data.push(MaybeNewTag {
            id: None,
            user_id: user_id.clone(),
            tag: Faker.fake::<String>(),
        });

        let url = String::from("https://example.com");
        let notes: Option<String> = Some("notes".into());
        let tab_data = NewTabWithTags {
            tab: RequestedNewTab {
                url: url.clone(),
                notes: notes.clone(),
            },
            tags: tags_data,
        };

        let resp = server
            .post("/tabs/with-tags")
            .json(&tab_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::CREATED);
        let tab_with_tags = resp.json::<TabWithTags>();
        assert_eq!(tab_with_tags.tab.user_id, user_id);
        assert_eq!(tab_with_tags.tab.url, url);
        assert_eq!(tab_with_tags.tab.notes, notes);
        assert_eq!(tab_with_tags.tags.len(), 5);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_edit_tab_update_tab_no_tags_ok() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;

        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let new_url = String::from("http://example.com/new");
        let new_notes = Some(String::from("new notes"));

        let old_tab_data = NewTab {
            user_id: user_id.clone(),
            url: String::from("http://example.com/old"),
            notes: None,
        };
        let c = pool.get().await?;
        let old_tab = tabs::new_tab(c, old_tab_data).await?;
        let tab_id = old_tab.id.clone();

        let update_data = UpdateTabWithTags {
            tab: UpdateTab {
                url: new_url.clone(),
                notes: new_notes.clone(),
            },
            tags: vec![],
        };
        let session = sessions::new_session(pool.clone(), user.email.clone()).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let resp = server
            .put(&format!("/tabs/{}", tab_id))
            .json(&update_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let updated_tab = resp.json::<TabWithTags>();
        assert_eq!(updated_tab.tab.url, new_url);
        assert_eq!(updated_tab.tab.notes, new_notes);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_edit_tab_update_tab_same_tags() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;

        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let old_url = String::from("http://example.com/old");
        let old_notes = Some(String::from("old notes"));

        let old_tab_data = NewTab {
            user_id: user_id.clone(),
            url: old_url.clone(),
            notes: old_notes.clone(),
        };
        let c = pool.get().await?;
        let old_tab = tabs::new_tab(c, old_tab_data).await?;
        let tab_id = old_tab.id.clone();
        let c = pool.get().await?;
        let tags =
            bulk_create_tags_from_strings(c, user_id.clone(), vec!["tag1".into(), "tag2".into()])
                .await?;
        let mut update_tags: UpdateTags = Vec::new();
        let mut tts_to_insert: Vec<NewTabTag> = Vec::new();
        for tag in tags.iter() {
            update_tags.push(MaybeNewTag {
                id: Some(tag.id.clone()),
                user_id: user_id.clone(),
                tag: tag.tag.clone(),
            });
            tts_to_insert.push(NewTabTag {
                tab_id: tab_id.clone(),
                tag_id: tag.id.clone(),
            });
        }
        let c = pool.get().await?;
        bulk_mk_tab_tags(c, tts_to_insert).await?;

        let update_data = UpdateTabWithTags {
            tab: UpdateTab {
                url: old_url.clone(),
                notes: old_notes.clone(),
            },
            tags: update_tags,
        };
        let session = sessions::new_session(pool.clone(), user.email.clone()).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let resp = server
            .put(&format!("/tabs/{}", tab_id))
            .json(&update_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let updated_tab = resp.json::<TabWithTags>();
        assert_eq!(updated_tab.tab.url, old_url);
        assert_eq!(updated_tab.tab.notes, old_notes);
        let updated_tags: HashSet<&Tag> = HashSet::from_iter(updated_tab.tags.iter());
        let old_tags: HashSet<&Tag> = HashSet::from_iter(tags.iter());
        assert_eq!(updated_tags, old_tags);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_edit_tab_update_tab_delete_tags() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;

        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let old_url = String::from("http://example.com/old");
        let old_notes = Some(String::from("old notes"));

        let old_tab_data = NewTab {
            user_id: user_id.clone(),
            url: old_url.clone(),
            notes: old_notes.clone(),
        };
        let c = pool.get().await?;
        let old_tab = tabs::new_tab(c, old_tab_data).await?;
        let tab_id = old_tab.id.clone();
        let c = pool.get().await?;
        let tags =
            bulk_create_tags_from_strings(c, user_id.clone(), vec!["tag1".into(), "tag2".into()])
                .await?;
        let mut tag_ids: Vec<String> = Vec::with_capacity(tags.len());
        let mut tts_to_insert: Vec<NewTabTag> = Vec::with_capacity(tags.len());
        for tag in tags.iter() {
            tag_ids.push(tag.id.clone());
            tts_to_insert.push(NewTabTag {
                tab_id: tab_id.clone(),
                tag_id: tag.id.clone(),
            });
        }
        let c = pool.get().await?;
        bulk_mk_tab_tags(c, tts_to_insert).await?;

        let update_data = UpdateTabWithTags {
            tab: UpdateTab {
                url: old_url.clone(),
                notes: old_notes.clone(),
            },
            tags: vec![],
        };
        let session = sessions::new_session(pool.clone(), user.email.clone()).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let resp = server
            .put(&format!("/tabs/{}", tab_id))
            .json(&update_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let updated_tab = resp.json::<TabWithTags>();
        assert_eq!(updated_tab.tab.url, old_url);
        assert_eq!(updated_tab.tab.notes, old_notes);
        assert!(updated_tab.tags.is_empty());
        let c = pool.get().await?;
        let existing = get_tags_by_ids(c, tag_ids).await?;
        assert_eq!(tags, existing);
        Ok(())
    }
    #[ignore]
    #[test_log::test(tokio::test)]
    async fn test_edit_tab_update_tab_create_and_attach_tags() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;

        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let old_url = String::from("http://example.com/old");
        let old_notes = Some(String::from("old notes"));

        let old_tab_data = NewTab {
            user_id: user_id.clone(),
            url: old_url.clone(),
            notes: old_notes.clone(),
        };
        let c = pool.get().await?;
        let old_tab = tabs::new_tab(c, old_tab_data).await?;
        let tab_id = old_tab.id.clone();
        let c = pool.get().await?;
        let tags =
            bulk_create_tags_from_strings(c, user_id.clone(), vec!["tag1".into(), "tag2".into()])
                .await?;
        let mut to_attach = tags
            .iter()
            .map(|tag| MaybeNewTag {
                id: Some(tag.id.clone()),
                user_id: user_id.clone(),
                tag: tag.tag.clone(),
            })
            .collect::<Vec<_>>();
        to_attach.push(MaybeNewTag {
            id: None,
            user_id: user_id.clone(),
            tag: String::from("new tag"),
        });

        let update_data = UpdateTabWithTags {
            tab: UpdateTab {
                url: old_url.clone(),
                notes: old_notes.clone(),
            },
            tags: to_attach,
        };
        let session = sessions::new_session(pool.clone(), user.email.clone()).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let resp = server
            .put(&format!("/tabs/{}", tab_id))
            .json(&update_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let updated_tab = resp.json::<TabWithTags>();
        assert_eq!(updated_tab.tab.url, old_url);
        assert_eq!(updated_tab.tab.notes, old_notes);

        let c = pool.get().await?;
        let actual_tags = get_tab_tags(c, user_id.clone(), tab_id.clone()).await?;
        assert_eq!(3, actual_tags.len());
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_delete_tab_ok() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let c = pool.get().await?;

        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();
        let url = String::from("https://example.com");
        let notes: Option<String> = Some("notes".into());
        let tab_data = NewTab {
            user_id: user_id.clone(),
            url: url.clone(),
            notes: notes.clone(),
        };
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let tab_id = tab.id.clone();
        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let resp = server
            .delete(&format!("/tabs/{}", tab_id))
            .add_header(header_name, header_value)
            .await;
        resp.assert_status(StatusCode::NO_CONTENT);
        let c = pool.get().await?;
        let get_tab_err = tabs::get_tab(c, user_id.clone(), tab_id.clone())
            .await
            .expect_err("tab not deleted");
        assert!(matches!(get_tab_err, AppError::NotFound));

        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_delete_tab_wrong_user() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;

        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();

        let c = pool.get().await?;
        let mut other_user_data = Faker.fake::<NewConfirmedUser>();
        other_user_data.confirmed = true;
        let other_user = users::new_user_confirmed(c, other_user_data).await?;
        let other_user_email = other_user.email.clone();
        let other_user_id = other_user.id.clone();

        let url = String::from("https://example.com");
        let notes: Option<String> = Some("notes".into());
        let tab_data = NewTab {
            user_id: user_id.clone(),
            url: url.clone(),
            notes: notes.clone(),
        };
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let tab_id = tab.id.clone();

        let session = sessions::new_session(pool.clone(), other_user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let resp = server
            .delete(&format!("/tabs/{}", tab_id))
            .add_header(header_name, header_value)
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);
        let c = pool.get().await?;
        tabs::get_tab(c, user_id.clone(), tab_id.clone()).await?;

        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tabs::delete_user_tabs(c, other_user_id.clone()).await?;
        Ok(())
    }
}
