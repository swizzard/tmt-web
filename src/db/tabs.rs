use deadpool_diesel::postgres::Connection;
use diesel::prelude::*;

use crate::{
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
        tracing::error!("error creating tab: {:?}", e);
        AppError::DBError
    })
}

pub async fn get_tab(conn: Connection, user_id: String, tab_id: String) -> Result<Tab, AppError> {
    if let Some(tab) = conn
        .interact(|conn| {
            tabs_dsl::tabs
                .filter(tabs_dsl::id.eq(tab_id))
                .filter(tabs_dsl::user_id.eq(user_id))
                .select(Tab::as_select())
                .first(conn)
                .optional()
        })
        .await
        .map_err(|e| {
            tracing::error!("error retrieving tab: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            tracing::error!("error retrieving tab: {:?}", e);
            AppError::DBError
        })?
    {
        Ok(tab)
    } else {
        Err(AppError::NotFound)
    }
}

// pub async fn user
