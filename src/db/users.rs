use super::util::get_conn;
use crate::{
    models::{CreatedInvite, CreatedUser, Invite, InviteStatus, NewInvite, NewUser, User},
    schema::invites,
    schema::invites::dsl as invites_dsl,
    schema::users,
    schema::users::dsl as users_dsl,
    types::AppError,
};
use chrono::Utc;
use deadpool_diesel::postgres::{Connection, Pool};
use diesel::prelude::*;

pub async fn new_user(conn: Connection, user: NewUser) -> Result<CreatedUser, AppError> {
    conn.interact(|conn| {
        diesel::insert_into(users::table)
            .values(user)
            .returning(CreatedUser::as_returning())
            .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error creating user: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })?
    .map_err(|e| {
        tracing::error!("error creating user: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })
}

pub async fn new_invite(conn: Connection, invite: NewInvite) -> Result<CreatedInvite, AppError> {
    conn.interact(|conn| {
        diesel::insert_into(invites::table)
            .values(invite)
            .returning(CreatedInvite::as_returning())
            .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error creating invite: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })?
    .map_err(|e| {
        tracing::error!("error creating invite: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })
}

pub async fn mark_invite_sent(conn: Connection, invite_id: String) -> Result<Invite, AppError> {
    conn.interact(|conn| {
        diesel::update(invites_dsl::invites)
            .filter(invites_dsl::id.eq(invite_id))
            .set(invites_dsl::status.eq(InviteStatus::Sent))
            .returning(Invite::as_returning())
            .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error creating invite: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })?
    .map_err(|e| {
        tracing::error!("error creating invite: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })
}

pub async fn confirm_user(
    pool: Pool,
    invite_id: String,
    user_id: String,
    email: String,
) -> Result<User, AppError> {
    let p = pool.clone();
    let conn = get_conn(p).await?;
    let _ = confirm_invite(conn, invite_id.clone(), user_id.clone(), email.clone()).await?;
    let conn = get_conn(pool).await?;
    conn.interact(|conn| {
        diesel::update(users_dsl::users)
            .filter(users_dsl::id.eq(user_id))
            .set(users_dsl::confirmed.eq(true))
            .returning(User::as_returning())
            .get_result(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error confirming invite: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })?
    .map_err(|e| {
        tracing::error!("error confirming invite: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })
}

pub(crate) async fn confirm_invite(
    conn: Connection,
    invite_id: String,
    user_id: String,
    email: String,
) -> Result<String, AppError> {
    conn.interact(|conn| {
        let now = Utc::now().naive_utc();
        diesel::update(invites_dsl::invites)
            .filter(invites_dsl::id.eq(invite_id))
            .filter(invites_dsl::user_id.eq(user_id))
            .filter(invites_dsl::email.eq(email))
            .filter(invites_dsl::expires.gt(now))
            .set(invites_dsl::status.eq(InviteStatus::Accepted))
            .returning(invites_dsl::id)
            .get_result(conn)
    })
    .await
    // TODO(SHR): introspect/tease apart these errors
    .map_err(|e| {
        tracing::error!("error confirming invite: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })?
    .map_err(|e| {
        tracing::error!("error confirming invite: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })
}

pub async fn get_invite(conn: Connection, invite_id: String) -> Result<Invite, AppError> {
    conn.interact(|conn| {
        invites_dsl::invites
            .filter(invites_dsl::id.eq(invite_id))
            .first(conn)
    })
    .await
    .map_err(|e| {
        tracing::error!("error confirming invite: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })?
    .map_err(|e| {
        tracing::error!("error confirming invite: {:?}", e);
        AppError::DBErrorWithMessage(e.to_string())
    })
}
