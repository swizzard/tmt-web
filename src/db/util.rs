use deadpool_diesel::postgres::{Connection, Pool};
use diesel::result::{Error as DE, UnexpectedNullError};

use crate::types::AppError;

pub(crate) async fn get_conn(pool: Pool) -> Result<Connection, AppError> {
    pool.get().await.map_err(|e| {
        tracing::error!("db connection error {:?}", e);
        AppError::InternalServerError
    })
}
pub fn err_is_not_found(err: &DE) -> bool {
    matches!(err, DE::NotFound)
}

pub fn err_is_deserialization_unexpected_null(err: &DE) -> bool {
    match err {
        DE::DeserializationError(e) => e.downcast_ref::<UnexpectedNullError>().is_some(),
        _ => false,
    }
}
