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
    use crate::AppState;

    use serde_json::json;

    use axum_test::{TestServer, TestServerConfig};

    fn test_app() -> anyhow::Result<TestServer> {
        let st = AppState::from_env();
        let app = misc_router().with_state(st);
        let cfg = TestServerConfig {
            expect_success_by_default: true,
            ..TestServerConfig::default()
        };
        TestServer::new_with_config(app, cfg)
    }

    #[tokio::test]
    async fn test_hello_world() -> anyhow::Result<()> {
        let server = test_app()?;
        let resp = server.get(&"/").await;
        assert_eq!(resp.status_code(), 200);
        Ok(())
    }

    // #[tokio::test]
    // async fn test_private() -> anyhow::Result<()> {
    //     let server = test_app()?;
    // }
}
