use crate::auth::get_claims;
use crate::types::{AppError, AppState};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Queryable, Selectable)]
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
        use crate::db::session::session_from_claims;
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
