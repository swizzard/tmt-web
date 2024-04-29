mod auth;
mod db;
mod models;
mod routes;
mod schema;
mod types;
use axum::Router;
use routes::{
    auth::auth_router, misc::misc_router, tabs::tabs_router, tags::tags_router, users::users_router,
};
pub use types::AppState;

pub fn make_app(state: AppState) -> Router {
    Router::new()
        .merge(auth_router())
        .merge(misc_router())
        .merge(tabs_router())
        .merge(tags_router())
        .merge(users_router())
        .with_state(state)
}
