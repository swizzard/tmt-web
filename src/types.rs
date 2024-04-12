use crate::auth::get_claims;
use crate::models::Session;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use deadpool_diesel::postgres;
use dotenvy::dotenv;
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{convert::Infallible, env};

#[derive(Clone)]
pub struct AppState {
    pool: postgres::Pool,
    keys: Keys,
}

impl AppState {
    fn new(secret: &[u8], client_secret: String, db_url: String) -> Self {
        let manager = postgres::Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
        let pool = postgres::Pool::builder(manager).build().unwrap();
        let keys = Keys::new(secret, client_secret);
        Self { pool, keys }
    }
    pub fn from_env() -> Self {
        // panics
        dotenv().ok();
        let secret = env::var("JWT_SECRET").expect("missing JWT_SECRET");
        let client_secret = env::var("CLIENT_SECRET").expect("missing CLIENT_SECRET");
        let db_url = env::var("DATABASE_URL").expect("missing DATABASE_URL");
        Self::new(secret.as_bytes(), client_secret, db_url)
    }
    pub fn encoding(&self) -> &EncodingKey {
        self.keys.encoding()
    }
    pub fn decoding(&self) -> &DecodingKey {
        self.keys.decoding()
    }
    pub fn pool(&self) -> postgres::Pool {
        self.pool.clone()
    }
    pub async fn conn(&self) -> Result<postgres::Connection, AppError> {
        self.pool.get().await.map_err(|e| {
            tracing::error!("db connection error: {:?}", e);
            AppError::InternalServerError
        })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AppState
where
    Self: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;
    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}

#[derive(Clone)]
pub(crate) struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
    client_secret: String,
}

impl Keys {
    pub fn new(secret: &[u8], client_secret: String) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
            client_secret,
        }
    }
    fn encoding(&self) -> &EncodingKey {
        &self.encoding
    }
    fn decoding(&self) -> &DecodingKey {
        &self.decoding
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: String,
    pub(crate) jti: String,
    pub(crate) exp: i64,
}

impl Claims {
    pub fn from_session(session: &Session) -> Self {
        Self {
            sub: session.user_id.clone(),
            jti: session.id.clone(),
            exp: session.expires.and_utc().timestamp(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let st = AppState::from_ref(state);
        let claims = get_claims(parts, st.decoding()).await?;
        tracing::info!("claims {:?}", claims);
        Ok(claims)
    }
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}
impl AuthBody {
    pub(crate) fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogoutResult {
    pub session_id: String,
    pub ok: bool,
}
impl LogoutResult {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            ok: true,
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    ExpiredToken,
    InternalServerError,
    DBError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub expiration: DateTime<Utc>,
    pub token: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AppError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AppError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AppError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            AppError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Token Expired"),
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::DBError => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
