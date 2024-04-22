use crate::{
    db::tabs,
    models::{NewTab, Session, Tab},
    types::{AppError, AppState},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

pub fn tabs_router() -> Router<AppState> {
    Router::new()
        .route("/tabs", post(create))
        .route("/tabs/:tab_id", get(get_tab))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{sessions, users},
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
            .post(&"/tabs")
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
            .post(&"/tabs")
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
}
