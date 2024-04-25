use crate::{
    db::tags,
    models::{
        session::Session,
        tab::{AttachTagRequest, DetachTagRequest, TagDetachedResponse},
        tag::{MatchedTags, NewTag, Tag},
    },
    types::{AppError, AppState, MatchFragmentRequest, PaginatedResult, PaginationRequest},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};

pub fn tags_router() -> Router<AppState> {
    Router::new()
        .route("/tabs/:tab_id/tags", post(attach))
        .route("/tabs/:tab_id/tags/:tag_id", delete(detach))
        .route("/tags", post(create))
        .route("/tags/:tag_id", delete(delete_tag))
        .route("/users/:user_id/tags", get(user_tags))
        .route("/users/:user_id/tags/fuzzy", get(user_tags_fuzzy))
}

async fn create(
    State(st): State<AppState>,
    session: Session,
    Json(payload): Json<NewTag>,
) -> Result<impl IntoResponse, AppError> {
    if payload.user_id != session.user_id {
        Err(AppError::WrongCredentials)
    } else {
        let conn = st.conn().await?;
        Ok((
            StatusCode::CREATED,
            Json(tags::new_tag(conn, payload).await?),
        ))
    }
}

async fn attach(
    State(st): State<AppState>,
    session: Session,
    Path(path_tid): Path<String>,
    Json(payload): Json<AttachTagRequest>,
) -> Result<impl IntoResponse, AppError> {
    if path_tid != payload.tab_id {
        Err(AppError::BadRequest)
    } else if session.user_id != payload.user_id {
        Err(AppError::WrongCredentials)
    } else {
        let pool = st.pool();
        Ok((
            StatusCode::CREATED,
            Json(tags::attach_tag(pool, payload).await?),
        ))
    }
}

async fn detach(
    State(st): State<AppState>,
    session: Session,
    Path((tab_id, tag_id)): Path<(String, String)>,
) -> Result<Json<TagDetachedResponse>, AppError> {
    let pool = st.pool();
    let payload = DetachTagRequest {
        user_id: session.user_id.clone(),
        tab_id,
        tag_id,
    };
    Ok(Json(tags::detach_tag(pool, payload).await?))
}

