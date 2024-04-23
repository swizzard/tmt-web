use crate::types::AppError;
use deadpool_diesel::postgres::{Connection, Pool};
use diesel::result::{Error as DE, UnexpectedNullError};

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

#[cfg(test)]
pub mod test_util {
    use super::*;
    use crate::{
        db::tabs,
        models::{NewTab, Tab},
    };
    use fake::{Fake, Faker};

    pub async fn bulk_create_tabs(
        conn: Connection,
        user_id: String,
        n_tabs: usize,
    ) -> Result<Vec<Tab>, AppError> {
        let mut tabs = Vec::with_capacity(n_tabs);
        for _ in 0..n_tabs {
            let mut t = Faker.fake::<NewTab>();
            t.user_id.clone_from(&user_id);
            tabs.push(t);
        }
        tabs::bulk_insert_tabs(conn, tabs).await
    }
}
