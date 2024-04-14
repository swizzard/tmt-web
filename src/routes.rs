pub mod auth;
pub mod tabs;

use crate::{models::Session, types::AppError};

pub(crate) use auth::{authorize, logout};

pub(crate) async fn hello_world() -> String {
    tracing::debug!("Hello world");
    String::from("hello")
}

pub(crate) async fn private(session: Session) -> Result<String, AppError> {
    tracing::debug!("private");
    Ok(format!("Hello {:?}", session))
}
