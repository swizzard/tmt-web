use chrono::Utc;
use deadpool_diesel::postgres::{Connection, Pool};
use diesel::prelude::*;
use tracing::error;

use super::util::get_conn;
use crate::{
    models::{NewSession, Session},
    schema::{
        sessions::{self, dsl as sessions_dsl},
        users::dsl as users_dsl,
    },
    types::{AppError, Claims},
};

pub async fn new_session(pool: Pool, user_email: String) -> Result<Session, AppError> {
    let conn = get_conn(pool).await?;
    let user_id: String = conn
        .interact(|conn| {
            users_dsl::users
                .filter(users_dsl::email.eq(user_email))
                .select(users_dsl::id)
                .first(conn)
        })
        .await
        .map_err(|e| {
            error!("error retrieving user id: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            error!("error retrieving user id: {:?}", e);
            AppError::DBError
        })?;
    let uid = user_id.clone();
    if let Some(s) = conn
        .interact(|conn| {
            sessions_dsl::sessions
                .filter(sessions_dsl::user_id.eq(uid))
                .select(Session::as_select())
                .first(conn)
                .optional()
        })
        .await
        .map_err(|e| {
            error!("error retrieving session: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            error!("error retrieving session: {:?}", e);
            AppError::DBError
        })?
    {
        tracing::info!("session: {:?}", &s);
        Ok(s)
    } else {
        let new_sess = NewSession { user_id };
        let created = conn
            .interact(|conn| {
                diesel::insert_into(sessions::table)
                    .values(new_sess)
                    .returning(Session::as_returning())
                    .get_result(conn)
                    .map_err(|e| {
                        error!("error creating session: {:?}", e);
                        AppError::DBError
                    })
            })
            .await
            .map_err(|e| {
                error!("error creating session: {:?}", e);
                AppError::DBError
            })??;
        Ok(created)
    }
}

pub async fn delete_session(conn: Connection, session_id: String) -> Result<(), AppError> {
    use crate::schema::sessions::dsl::*;
    let _ = conn
        .interact(|conn| diesel::delete(sessions.filter(id.eq(session_id))).execute(conn))
        .await
        .map_err(|e| {
            error!("error deleting session: {:?}", e);
            AppError::DBError
        })?;
    Ok(())
}

pub async fn session_from_claims(conn: Connection, claims: Claims) -> Result<Session, AppError> {
    use crate::schema::sessions::dsl::*;

    let resp: Option<Session> = conn
        .interact(|conn| {
            sessions
                .filter(id.eq(claims.jti))
                .filter(user_id.eq(claims.sub))
                .select(Session::as_select())
                .first(conn)
                .optional()
        })
        .await
        .map_err(|e| {
            error!("error retrieving session: {:?}", e);
            AppError::DBError
        })?
        .map_err(|e| {
            error!("error retrieving session: {:?}", e);
            AppError::DBError
        })?;
    if let Some(sess) = resp {
        if session_expired(&sess) {
            Err(AppError::ExpiredToken)
        } else {
            Ok(sess)
        }
    } else {
        Err(AppError::InvalidToken)
    }
}

fn session_expired(session: &Session) -> bool {
    let now = Utc::now().naive_utc();
    session.expires < now
}
