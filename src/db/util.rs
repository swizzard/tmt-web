use deadpool_diesel::postgres::{Connection, Pool};
use diesel::result::Error as DE;

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
