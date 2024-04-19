use crate::{
    db::users,
    models::{
        CreatedInvite, CreatedUser, DeconfirmedUser, Invite, NewInvite, NewUser, User,
        UserConfirmationPayload, UserInviteResponse,
    },
    types::{AppError, AppState},
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

fn _users_router() -> Router<AppState> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/invites/:invite_id", get(get_invite))
        .route("/users/invites/:invite_id", post(send_invite))
        .route("/users/:user_id", post(confirm_user))
}

#[cfg(not(test))]
pub fn users_router() -> Router<AppState> {
    _users_router()
}
#[cfg(test)]
pub fn users_router() -> Router<AppState> {
    _users_router().route("/users/:user_id/deconfirm", post(deconfirm_user))
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

// cfg(test) for now, needs better security
#[cfg(test)]
pub async fn deconfirm_user(
    State(st): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<DeconfirmedUser>, AppError> {
    let conn = st.conn().await?;
    let res = users::deconfirm_user(conn, user_id).await?;
    Ok(Json(res))
}

// cfg(test) forever, lol
// #[cfg(test)]
// pub async fn _test_create_confirmed_user(
//     State(st): State<AppState>,
//     Json(new_user_data): Json<NewUser>,
// ) -> Result<Json<User>, AppError> {
//     let conn = st.conn().await?;
//     let res = users::new_user_confirmed(conn, new_user_data).await?;
//     Ok(Json(res))
// }
