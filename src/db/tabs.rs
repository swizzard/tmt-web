use deadpool_diesel::postgres::Connection;
use diesel::prelude::*;

use crate::{
    db::util::err_is_not_found,
    models::{NewTab, Tab},
    schema::tabs,
    schema::tabs::dsl as tabs_dsl,
    types::AppError,
};

pub async fn new_tab(conn: Connection, data: NewTab) -> Result<Tab, AppError> {
    conn.interact(|conn| {
        diesel::insert_into(tabs::table)
            .values(data)
            .returning(Tab::as_returning())
            .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error creating tab: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        if err_is_not_found(&e) {
            AppError::NotFound
        } else {
            tracing::error!("error creating tab: {:?}", e);
            AppError::DBError
        }
    })
}

pub async fn get_tab(conn: Connection, user_id: String, tab_id: String) -> Result<Tab, AppError> {
    conn.interact(|conn| {
        tabs_dsl::tabs
            .filter(tabs_dsl::id.eq(tab_id))
            .filter(tabs_dsl::user_id.eq(user_id))
            .select(Tab::as_select())
            .first(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error retrieving tab: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        if err_is_not_found(&e) {
            AppError::NotFound
        } else {
            tracing::error!("error retrieving tab: {:?}", e);
            AppError::DBError
        }
    })
}

pub async fn delete_tab(
    conn: Connection,
    user_id: String,
    tab_id: String,
) -> Result<usize, AppError> {
    conn.interact(|conn| {
        diesel::delete(
            tabs_dsl::tabs
                .filter(tabs_dsl::id.eq(tab_id))
                .filter(tabs_dsl::user_id.eq(user_id)),
        )
        .execute(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error deleting tab: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        if err_is_not_found(&e) {
            AppError::NotFound
        } else {
            tracing::error!("error deleting tab: {:?}", e);
            AppError::DBError
        }
    })
}

#[cfg(test)]
pub async fn delete_user_tabs(conn: Connection, user_id: String) -> Result<usize, AppError> {
    conn.interact(|conn| {
        diesel::delete(tabs_dsl::tabs.filter(tabs_dsl::user_id.eq(user_id))).execute(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error deleting user tabs: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        tracing::error!("error deleting user tabs: {:?}", e);
        AppError::DBError
    })
}

// pub async fn user
