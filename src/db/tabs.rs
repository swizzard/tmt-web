use deadpool_diesel::postgres::{Connection, Pool};
use diesel::prelude::*;

use crate::{
    db::util::{err_is_not_found, get_conn},
    models::{
        tab::{NewTab, Tab, TabTag, TabWithTags},
        tag::Tag,
    },
    schema::{
        tabs::{self, dsl as tabs_dsl},
        tabs_tags::dsl as tt_dsl,
        tags::dsl as tags_dsl,
    },
    types::{AppError, PaginatedResult, PaginationRequest},
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

pub async fn get_tab_with_tags(
    pool: Pool,
    user_id: String,
    tab_id: String,
) -> Result<TabWithTags, AppError> {
    let c = get_conn(pool.clone()).await?;
    let tid = tab_id.clone();
    let uid = user_id.clone();
    let tab = get_tab(c, user_id.clone(), tab_id.clone()).await?;
    let c = get_conn(pool.clone()).await?;
    let tags = c
        .interact(move |conn| {
            tags_dsl::tags
                .inner_join(tt_dsl::tabs_tags.on(tt_dsl::tag_id.eq(tags_dsl::id)))
                .filter(tt_dsl::tab_id.eq(tid))
                .filter(tags_dsl::user_id.eq(uid))
                .select(Tag::as_select())
                .get_results(conn)
        })
        .await
        .map_err(|e| {
            tracing::error!("error getting tab tags: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            tracing::error!("error getting tab tags: {:?}", e);
            AppError::DBError
        })?;
    Ok(TabWithTags { tab, tags })
}

pub async fn get_user_tabs(
    pool: Pool,
    user_id: String,
    pr: PaginationRequest,
) -> Result<PaginatedResult<Tab>, AppError> {
    let offset = pr.offset();
    let limit = pr.limit();

    let c = get_conn(pool).await?;
    let count_q = tabs_dsl::tabs.filter(tabs_dsl::user_id.eq(user_id.clone()));
    let count: i64 = c
        .interact(|conn| count_q.count().get_result(conn))
        .await
        .map_err(|e| {
            tracing::error!("error getting user tabs count: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            tracing::error!("error getting user tabs count: {:?}", e);
            AppError::DBError
        })?;
    let has_more = count - offset > limit;

    let tabs_q = tabs_dsl::tabs
        .filter(tabs_dsl::user_id.eq(user_id.clone()))
        .order(tabs_dsl::created_at.desc())
        .limit(limit)
        .offset(offset);
    let tabs = c
        .interact(move |conn| tabs_q.select(Tab::as_select()).get_results(conn))
        .await
        .map_err(|e| {
            tracing::error!("error getting user tabs: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            tracing::error!("error getting user tabs: {:?}", e);
            AppError::DBError
        })?;
    Ok(PaginatedResult {
        results: tabs,
        has_more,
    })
}

#[allow(dead_code)]
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

#[cfg(test)] // for now
pub async fn bulk_insert_tabs(conn: Connection, data: Vec<NewTab>) -> Result<Vec<Tab>, AppError> {
    conn.interact(|conn| {
        diesel::insert_into(tabs::table)
            .values(data)
            .returning(Tab::as_returning())
            .get_results(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error bulk inserting tabs: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        tracing::error!("error bulk inserting tabs: {:?}", e);
        AppError::DBError
    })
}
