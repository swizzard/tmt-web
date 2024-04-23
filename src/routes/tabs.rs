use crate::{
    db::tabs,
    models::{NewTab, Session, Tab},
    types::{AppError, AppState, PaginatedResult, PaginationRequest},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

pub fn tabs_router() -> Router<AppState> {
    Router::new()
        .route("/tabs", post(create))
        .route("/tabs/:tab_id", get(get_tab))
        .route("/users/:user_id/tabs", get(user_tabs))
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

async fn get_tab(
    State(st): State<AppState>,
    session: Session,
    Path(tab_id): Path<String>,
) -> Result<Json<Tab>, AppError> {
    let conn = st.conn().await?;
    let tab = tabs::get_tab(conn, session.user_id.clone(), tab_id).await?;
    Ok(Json(tab))
}

async fn user_tabs(
    State(st): State<AppState>,
    session: Session,
    Path(user_id): Path<String>,
    Query(PaginationRequest { page, page_size }): Query<PaginationRequest>,
) -> Result<Json<PaginatedResult<Tab>>, AppError> {
    if user_id != session.user_id {
        return Err(AppError::WrongCredentials);
    }
    let pool = st.pool();
    let tabs = tabs::get_user_tabs(pool, session.user_id.clone(), page, page_size).await?;
    Ok(Json(tabs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{sessions, test_util::bulk_create_tabs, users},
        models::NewConfirmedUser,
        routes::_test_utils::test_app,
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
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

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
            .get(&format!("/users/{}/tabs", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<Tab>>();
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
            .get(&format!("/users/{}/tabs", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<Tab>>();
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
            .get(&format!("/users/{}/tabs", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<Tab>>();
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
            .get(&format!("/users/{}/tabs", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<Tab>>();
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
            .get(&format!("/users/{}/tabs", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<Tab>>();
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
            .get(&format!("/users/{}/tabs", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tabs = resp.json::<PaginatedResult<Tab>>();
        let expected_tabs = tabs.into_iter().take(25).collect::<Vec<_>>();
        assert!(paginated_tabs.has_more);
        assert_eq!(paginated_tabs.results, expected_tabs);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_get_user_tabs_wrong_user() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tabs_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let c = pool.get().await?;
        let _tabs = bulk_create_tabs(c, user_id.clone(), 5).await?;

        let mut login_user_data = Faker.fake::<NewConfirmedUser>();
        login_user_data.confirmed = true;
        let c = pool.get().await?;
        let login_user = users::new_user_confirmed(c, login_user_data).await?;
        let login_user_email = login_user.email.clone();

        let session = sessions::new_session(pool.clone(), login_user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let pag_info = PaginationRequest {
            page: Some(1),
            page_size: Some(5),
        };

        let resp = server
            .get(&format!("/users/{}/tabs", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::FORBIDDEN);
        Ok(())
    }
}
