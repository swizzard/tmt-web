use crate::{
    db::users,
    models::{
        CreatedInvite, CreatedUser, Invite, NewInvite, NewUser, User, UserConfirmationPayload,
        UserInviteResponse,
    },
    types::{AppError, AppState},
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

pub fn users_router() -> Router<AppState> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/invites/:invite_id", get(get_invite))
        .route("/users/invites/:invite_id", post(send_invite))
        .route("/users/:user_id", post(confirm_user))
}

pub async fn create_user(
    State(st): State<AppState>,
    Json(payload): Json<NewUser>,
) -> Result<Json<UserInviteResponse>, AppError> {
    let c = st.conn().await?;
    let CreatedUser { id, email } = users::new_user(c, payload).await?;
    let user_id = id.clone();
    let user_email = email.clone();
    let c = st.conn().await?;
    let inv_data = NewInvite {
        user_id,
        email: user_email,
    };
    let CreatedInvite { id, user_id, email } = users::new_invite(c, inv_data).await?;
    let resp_data = UserInviteResponse {
        email,
        invite_id: id,
        user_id,
    };
    Ok(Json(resp_data))
}

pub async fn send_invite(
    State(st): State<AppState>,
    Path(invite_id): Path<String>,
) -> Result<Json<Invite>, AppError> {
    let conn = st.conn().await?;
    let inv = users::mark_invite_sent(conn, invite_id).await?;
    Ok(Json(inv))
}

pub async fn get_invite(
    State(st): State<AppState>,
    Path(invite_id): Path<String>,
) -> Result<Json<Invite>, AppError> {
    let conn = st.conn().await?;
    let inv = users::get_invite(conn, invite_id).await?;
    Ok(Json(inv))
}

// POST to /users/<user_id> {email, invite_id}
pub async fn confirm_user(
    State(st): State<AppState>,
    Path(user_id): Path<String>,
    Json(UserConfirmationPayload { email, invite_id }): Json<UserConfirmationPayload>,
) -> Result<Json<User>, AppError> {
    let user = users::confirm_user(st.pool(), invite_id, user_id, email).await?;
    Ok(Json(user))
}
