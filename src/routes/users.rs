#[cfg(test)]
use crate::models::DeconfirmedUser;
use crate::{
    db::users,
    models::{
        CreatedInvite, CreatedUser, Invite, InviteUpdate, NewInvite, NewUser, User,
        UserConfirmationPayload, UserInviteResponse,
    },
    types::{AppError, AppState},
};
use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};

fn _users_router() -> Router<AppState> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/invites/:invite_id", put(update_invite))
        .route("/users/invites/:invite_id", get(get_invite))
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

// TODO(SHR): auth???
pub async fn update_invite(
    State(st): State<AppState>,
    Path(invite_id): Path<String>,
    Json(InviteUpdate { status, .. }): Json<InviteUpdate>,
) -> Result<Json<Invite>, AppError> {
    let conn = st.conn().await?;
    let inv = users::update_invite_status(conn, invite_id, status).await?;
    println!("new status: {:?}", inv.status);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{InviteStatus, NewConfirmedUser},
        routes::_test_utils::test_app,
        types::test_pool_from_env,
    };
    use fake::{Fake, Faker};
    use http::StatusCode;
    use serde_json::json;

    #[test_log::test(tokio::test)]
    async fn test_create_user() -> anyhow::Result<()> {
        use crate::{db::users, models::InviteStatus};

        let server = test_app(users_router())?;

        let create_user_data = Faker.fake::<NewUser>();
        let resp = server.post(&"/users").json(&json!(create_user_data)).await;

        resp.assert_status_ok();
        let resp_data = resp.json::<UserInviteResponse>();

        let pool = test_pool_from_env();

        let c = pool.get().await?;
        let user = users::get_user(c, resp_data.user_id).await?;
        let user_email = user.email.clone();
        let user_id = user.id.clone();
        assert_eq!(user_email.clone(), resp_data.email);
        let c = pool.get().await?;
        let ci = users::get_invite(c, resp_data.invite_id).await?;
        assert_eq!(ci.user_id, user_id.clone());
        assert_eq!(ci.email, user_email.clone());
        assert_eq!(ci.status, InviteStatus::Created);

        let c = pool.get().await?;
        let _d = users::delete_invite(c, ci.id).await?;

        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_create_user_email_exists() -> anyhow::Result<()> {
        use crate::db::users;
        use http::StatusCode;
        let pool = test_pool_from_env();
        let cud_1 = Faker.fake::<NewUser>();
        let email = cud_1.email.clone();
        let c = pool.get().await?;
        let _existing_user = users::new_user(c, cud_1).await?;

        let server = test_app(users_router())?;

        let mut cud_2 = Faker.fake::<NewUser>();
        cud_2.email = email.clone();
        let resp = server.post(&"/users").json(&json!(cud_2)).await;

        resp.assert_status(StatusCode::BAD_REQUEST);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_update_invite() -> anyhow::Result<()> {
        use crate::{db::users, models::InviteStatus};
        use http::header;
        let pool = test_pool_from_env();
        let ud = Faker.fake::<NewUser>();
        let c = pool.get().await?;
        let CreatedUser { id, email } = users::new_user(c, ud).await?;
        let user_id = id.clone();
        let user_email = email.clone();
        let inv = NewInvite {
            user_id: user_id.clone(),
            email: user_email.clone(),
        };
        let c = pool.get().await?;
        let CreatedInvite { id, .. } = users::new_invite(c, inv).await?;
        let inv_id = id.clone();
        let c = pool.get().await?;
        let Invite { status, .. } = users::get_invite(c, inv_id.clone()).await?;
        assert_eq!(status, InviteStatus::Created);

        let server = test_app(users_router())?;

        let url = format!("/users/invites/{}", inv_id.clone());
        let header_name = header::CONTENT_TYPE;
        let header_value = "application/json";
        let header_value = header::HeaderValue::from_static(header_value);

        let update_invite_data = InviteUpdate {
            id: inv_id.clone(),
            status: InviteStatus::Sent,
        };

        let resp = server
            .put(&url)
            .add_header(header_name, header_value)
            .json(&json!(update_invite_data))
            .await;
        resp.assert_status_ok();
        let c = pool.get().await?;
        let Invite {
            status: updated_status,
            ..
        } = users::get_invite(c, inv_id.clone()).await?;
        assert_eq!(updated_status, InviteStatus::Sent);
        let c = pool.get().await?;
        let _d = users::delete_invite(c, inv_id.clone()).await?;

        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_get_invite_exists() -> anyhow::Result<()> {
        let server = test_app(users_router())?;
        let pool = test_pool_from_env();
        let ud = Faker.fake::<NewUser>();
        let c = pool.get().await?;
        let CreatedUser { id, email } = users::new_user(c, ud).await?;
        let user_id = id.clone();
        let inv = NewInvite {
            user_id: user_id.clone(),
            email,
        };
        let c = pool.get().await?;
        let CreatedInvite { id: invite_id, .. } = users::new_invite(c, inv).await?;

        let url = format!("/users/invites/{}", invite_id.clone());
        let resp = server.get(&url).await;
        resp.assert_status_ok();
        let gotten_invite = resp.json::<Invite>();
        assert_eq!(gotten_invite.id, invite_id.clone());
        assert_eq!(gotten_invite.status, InviteStatus::Created);

        let c = pool.get().await?;
        let _d = users::delete_invite(c, invite_id.clone()).await?;
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_get_invite_doesnt_exist() -> anyhow::Result<()> {
        let server = test_app(users_router())?;
        let pool = test_pool_from_env();
        let ud = Faker.fake::<NewUser>();
        let c = pool.get().await?;
        let CreatedUser { id, email } = users::new_user(c, ud).await?;
        let user_id = id.clone();
        let inv = NewInvite {
            user_id: user_id.clone(),
            email,
        };
        let c = pool.get().await?;
        let CreatedInvite { id: inv_id, .. } = users::new_invite(c, inv).await?;
        let other_id = Faker.fake::<String>();

        let url = format!("/users/invites/{}", other_id.clone());
        let resp = server.get(&url).await;
        resp.assert_status(StatusCode::NOT_FOUND);

        let c = pool.get().await?;
        let _d = users::delete_invite(c, inv_id.clone()).await?;
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_confirm_user_exists() -> anyhow::Result<()> {
        let server = test_app(users_router())?;
        let pool = test_pool_from_env();
        let ud = Faker.fake::<NewUser>();
        let c = pool.get().await?;
        let cu = users::new_user(c, ud).await?;
        let user_id = cu.id.clone();
        let user_email = cu.email.clone();
        let inv = NewInvite {
            user_id: user_id.clone(),
            email: user_email.clone(),
        };
        let c = pool.get().await?;
        let ci = users::new_invite(c, inv).await?;
        let inv_id = ci.id.clone();

        let url = format!("/users/{}", user_id);
        let confirm_data = UserConfirmationPayload {
            email: user_email.clone(),
            invite_id: inv_id.clone(),
        };
        let resp = server.post(&url).json(&json!(confirm_data)).await;

        let c = pool.get().await?;
        let _d = users::delete_invite(c, inv_id.clone()).await?;
        let c = pool.get().await?;
        let _d = users::deconfirm_user(c, user_id.clone()).await?;

        resp.assert_status_ok();
        let confirmed_user = resp.json::<User>();
        assert_eq!(confirmed_user.id, user_id.clone());
        assert!(confirmed_user.confirmed);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_confirm_user_doesnt_exist() -> anyhow::Result<()> {
        let server = test_app(users_router())?;
        let pool = test_pool_from_env();
        let ud = Faker.fake::<NewUser>();
        let c = pool.get().await?;
        let cu = users::new_user(c, ud).await?;
        let user_id = cu.id.clone();
        let user_email = cu.email.clone();
        let inv = NewInvite {
            user_id: user_id.clone(),
            email: user_email.clone(),
        };
        let c = pool.get().await?;
        let ci = users::new_invite(c, inv).await?;
        let inv_id = ci.id.clone();
        let other_id = Faker.fake::<String>();
        let url = format!("/users/{}", other_id);
        let confirm_data = UserConfirmationPayload {
            email: user_email.clone(),
            invite_id: inv_id.clone(),
        };
        let resp = server.post(&url).json(&json!(confirm_data)).await;

        let c = pool.get().await?;
        let _d = users::delete_invite(c, inv_id.clone()).await?;

        resp.assert_status(StatusCode::NOT_FOUND);

        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_deconfirm_user_exists() -> anyhow::Result<()> {
        let server = test_app(users_router())?;
        let pool = test_pool_from_env();
        let mut ud = Faker.fake::<NewConfirmedUser>();
        ud.confirmed = true;
        let c = pool.get().await?;
        let cu = users::new_user_confirmed(c, ud).await?;
        let user_id = cu.id.clone();

        let url = format!("/users/{}/deconfirm", user_id.clone());
        let resp = server.post(&url).await;
        resp.assert_status_ok();
        let du = resp.json::<DeconfirmedUser>();
        assert_eq!(du.id, user_id.clone());
        let c = pool.get().await?;
        let db_user = users::get_user(c, user_id.clone()).await?;
        assert!(!db_user.confirmed);
        Ok(())
    }
    #[test_log::test(tokio::test)]
    async fn test_deconfirm_user_doesnt_exist() -> anyhow::Result<()> {
        let server = test_app(users_router())?;
        let pool = test_pool_from_env();
        let mut ud = Faker.fake::<NewConfirmedUser>();
        ud.confirmed = true;
        let c = pool.get().await?;
        let cu = users::new_user_confirmed(c, ud).await?;
        let bad_id = Faker.fake::<String>();
        let url = format!("/users/{}/deconfirm", bad_id.clone());
        let resp = server.post(&url).await;

        let c = pool.get().await?;
        let _d = users::deconfirm_user(c, cu.id.clone()).await?;

        resp.assert_status(StatusCode::NOT_FOUND);
        Ok(())
    }
}
