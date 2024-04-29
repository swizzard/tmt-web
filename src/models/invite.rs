use diesel::prelude::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Debug, diesel_derive_enum::DbEnum, Serialize, Deserialize, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::InviteStatus"]
pub enum InviteStatus {
    Created,
    Sent,
    Accepted,
    Expired,
}

#[derive(Debug, Deserialize, Queryable, Selectable, Serialize)]
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

#[derive(Debug, Identifiable, AsChangeset, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::invites)]
pub struct InviteUpdate {
    pub id: String,
    pub status: InviteStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserInviteResponse {
    pub email: String,
    pub invite_id: String,
    pub user_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserConfirmationPayload {
    pub email: String,
    pub invite_id: String,
}
