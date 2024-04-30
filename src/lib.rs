mod auth;
mod db;
mod models;
mod routes;
mod schema;
mod types;

use axum::http::Method;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use routes::{
    auth::auth_router, misc::misc_router, tabs::tabs_router, tags::tags_router, users::users_router,
};
pub use types::AppState;

pub fn make_app(state: AppState) -> Router {
    let cors = CorsLayer::very_permissive();
    // .allow_methods([Method::GET, Method::POST, Method::DELETE])
    // .allow_origin(Any);
    Router::new()
        .merge(auth_router())
        .merge(misc_router())
        .merge(tabs_router())
        .merge(tags_router())
        .merge(users_router())
        .with_state(state)
        .layer(cors)
}
