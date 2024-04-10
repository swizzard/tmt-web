mod auth;
mod types;
use auth::{encode_jwt, generate_claims};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use types::{AppError, AppState, AuthBody, AuthPayload, Claims};

pub fn make_app() -> Router {
    let st = AppState::from_env();
    Router::new()
        .route("/", get(hello_world))
        .route("/authorize", post(authorize))
        .route("/private", get(private))
        .with_state(st)
}

async fn hello_world() -> String {
    tracing::debug!("Hello world");
    String::from("hello")
}

async fn authorize(
    State(st): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AppError> {
    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err(AppError::MissingCredentials);
    }
    if payload.client_secret != st.client_secret() {
        return Err(AppError::WrongCredentials);
    }
    let claims = generate_claims(payload.client_id.clone());
    let token = encode_jwt(claims, &st.encoding())?;
    Ok(Json(AuthBody::new(token)))
}

async fn private(claims: Claims) -> Result<String, AppError> {
    tracing::debug!("private");
    Ok(format!("Hello {}", claims.sub))
}
