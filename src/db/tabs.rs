use deadpool_diesel::postgres::{Connection as Conn, Pool};
use diesel::connection::Connection;
use diesel::prelude::*;

use crate::{
    db::util::{err_is_not_found, get_conn},
    models::{
        tab::{NewTab, NewTabTag, Tab, TabWithTags, UpdateTabWithTags, UserListTab},
        tag::{NewTag, Tag},
    },
    schema::{
        tabs::{self, dsl as tabs_dsl},
        tabs_tags::{self, dsl as tt_dsl},
        tags::{self, dsl as tags_dsl},
    },
    types::{AppError, PaginatedResult, PaginationRequest},
};

pub async fn new_tab(conn: Conn, data: NewTab) -> Result<Tab, AppError> {
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

pub async fn get_tab(conn: Conn, user_id: String, tab_id: String) -> Result<Tab, AppError> {
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
    let c = get_conn(&pool).await?;
    let tab = get_tab(c, user_id.clone(), tab_id.clone()).await?;
    let c = get_conn(&pool).await?;
    let tags = get_tab_tags(c, user_id.clone(), tab_id.clone()).await?;
    Ok(TabWithTags { tab, tags })
}

pub async fn get_tab_tags(
    conn: Conn,
    user_id: String,
    tab_id: String,
) -> Result<Vec<Tag>, AppError> {
    conn.interact(|conn| {
        tags_dsl::tags
            .inner_join(tt_dsl::tabs_tags.on(tt_dsl::tag_id.eq(tags_dsl::id)))
            .filter(tt_dsl::tab_id.eq(tab_id))
            .filter(tags_dsl::user_id.eq(user_id))
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
    })
}

pub async fn get_user_tabs(
    pool: Pool,
    user_id: String,
    pr: PaginationRequest,
) -> Result<PaginatedResult<UserListTab>, AppError> {
    let offset = pr.offset();
    let limit = pr.limit();

    let c = get_conn(&pool).await?;
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
        .interact(move |conn| tabs_q.select(UserListTab::as_select()).get_results(conn))
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

pub async fn update_tab_and_tags(
    conn: Conn,
    tab_id: String,
    user_id: String,
    UpdateTabWithTags {
        tab,
        tags: updated_tags,
    }: UpdateTabWithTags,
) -> Result<TabWithTags, AppError> {
    let tid = tab_id.clone();
    let uid = user_id.clone();
    conn.interact(|co| {
        co.transaction(move |c| {
            // update tab
            let updated_tab = diesel::update(tabs_dsl::tabs.filter(tabs_dsl::id.eq(tid.clone())))
                .set((tabs_dsl::url.eq(tab.url), tabs_dsl::notes.eq(tab.notes)))
                .returning(Tab::as_returning())
                .get_result(c)?;

            let mut to_create: Vec<NewTag> = Vec::with_capacity(updated_tags.len());
            let mut to_attach: Vec<NewTabTag> = Vec::with_capacity(updated_tags.len());
            for mnt in updated_tags.iter() {
                if let Some(ref existing_id) = mnt.id {
                    to_attach.push(NewTabTag {
                        tab_id: tid.clone(),
                        tag_id: existing_id.clone(),
                    });
                } else {
                    to_create.push(NewTag {
                        user_id: mnt.user_id.clone(),
                        tag: mnt.tag.clone(),
                    });
                }
            }
            // create new tags & prepare to attach
            let created: Vec<String> = diesel::insert_into(tags::table)
                .values(to_create)
                .returning(tags::id)
                .get_results(c)?;
            let bid2 = tid.clone();
            for id in created.iter() {
                to_attach.push(NewTabTag {
                    tab_id: tid.clone(),
                    tag_id: id.clone(),
                });
            }

            // detatch all tags from tab
            diesel::delete(tabs_tags::table.filter(tt_dsl::tab_id.eq(bid2))).execute(c)?;

            // attach updated tags (created & existing)
            diesel::insert_into(tabs_tags::table)
                .values(to_attach)
                .execute(c)?;

            let tags = tags_dsl::tags
                .inner_join(tt_dsl::tabs_tags.on(tt_dsl::tag_id.eq(tags_dsl::id)))
                .filter(tt_dsl::tab_id.eq(tid.clone()))
                .filter(tags_dsl::user_id.eq(uid.clone()))
                .order(tags_dsl::tag.asc())
                .select(Tag::as_select())
                .get_results(c)?;
            Ok(TabWithTags {
                tab: updated_tab,
                tags,
            })
        })
    })
    .await
    .map_err(|e| {
        tracing::error!("error updating tab: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e: diesel::result::Error| {
        tracing::error!("error updating tab: {:?}", e);
        AppError::DBError
    })
}

#[allow(dead_code)]
pub async fn delete_tab(conn: Conn, user_id: String, tab_id: String) -> Result<usize, AppError> {
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
pub async fn delete_user_tabs(conn: Conn, user_id: String) -> Result<usize, AppError> {
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
pub async fn bulk_insert_tabs(conn: Conn, data: Vec<NewTab>) -> Result<Vec<Tab>, AppError> {
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
