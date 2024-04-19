use crate::{
    models::Session,
    types::{AppError, AppState},
};
use axum::{routing::get, Router};

pub fn misc_router() -> Router<AppState> {
    Router::new()
        .route("/", get(hello_world))
        .route("/private", get(private))
}
async fn hello_world() -> String {
    tracing::debug!("Hello world");
    String::from("hello")
}

async fn private(session: Session) -> Result<String, AppError> {
    tracing::debug!("private");
    Ok(format!("Hello {:?}", session))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::routes::_test_utils::test_app;
    use fake::{Fake, Faker};

    #[test_log::test(tokio::test)]
    async fn test_hello_world() -> anyhow::Result<()> {
        let server = test_app(misc_router())?;
        let resp = server.get(&"/").await;
        assert_eq!(resp.status_code(), 200);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_private_not_logged_in() -> anyhow::Result<()> {
        use crate::routes::{auth::auth_router, users::users_router};

        let server = test_app(misc_router().merge(auth_router()).merge(users_router()))?;
        let resp = server.get(&"/private").await;
        assert_eq!(resp.status_code(), 400);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_private_logged_in() -> anyhow::Result<()> {
        use crate::{
            db::{
                sessions::{delete_session, new_session},
                users::{deconfirm_user, new_user_confirmed},
            },
            models::{NewConfirmedUser, NewUser},
            types::{test_pool_from_env, Claims},
        };
        use http::header;

        let server = test_app(misc_router())?;

        let pool = test_pool_from_env();

        let c = pool.get().await?;
        let create_user_data = Faker.fake::<NewUser>();
        let create_user_data = NewConfirmedUser {
            email: create_user_data.email,
            password: create_user_data.password,
            confirmed: true,
        };
        let user = new_user_confirmed(c, create_user_data).await?;

        let sp = pool.clone();
        let session = new_session(sp, user.email.clone()).await?;
        let token = Claims::from_session(&session).test_to_token()?;

        let bearer = format!("Bearer {}", token);
        let header_value = header::HeaderValue::from_str(&bearer)?;
        let header_name = header::AUTHORIZATION;

        let private_resp = server
            .get(&"/private")
            .add_header(header_name, header_value)
            .await;

        // cleanup
        let c = pool.get().await?;
        let _deconfirmed = deconfirm_user(c, user.id).await?;
        let c = pool.get().await?;
        let _ = delete_session(c, session.id).await?;

        private_resp.assert_status_ok();
        Ok(())
    }
}
