use deadpool_diesel::postgres::{Connection, Pool};

use crate::types::AppError;

pub(crate) async fn get_conn(pool: Pool) -> Result<Connection, AppError> {
    pool.get().await.map_err(|e| {
        tracing::error!("db connection error {:?}", e);
        AppError::InternalServerError
    })
}
