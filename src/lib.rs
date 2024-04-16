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
use std::sync::Arc;
pub use types::AppSt;

pub fn make_app(state: Box<dyn AppSt>) -> Router {
    let st = Arc::new(state);
    Router::new()
        .route("/", get(hello_world))
        .route("/authorize", post(authorize))
        .route("/private", get(private))
        .route("/logout", post(logout))
        .merge(tabs::tabs_router())
        .merge(users::users_router())
        .with_state(st)
}
