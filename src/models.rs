use crate::auth::get_claims;
use crate::types::{AppError, AppState};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Queryable, Selectable, Serialize, Identifiable, PartialEq)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub confirmed: bool,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct NewUser {
    pub email: String,
    pub password: String,
}

#[cfg(test)]
#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct NewConfirmedUser {
    pub email: String,
    pub password: String,
    pub confirmed: bool,
}

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
pub struct CreatedUser {
    pub id: String,
    pub email: String,
}

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
pub struct DeconfirmedUser {
    pub id: String,
    pub email: String,
}

#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires: chrono::NaiveDateTime,
}

#[async_trait]
impl<S> FromRequestParts<S> for Session
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use crate::db::sessions::session_from_claims;
        let st = AppState::from_ref(state);
        let claims = get_claims(parts, st.decoding()).await?;
        let conn = st.conn().await?;
        session_from_claims(conn, claims).await
    }
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::sessions)]
pub struct NewSession {
    pub user_id: String,
}

#[derive(Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::tabs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tab {
    pub id: String,
    pub user_id: String,
    pub url: String,
    pub notes: Option<String>,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::tabs)]
pub struct NewTab {
    pub user_id: String,
    pub url: String,
    pub notes: Option<String>,
}

#[derive(Debug, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::InviteStatus"]
pub enum InviteStatus {
    Created,
    Sent,
    Accepted,
    Expired,
}

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::invites)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Invite {
    pub id: String,
    pub user_id: String,
    pub email: String,
    pub status: InviteStatus,
    pub expires: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::invites)]
pub struct NewInvite {
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Selectable, Queryable, Serialize)]
#[diesel(table_name = crate::schema::invites)]
pub struct CreatedInvite {
    pub id: String,
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct UserInviteResponse {
    pub email: String,
    pub invite_id: String,
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UserConfirmationPayload {
    pub email: String,
    pub invite_id: String,
}
