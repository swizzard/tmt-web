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
        db::tags,
        models::tab::{NewTab, Tab},
        models::tag::{NewTag, Tag},
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
    pub async fn bulk_create_tags(
        conn: Connection,
        user_id: String,
        n_tags: usize,
    ) -> Result<Vec<Tag>, AppError> {
        let mut tags = Vec::with_capacity(n_tags);
        for _ in 0..n_tags {
            let mut t = Faker.fake::<NewTag>();
            t.user_id.clone_from(&user_id);
            tags.push(t);
        }
        tags::bulk_insert_tags(conn, tags).await
    }
    pub async fn create_tags_reverse_alpha(
        conn: Connection,
        user_id: String,
    ) -> Result<Vec<Tag>, AppError> {
        let tags = ('a'..='z')
            .rev()
            .map(|c| NewTag {
                user_id: user_id.clone(),
                tag: c.to_string(),
            })
            .collect();
        tags::bulk_insert_tags(conn, tags).await
    }
    pub async fn bulk_create_tags_from_strings(
        conn: Connection,
        user_id: String,
        ts: Vec<String>,
    ) -> Result<Vec<Tag>, AppError> {
        let tags = ts
            .into_iter()
            .map(|t| NewTag {
                user_id: user_id.clone(),
                tag: t,
            })
            .collect();
        tags::bulk_insert_tags(conn, tags).await
    }
}
