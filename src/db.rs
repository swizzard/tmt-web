pub(crate) mod sessions;
pub(crate) mod tabs;
pub(crate) mod users;
mod util;
use deadpool_diesel::postgres::Connection;
use diesel::{prelude::*, select};
use tracing::error;

use crate::{auth::check_user_pwd, types::AppError};

pub async fn validate_password(
    conn: Connection,
    email: String,
    password: String,
) -> Result<bool, AppError> {
    // TODO(SHR): `DeserializationError(UnexpectedNullError)` here seems to indicate no user &
    // should be handled as such
    let result = conn
        .interact(|conn| select(check_user_pwd(email, password)).load(conn))
        .await
        .map_err(|e| {
            error!("validate_password db error: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            error!("validate_password db error: {:?}", e);
            AppError::DBError
        })?;
    Ok(result[0])
}
