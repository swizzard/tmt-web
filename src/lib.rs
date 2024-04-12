mod auth;
mod db;
mod models;
mod routes;
mod schema;
mod types;
use axum::{
    routing::{get, post},
    Router,
};
use routes::*;
use types::AppState;

pub fn make_app() -> Router {
    let st = AppState::from_env();
    Router::new()
        .route("/", get(hello_world))
        .route("/authorize", post(authorize))
        .route("/private", get(private))
        .route("/logout", post(logout))
        .with_state(st)
}