async fn delete_tag(
    State(st): State<AppState>,
    session: Session,
    Path(tag_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let conn = st.conn().await?;
    tags::delete_tag(conn, session.user_id, tag_id.clone()).await?;
    Ok(StatusCode::OK)
}

async fn user_tags(
    State(st): State<AppState>,
    session: Session,
    Path(user_id): Path<String>,
    Query(pr): Query<PaginationRequest>,
) -> Result<Json<PaginatedResult<Tag>>, AppError> {
    if user_id != session.user_id {
        return Err(AppError::WrongCredentials);
    }
    let conn = st.conn().await?;
    Ok(Json(tags::get_user_tags(conn, session.user_id, pr).await?))
}

async fn user_tags_fuzzy(
    State(st): State<AppState>,
    session: Session,
    Path(user_id): Path<String>,
    Query(MatchFragmentRequest { fragment }): Query<MatchFragmentRequest>,
) -> Result<Json<MatchedTags>, AppError> {
    if user_id != session.user_id {
        return Err(AppError::WrongCredentials);
    }
    if fragment.len() < 3 || fragment.len() > 20 {
        return Err(AppError::BadRequest);
    }
    let conn = st.conn().await?;
    Ok(Json(MatchedTags::new(
        tags::get_user_tags_fuzzy(conn, session.user_id, fragment).await?,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{
            sessions, tabs,
            test_util::{
                bulk_create_tags, bulk_create_tags_from_strings, create_tags_reverse_alpha,
            },
            users,
        },
        models::{tab::NewTab, user::NewConfirmedUser},
        routes::_test_utils::test_app,
        types::{test_pool_from_env, Claims},
    };
    use fake::{Fake, Faker};
    use http::header;

    #[test_log::test(tokio::test)]
    async fn test_create_tag_ok() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
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
        let tag_str = String::from("tag");
        let tag_data = NewTag {
            user_id: user_id.clone(),
            tag: tag_str.clone(),
        };
        let resp = server
            .post("/tags")
            .json(&tag_data)
            .add_header(header_name, header_value)
            .await;
        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::CREATED);
        let tag = resp.json::<Tag>();
        assert_eq!(tag.user_id, user_id);
        assert_eq!(tag.tag, tag_str);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_create_tag_wrong_user_id() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
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
        let other_user_id = Faker.fake::<String>();
        let tag_str = String::from("tag");
        let tag_data = NewTag {
            user_id: other_user_id,
            tag: tag_str.clone(),
        };
        let resp = server
            .post("/tags")
            .json(&tag_data)
            .add_header(header_name, header_value)
            .await;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::FORBIDDEN);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_attach_ok() -> anyhow::Result<()> {
        use crate::models::tab::TagAttachedResponse;
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();

        let mut tab_data = Faker.fake::<NewTab>();
        tab_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let attach_data = AttachTagRequest {
            user_id: user_id.clone(),
            tab_id: tab.id.clone(),
            tag_id: tag.id.clone(),
        };
        let resp = server
            .post(&format!("/tabs/{}/tags", tab.id))
            .json(&attach_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let created = tags::get_tab_tag(c, tab.id.clone(), tag.id.clone()).await?;

        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::CREATED);
        let tar = resp.json::<TagAttachedResponse>();
        assert_eq!(&tar.user_id, &user_id);
        assert_eq!(&tar.tab_id, &tab.id);
        assert_eq!(&tar.tag_id, &tag.id);
        assert_eq!(&created.tab_id, &tab.id);
        assert_eq!(&created.tag_id, &tag.id);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_attach_wrong_tab_id() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();

        let mut tab_data = Faker.fake::<NewTab>();
        tab_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let other_tab_id = Faker.fake::<String>();

        let attach_data = AttachTagRequest {
            user_id: user_id.clone(),
            tab_id: other_tab_id,
            tag_id: tag.id.clone(),
        };
        let resp = server
            .post(&format!("/tabs/{}/tags", tab.id))
            .json(&attach_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let cr = tags::get_tab_tag(c, tab.id.clone(), tag.id.clone()).await;
        assert!(cr.is_err());

        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::BAD_REQUEST);

        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_attach_wrong_user_id() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();

        let mut tab_data = Faker.fake::<NewTab>();
        tab_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let other_user_id = Faker.fake::<String>();

        let attach_data = AttachTagRequest {
            user_id: other_user_id,
            tab_id: tab.id.clone(),
            tag_id: tag.id.clone(),
        };
        let resp = server
            .post(&format!("/tabs/{}/tags", tab.id))
            .json(&attach_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let cr = tags::get_tab_tag(c, tab.id.clone(), tag.id.clone()).await;
        assert!(cr.is_err());

        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::FORBIDDEN);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_attach_tab_doesnt_belong() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;

        let c = pool.get().await?;
        let mut other_user_data = Faker.fake::<NewConfirmedUser>();
        other_user_data.confirmed = true;
        let other_user = users::new_user_confirmed(c, other_user_data).await?;
        let other_user_id = other_user.id.clone();

        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();

        let mut tab_data = Faker.fake::<NewTab>();
        tab_data.user_id.clone_from(&other_user_id);
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let attach_data = AttachTagRequest {
            user_id: user_id.clone(),
            tab_id: tab.id.clone(),
            tag_id: tag.id.clone(),
        };
        let resp = server
            .post(&format!("/tabs/{}/tags", tab.id))
            .json(&attach_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let cr = tags::get_tab_tag(c, tab.id.clone(), tag.id.clone()).await;
        assert!(cr.is_err());

        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tabs::delete_user_tabs(c, other_user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, other_user_id.clone()).await?;

        resp.assert_status(StatusCode::FORBIDDEN);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_attach_tag_doesnt_belong() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;

        let c = pool.get().await?;
        let mut other_user_data = Faker.fake::<NewConfirmedUser>();
        other_user_data.confirmed = true;
        let other_user = users::new_user_confirmed(c, other_user_data).await?;
        let other_user_id = other_user.id.clone();

        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();

        let mut tab_data = Faker.fake::<NewTab>();
        tab_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&other_user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let attach_data = AttachTagRequest {
            user_id: user_id.clone(),
            tab_id: tab.id.clone(),
            tag_id: tag.id.clone(),
        };
        let resp = server
            .post(&format!("/tabs/{}/tags", tab.id))
            .json(&attach_data)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let cr = tags::get_tab_tag(c, tab.id.clone(), tag.id.clone()).await;
        assert!(cr.is_err());

        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tabs::delete_user_tabs(c, other_user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, other_user_id.clone()).await?;

        resp.assert_status(StatusCode::FORBIDDEN);

        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_detach_tag_ok() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();

        let mut tab_data = Faker.fake::<NewTab>();
        tab_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let tab_id = tab.id.clone();

        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;
        let tag_id = tag.id.clone();

        let c = pool.get().await?;
        tags::mk_tab_tag(c, tab_id.clone(), tag_id.clone()).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let resp = server
            .delete(&format!("/tabs/{}/tags/{}", tab_id, tag_id))
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let cr = tags::get_tab_tag(c, tab.id.clone(), tag.id.clone()).await;
        assert!(cr.is_err());

        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let tdr = resp.json::<TagDetachedResponse>();
        assert_eq!(tdr.user_id, user_id);
        assert_eq!(tdr.tab_id, tab_id);
        assert_eq!(tdr.tag_id, tag_id);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_detach_tag_not_attached() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();

        let mut tab_data = Faker.fake::<NewTab>();
        tab_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let tab_id = tab.id.clone();

        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;
        let tag_id = tag.id.clone();

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let resp = server
            .delete(&format!("/tabs/{}/tags/{}", tab_id, tag_id))
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tabs::delete_user_tabs(c, user_id.clone()).await?;

        resp.assert_status_ok();
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_detach_tab_doesnt_belong() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;

        let c = pool.get().await?;
        let mut other_user_data = Faker.fake::<NewConfirmedUser>();
        other_user_data.confirmed = true;
        let other_user = users::new_user_confirmed(c, other_user_data).await?;
        let other_user_id = other_user.id.clone();

        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();

        let mut tab_data = Faker.fake::<NewTab>();
        tab_data.user_id.clone_from(&other_user_id);
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let resp = server
            .delete(&format!("/tabs/{}/tags/{}", tab.id, tag.id))
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tabs::delete_user_tabs(c, other_user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, other_user_id.clone()).await?;

        resp.assert_status(StatusCode::FORBIDDEN);

        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_detach_tag_doesnt_belong() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;

        let c = pool.get().await?;
        let mut other_user_data = Faker.fake::<NewConfirmedUser>();
        other_user_data.confirmed = true;
        let other_user = users::new_user_confirmed(c, other_user_data).await?;
        let other_user_id = other_user.id.clone();

        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();

        let mut tab_data = Faker.fake::<NewTab>();
        tab_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tab = tabs::new_tab(c, tab_data).await?;
        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&other_user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let resp = server
            .delete(&format!("/tabs/{}/tags/{}", tab.id, tag.id))
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tabs::delete_user_tabs(c, other_user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, other_user_id.clone()).await?;

        resp.assert_status(StatusCode::FORBIDDEN);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_delete_tag_ok() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;

        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();
        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let resp = server
            .delete(&format!("/tags/{}", tag.id))
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let cr = tags::get_tag(c, tag.id.clone()).await;
        assert!(cr.is_err());

        resp.assert_status_ok();
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_delete_tag_doesnt_exist() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;

        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();
        let tag_id = Faker.fake::<String>();

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let c = pool.get().await?;
        let cr = tags::get_tag(c, tag_id.clone()).await;
        assert!(cr.is_err());

        let resp = server
            .delete(&format!("/tags/{}", tag_id))
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_delete_tag_doesnt_belong() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;

        let c = pool.get().await?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();

        let c = pool.get().await?;
        let mut other_user_data = Faker.fake::<NewConfirmedUser>();
        other_user_data.confirmed = true;
        let other_user = users::new_user_confirmed(c, other_user_data).await?;
        let other_user_id = other_user.id.clone();

        let mut tag_data = Faker.fake::<NewTag>();
        tag_data.user_id.clone_from(&other_user_id);
        let c = pool.get().await?;
        let tag = tags::new_tag(c, tag_data).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let resp = server
            .delete(&format!("/tags/{}", tag.id))
            .add_header(header_name, header_value)
            .await;

        // header not actually deleted
        let c = pool.get().await?;
        let cr = tags::get_tag(c, tag.id.clone()).await;
        assert!(cr.is_ok());

        let c = pool.get().await?;
        users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        users::deconfirm_user(c, other_user_id.clone()).await?;

        let c = pool.get().await?;
        tags::delete_tag(c, other_user_id.clone(), tag.id.clone()).await?;

        resp.assert_status_ok();
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_user_tags_ok() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        bulk_create_tags(c, user_id.clone(), 5).await?;

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
            .get(&format!("/users/{}/tags", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tags = resp.json::<PaginatedResult<Tag>>();
        assert!(!paginated_tags.has_more);
        let gotten_tags = paginated_tags.results;
        assert_eq!(gotten_tags.len(), 5);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_user_tags_pagination_more() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        let tags = create_tags_reverse_alpha(c, user_id.clone()).await?;

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
            .get(&format!("/users/{}/tags", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tags = resp.json::<PaginatedResult<Tag>>();
        println!("paginated_tags {:?}", paginated_tags);
        assert!(paginated_tags.has_more);
        let expected_tags = tags.into_iter().take(5).collect::<Vec<Tag>>();
        let gotten_tags = paginated_tags.results;
        assert_eq!(gotten_tags, expected_tags);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_user_tags_pagination_no_more() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        let tags = create_tags_reverse_alpha(c, user_id.clone()).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let pag_info = PaginationRequest {
            page: Some(6),
            page_size: Some(5),
        };
        let resp = server
            .get(&format!("/users/{}/tags", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tags = resp.json::<PaginatedResult<Tag>>();
        println!("paginated_tags {:?}", paginated_tags);
        assert!(!paginated_tags.has_more);
        let expected_tags = tags.into_iter().skip(25).take(5).collect::<Vec<Tag>>();
        let gotten_tags = paginated_tags.results;
        assert_eq!(gotten_tags, expected_tags);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_user_tags_pagination_past_end() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        create_tags_reverse_alpha(c, user_id.clone()).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let pag_info = PaginationRequest {
            page: Some(7),
            page_size: Some(5),
        };
        let resp = server
            .get(&format!("/users/{}/tags", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = tags::delete_user_tags(c, user_id.clone()).await?;
        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tags = resp.json::<PaginatedResult<Tag>>();
        assert!(!paginated_tags.has_more);
        let gotten_tags = paginated_tags.results;
        assert!(gotten_tags.is_empty());
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_user_tags_no_tags() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
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
            page: Some(1),
            page_size: Some(5),
        };
        let resp = server
            .get(&format!("/users/{}/tags", &user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let paginated_tags = resp.json::<PaginatedResult<Tag>>();
        assert!(!paginated_tags.has_more);
        let gotten_tags = paginated_tags.results;
        assert!(gotten_tags.is_empty());
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_user_tags_wrong_user_id() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        create_tags_reverse_alpha(c, user_id.clone()).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let pag_info = PaginationRequest {
            page: Some(1),
            page_size: Some(5),
        };
        let other_user_id = Faker.fake::<String>();
        let resp = server
            .get(&format!("/users/{}/tags", &other_user_id))
            .add_query_params(pag_info)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::FORBIDDEN);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_user_tags_fuzzy_ok() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();

        let c = pool.get().await?;
        let ts = vec![
            "a bug".into(),
            "a dog".into(),
            "dog".into(),
            "doggy".into(),
            "dont".into(),
        ];
        bulk_create_tags_from_strings(c, user_id.clone(), ts).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let frag = MatchFragmentRequest {
            fragment: "dog".to_string(),
        };
        let resp = server
            .get(&format!("/users/{}/tags/fuzzy", &user_id))
            .add_query_params(frag)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let gotten_tags = resp.json::<MatchedTags>();
        assert_eq!(gotten_tags.matches.len(), 3);

        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_user_tags_fuzzy_wrong_user_id() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();
        let c = pool.get().await?;
        create_tags_reverse_alpha(c, user_id.clone()).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let other_user_id = Faker.fake::<String>();
        let frag = MatchFragmentRequest {
            fragment: "a".to_string(),
        };
        let resp = server
            .get(&format!("/users/{}/tags/fuzzy", &other_user_id))
            .add_query_params(frag)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::FORBIDDEN);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_user_tags_fuzzy_frag_too_short() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();

        let c = pool.get().await?;
        let ts = vec![
            "a bug".into(),
            "a dog".into(),
            "dog".into(),
            "doggy".into(),
            "dont".into(),
        ];
        bulk_create_tags_from_strings(c, user_id.clone(), ts).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let frag = MatchFragmentRequest {
            fragment: "d".to_string(),
        };
        let resp = server
            .get(&format!("/users/{}/tags/fuzzy", &user_id))
            .add_query_params(frag)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::BAD_REQUEST);

        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_user_tags_fuzzy_frag_too_long() -> anyhow::Result<()> {
        let pool = test_pool_from_env();
        let server = test_app(tags_router())?;
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = users::new_user_confirmed(c, user_data).await?;
        let user_id = user.id.clone();
        let user_email = user.email.clone();

        let c = pool.get().await?;
        let ts = vec![
            "a bug".into(),
            "a dog".into(),
            "dog".into(),
            "doggy".into(),
            "dont".into(),
        ];
        bulk_create_tags_from_strings(c, user_id.clone(), ts).await?;

        let session = sessions::new_session(pool.clone(), user_email).await?;
        let token = Claims::from_session(&session).test_to_token()?;
        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;
        let frag = MatchFragmentRequest {
            fragment: "doyoubelievehowlongthisisthough".to_string(),
        };
        let resp = server
            .get(&format!("/users/{}/tags/fuzzy", &user_id))
            .add_query_params(frag)
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _ = users::deconfirm_user(c, user_id.clone()).await?;
        let c = pool.get().await?;
        tags::delete_user_tags(c, user_id.clone()).await?;

        resp.assert_status(StatusCode::BAD_REQUEST);

        Ok(())
    }
}
