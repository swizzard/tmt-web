#[cfg(test)]
use crate::types::AppState;

use axum::Router;
use axum_test::{TestServer, TestServerConfig};

pub(crate) fn test_app(router_to_test: Router<AppState>) -> anyhow::Result<TestServer> {
    let st = AppState::from_env();
    let app = router_to_test.with_state(st);
    let cfg = TestServerConfig {
        ..TestServerConfig::default()
    };
    TestServer::new_with_config(app, cfg)
}
