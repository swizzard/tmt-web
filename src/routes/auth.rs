use crate::{
    auth::encode_jwt,
    db::{
        sessions::{delete_session, new_session},
        validate_password,
    },
    types::{AppError, AppState, AuthBody, AuthPayload, Claims, LogoutResult},
};
use axum::{extract::State, Json};

#[axum::debug_handler]
pub(crate) async fn authorize(
    State(st): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AppError> {
    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err(AppError::MissingCredentials);
    }
    let conn = st.conn().await?;
    let clid = payload.client_id.clone();
    let pwd_valid = validate_password(conn, clid, payload.client_secret).await?;
    if !pwd_valid {
        return Err(AppError::WrongCredentials);
    } else {
        let session = new_session(st.pool(), payload.client_id).await?;
        let claims = Claims::from_session(&session);
        let token = encode_jwt(claims, &st.encoding())?;
        Ok(Json(AuthBody::new(token)))
    }
}

#[axum::debug_handler]
pub(crate) async fn logout(
    State(st): State<AppState>,
    claims: Claims,
) -> Result<Json<LogoutResult>, AppError> {
    let sess_id = claims.jti.clone();
    let conn = st.conn().await?;
    let _ = delete_session(conn, claims.jti).await?;
    Ok(axum::Json(LogoutResult::new(sess_id)))
}
