use crate::{
    db::{
        sessions::{delete_session, new_session},
        validate_password,
    },
    types::{AppError, AppState, AuthBody, AuthPayload, Claims, LogoutResult},
};
use axum::{extract::State, routing::post, Json, Router};

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/authorize", post(authorize))
        .route("/logout", post(logout))
}

pub(crate) async fn authorize(
    State(st): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AppError> {
    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err(AppError::MissingCredentials);
    }
    let conn = st.conn().await?;
    let clid = payload.client_id.clone();
    let pwd_valid = validate_password(conn, clid, payload.client_secret).await?;
    if !pwd_valid {
        return Err(AppError::WrongCredentials);
    } else {
        let session = new_session(st.pool(), payload.client_id).await?;
        let claims = Claims::from_session(&session);
        let token = claims.to_token(&st.encoding())?;
        let ab = AuthBody::new(token);
        tracing::warn!("authorize ab: {:?}", ab);
        Ok(Json(ab))
    }
}

pub(crate) async fn logout(
    State(st): State<AppState>,
    claims: Claims,
) -> Result<Json<LogoutResult>, AppError> {
    let sess_id = claims.jti.clone();
    let conn = st.conn().await?;
    let _ = delete_session(conn, claims.jti).await?;
    Ok(axum::Json(LogoutResult::new(sess_id)))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        db::{
            sessions::{delete_user_sessions, get_session},
            users::{deconfirm_user, new_user_confirmed},
        },
        models::NewConfirmedUser,
        routes::_test_utils::test_app,
        types::test_pool_from_env,
    };
    use fake::{Fake, Faker};
    use http::StatusCode;
    use serde_json::json;

    #[test_log::test(tokio::test)]
    async fn test_authorize_ok() -> anyhow::Result<()> {
        let server = test_app(auth_router())?;
        let pool = test_pool_from_env();

        // create user
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        let pwd = user_data.password.clone();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let uid = user.id.clone();

        // post /authorize
        let login_data = AuthPayload {
            client_id: user_email,
            client_secret: pwd,
        };
        let resp = server.post(&"/authorize").json(&json!(login_data)).await;

        // cleanup
        let c = pool.get().await?;
        let _ = deconfirm_user(c, uid.clone()).await?;
        let c = pool.get().await?;
        let _ = delete_user_sessions(c, uid.clone()).await?;

        // assert
        resp.assert_status_ok();
        let resp_json = resp.json::<AuthBody>();
        assert!(resp_json.access_token.len() > 0);
        assert_eq!(resp_json.token_type, String::from("Bearer"));
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_authorize_bad_pwd() -> anyhow::Result<()> {
        let server = test_app(auth_router())?;
        let pool = test_pool_from_env();

        // create user
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let uid = user.id.clone();
        let bad_pass = "bad_password".to_string();

        // post /authorize
        let login_data = AuthPayload {
            client_id: user_email,
            client_secret: bad_pass,
        };
        let resp = server.post(&"/authorize").json(&json!(login_data)).await;

        // cleanup
        let c = pool.get().await?;
        let _ = deconfirm_user(c, uid.clone()).await?;
        let c = pool.get().await?;
        let _ = delete_user_sessions(c, uid.clone()).await?;

        // assert
        resp.assert_status(StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_authorize_user_doesnt_exist() -> anyhow::Result<()> {
        let server = test_app(auth_router())?;
        let pool = test_pool_from_env();

        // create user
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let pwd = user_data.password.clone();
        let c = pool.get().await?;
        let user = new_user_confirmed(c, user_data).await?;
        let uid = user.id.clone();
        let bad_email = "not-user@example.com".to_string();

        let login_data = AuthPayload {
            client_id: bad_email,
            client_secret: pwd,
        };

        let resp = server.post(&"/authorize").json(&json!(login_data)).await;

        let c = pool.get().await?;
        let _ = deconfirm_user(c, uid.clone()).await?;
        let c = pool.get().await?;
        let _ = delete_user_sessions(c, uid.clone()).await?;

        // assert
        // TODO(SHR): fix this
        resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_authorize_user_not_confirmed() -> anyhow::Result<()> {
        let server = test_app(auth_router())?;
        let pool = test_pool_from_env();

        // create user
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = false;
        let pwd = user_data.password.clone();
        let user_email = user_data.email.clone();
        let c = pool.get().await?;
        let _ = new_user_confirmed(c, user_data).await?;

        let login_data = AuthPayload {
            client_id: user_email,
            client_secret: pwd,
        };

        let resp = server.post(&"/authorize").json(&json!(login_data)).await;

        // assert
        // TODO(SHR): fix this
        resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_logout() -> anyhow::Result<()> {
        use crate::types::Claims;
        use http::header;

        let server = test_app(auth_router())?;
        let pool = test_pool_from_env();

        // create user
        let mut user_data = Faker.fake::<NewConfirmedUser>();
        user_data.confirmed = true;
        let c = pool.get().await?;
        let user = new_user_confirmed(c, user_data).await?;
        let user_email = user.email.clone();
        let uid = user.id.clone();

        let p = pool.clone();
        let session = new_session(p, user_email.clone()).await?;
        let sid = session.id.clone();
        let token = Claims::from_session(&session).test_to_token()?;

        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let c = pool.get().await?;
        let s = get_session(c, sid.clone()).await?;
        assert!(s.is_some());

        let resp = server
            .post("/logout")
            .add_header(header_name, header_value)
            .await;

        let c = pool.get().await?;
        let _d = deconfirm_user(c, uid).await?;

        let c = pool.get().await?;
        let s = get_session(c, sid.clone()).await?;
        assert!(s.is_none());
        resp.assert_status_ok();

        Ok(())
    }
}
