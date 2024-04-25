use deadpool_diesel::postgres::{Connection, Pool};
use diesel::dsl::exists;
use diesel::prelude::*;

use crate::{
    db::util::{err_is_not_found, get_conn},
    models::{
        tab::{
            AttachTagRequest, CreatedTabTag, DetachTagRequest, NewTabTag, TabTag,
            TagAttachedResponse, TagDetachedResponse,
        },
        tag::{NewTag, Tag},
    },
    schema::{
        tabs::dsl as tabs_dsl,
        tabs_tags::{self, dsl as tt_dsl},
        tags::{self, dsl as tags_dsl},
    },
    types::{AppError, PaginatedResult, PaginationRequest},
};

pub async fn new_tag(conn: Connection, data: NewTag) -> Result<Tag, AppError> {
    conn.interact(|conn| {
        diesel::insert_into(tags::table)
            .values(data)
            .returning(Tag::as_returning())
            .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error creating tag: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        if err_is_not_found(&e) {
            AppError::NotFound
        } else {
            tracing::error!("error creating tag: {:?}", e);
            AppError::DBError
        }
    })
}

pub async fn delete_tag(
    conn: Connection,
    user_id: String,
    tag_id: String,
) -> Result<usize, AppError> {
    conn.interact(|conn| {
        diesel::delete(
            tags_dsl::tags
                .filter(tags_dsl::id.eq(tag_id))
                .filter(tags_dsl::user_id.eq(user_id)),
        )
        .execute(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error deleting tag: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        tracing::error!("error deleting tag: {:?}", e);
        AppError::DBError
    })
}

pub async fn attach_tag(
    pool: Pool,
    AttachTagRequest {
        user_id,
        tab_id,
        tag_id,
    }: AttachTagRequest,
) -> Result<TagAttachedResponse, AppError> {
    let buid = user_id.clone();
    let bid = tab_id.clone();
    let gid = tag_id.clone();
    let guid = user_id.clone();
    if tab_belongs(get_conn(pool.clone()).await?, bid, buid).await?
        && tag_belongs(get_conn(pool.clone()).await?, gid, guid).await?
    {
        let ntt = NewTabTag { tab_id, tag_id };
        let conn = get_conn(pool).await?;
        let CreatedTabTag { tab_id, tag_id } = conn
            .interact(move |conn| {
                diesel::insert_into(tabs_tags::table)
                    .values(ntt)
                    .returning(CreatedTabTag::as_returning())
                    .get_result(conn)
            })
            .await
            .map_err(|e| {
                tracing::error!("error creating tag: {:?}", e);
                AppError::DBError
            })?
            .map_err(|e| {
                if err_is_not_found(&e) {
                    AppError::NotFound
                } else {
                    tracing::error!("error creating tag: {:?}", e);
                    AppError::DBError
                }
            })?;
        Ok(TagAttachedResponse {
            user_id,
            tab_id,
            tag_id,
        })
    } else {
        Err(AppError::WrongCredentials)
    }
}

pub async fn detach_tag(
    pool: Pool,
    DetachTagRequest {
        user_id,
        tab_id,
        tag_id,
    }: DetachTagRequest,
) -> Result<TagDetachedResponse, AppError> {
    let buid = user_id.clone();
    let bid = tab_id.clone();
    let gid = tag_id.clone();
    let bid2 = bid.clone();
    let gid2 = gid.clone();
    let guid = user_id.clone();
    tracing::info!("tab_id: {:?}", &bid2);
    tracing::info!("tag_id: {:?}", &gid2);
    if tab_belongs(get_conn(pool.clone()).await?, bid, buid).await?
        && tag_belongs(get_conn(pool.clone()).await?, gid, guid).await?
    {
        let conn = get_conn(pool).await?;
        conn.interact(|conn| {
            diesel::delete(tabs_tags::table)
                .filter(tt_dsl::tab_id.eq(bid2))
                .filter(tt_dsl::tag_id.eq(gid2))
                .execute(conn)
        })
        .await
        .map_err(|e| {
            tracing::error!("error detaching tag: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            // delete doesn't throw error if "not found"
            tracing::error!("error detaching tag: {:?}", e);
            AppError::DBError
        })?;
        Ok(TagDetachedResponse {
            user_id,
            tab_id,
            tag_id,
        })
    } else {
        Err(AppError::WrongCredentials)
    }
}

pub async fn get_user_tags(
    conn: Connection,
    user_id: String,
    pr: PaginationRequest,
) -> Result<PaginatedResult<Tag>, AppError> {
    let offset = pr.offset();
    let limit = pr.limit();
    let uid = user_id.clone();

    let cuid = uid.clone();
    let count: i64 = conn
        .interact(move |conn| {
            tags_dsl::tags
                .filter(tags_dsl::user_id.eq(cuid))
                .count()
                .get_result(conn)
        })
        .await
        .map_err(|e| {
            tracing::error!("error getting user tags count: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            tracing::error!("error getting user tags count: {:?}", e);
            AppError::DBError
        })?;
    let has_more = count - offset > limit;
    let tuid = uid.clone();
    let tags = conn
        .interact(move |conn| {
            tags_dsl::tags
                .filter(tags_dsl::user_id.eq(tuid))
                .order(tags_dsl::tag.desc())
                .limit(limit)
                .offset(offset)
                .get_results(conn)
        })
        .await
        .map_err(|e| {
            tracing::error!("error getting user tags: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            tracing::error!("error getting user tags: {:?}", e);
            AppError::DBError
        })?;
    Ok(PaginatedResult {
        results: tags,
        has_more,
    })
}

pub async fn get_user_tags_fuzzy(
    conn: Connection,
    user_id: String,
    to_match: String,
) -> Result<Vec<Tag>, AppError> {
    conn.interact(move |conn| {
        tags_dsl::tags
            .filter(tags_dsl::user_id.eq(user_id))
            .filter(tags_dsl::tag.ilike(format!("%{}%", to_match)))
            .order(tags_dsl::tag.asc())
            .limit(10)
            .get_results(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error getting user tags: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        tracing::error!("error getting user tags: {:?}", e);
        AppError::DBError
    })
}
async fn tab_belongs(conn: Connection, tab_id: String, user_id: String) -> Result<bool, AppError> {
    conn.interact(move |conn| {
        diesel::select(exists(
            tabs_dsl::tabs
                .filter(tabs_dsl::id.eq(tab_id))
                .filter(tabs_dsl::user_id.eq(user_id)),
        ))
        .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error getting tab: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        tracing::error!("error getting tab: {:?}", e);
        AppError::DBError
    })
}

async fn tag_belongs(conn: Connection, tag_id: String, user_id: String) -> Result<bool, AppError> {
    conn.interact(move |conn| {
        diesel::select(exists(
            tags_dsl::tags
                .filter(tags_dsl::id.eq(tag_id))
                .filter(tags_dsl::user_id.eq(user_id)),
        ))
        .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error getting tag: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        tracing::error!("error getting tag: {:?}", e);
        AppError::DBError
    })
}

#[cfg(test)]
pub async fn delete_user_tags(conn: Connection, user_id: String) -> Result<usize, AppError> {
    conn.interact(|conn| {
        diesel::delete(tags_dsl::tags.filter(tags_dsl::user_id.eq(user_id))).execute(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error deleting user tags: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        tracing::error!("error deleting user tags: {:?}", e);
        AppError::DBError
    })
}

#[cfg(test)]
pub async fn get_tag(conn: Connection, tag_id: String) -> Result<Tag, AppError> {
    conn.interact(move |conn| {
        tags_dsl::tags
            .filter(tags_dsl::id.eq(tag_id))
            .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error getting tag: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        if err_is_not_found(&e) {
            AppError::NotFound
        } else {
            tracing::error!("error getting tag: {:?}", e);
            AppError::DBError
        }
    })
}

#[cfg(test)]
pub async fn mk_tab_tag(
    conn: Connection,
    tab_id: String,
    tag_id: String,
) -> Result<CreatedTabTag, AppError> {
    let ntt = NewTabTag { tab_id, tag_id };
    conn.interact(move |conn| {
        diesel::insert_into(tabs_tags::table)
            .values(ntt)
            .returning(CreatedTabTag::as_returning())
            .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error creating tab tag: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        if err_is_not_found(&e) {
            AppError::NotFound
        } else {
            tracing::error!("error creating tab tag: {:?}", e);
            AppError::DBError
        }
    })
}

#[cfg(test)]
pub async fn bulk_mk_tab_tags(
    conn: Connection,
    data: Vec<NewTabTag>,
) -> Result<Vec<TabTag>, AppError> {
    conn.interact(|conn| {
        diesel::insert_into(tabs_tags::table)
            .values(data)
            .returning(TabTag::as_returning())
            .get_results(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error creating tab tags: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        tracing::error!("error creating tab tags: {:?}", e);
        AppError::DBError
    })
}
#[cfg(test)]
pub async fn get_tab_tag(
    conn: Connection,
    tab_id: String,
    tag_id: String,
) -> Result<CreatedTabTag, AppError> {
    conn.interact(move |conn| {
        tabs_tags::table
            .filter(tabs_tags::tab_id.eq(tab_id))
            .filter(tabs_tags::tag_id.eq(tag_id))
            .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error getting tab tag: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        if err_is_not_found(&e) {
            AppError::NotFound
        } else {
            tracing::error!("error getting tab tag: {:?}", e);
            AppError::DBError
        }
    })
}
#[cfg(test)] // for now
pub async fn bulk_insert_tags(conn: Connection, data: Vec<NewTag>) -> Result<Vec<Tag>, AppError> {
    conn.interact(|conn| {
        diesel::insert_into(tags::table)
            .values(data)
            .returning(Tag::as_returning())
            .get_results(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error bulk inserting tags: {:?}", e);
        AppError::DBError
    })?
    .map_err(|e| {
        tracing::error!("error bulk inserting tags: {:?}", e);
        AppError::DBError
    })
}
