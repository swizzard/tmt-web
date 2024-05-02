use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use diesel::{associations::Identifiable, deserialize::Queryable, Insertable, Selectable};
use serde::{Deserialize, Serialize};

use crate::{auth::get_claims, types::AppError, AppState};

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

#[derive(Debug, Deserialize, Serialize)]
pub struct RenewSessionRequest {
    pub token: String,
}
