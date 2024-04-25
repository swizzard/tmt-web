use crate::auth::{encode_jwt, get_claims};
use crate::models::session::Session;
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
    fn new(secret: &[u8], db_url: String) -> Self {
        let keys = Keys::new(secret);
        let pool = make_pool(db_url);
        Self { pool, keys }
    }
    #[cfg(not(test))]
    pub fn from_env() -> Self {
        // panics
        dotenv().ok();
        tracing::info!("loading env");
        let secret = env::var("JWT_SECRET").expect("missing JWT_SECRET");
        let db_url = env::var("DATABASE_URL").expect("missing DATABASE_URL");
        Self::new(secret.as_bytes(), db_url)
    }
    #[cfg(test)]
    pub fn from_env() -> Self {
        // panics
        dotenv().ok();
        tracing::info!("loading test env");
        let secret = env::var("JWT_SECRET_TEST").expect("missing JWT_SECRET_TEST");
        let db_url = env::var("DATABASE_URL_TEST").expect("missing DATABASE_URL_TEST");
        Self::new(secret.as_bytes(), db_url)
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
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
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

    pub fn to_token(self, encoding_key: &EncodingKey) -> Result<String, AppError> {
        encode_jwt(self, encoding_key)
    }
    #[cfg(test)]
    pub(crate) fn test_to_token(self) -> Result<String, AppError> {
        let key = env::var("JWT_SECRET_TEST").expect("missing JWT_SECRET_TEST");
        self.to_token(&EncodingKey::from_secret(key.as_bytes()))
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

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}
impl AuthBody {
    pub(crate) fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthPayload {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationRequest {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl PaginationRequest {
    pub fn limit(&self) -> i64 {
        self.page_size.unwrap_or(25)
    }
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1)
    }
    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.limit()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginatedResult<T> {
    pub results: Vec<T>,
    pub has_more: bool,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub expiration: DateTime<Utc>,
    pub token: String,
}

#[derive(Debug, Clone)]
pub enum AppError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    ExpiredToken,
    InternalServerError,
    DBError,
    NotFound,
    DBErrorWithMessage(String),
    BadRequest,
}

impl AppError {
    fn to_status_message(&self) -> (StatusCode, String) {
        match self {
            AppError::WrongCredentials => (StatusCode::FORBIDDEN, "Wrong credentials".into()),
            AppError::MissingCredentials => {
                (StatusCode::UNAUTHORIZED, "Missing credentials".into())
            }
            AppError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error".into(),
            ),
            AppError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token".into()),
            AppError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Token Expired".into()),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            ),
            AppError::DBError => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not Found".into()),
            AppError::DBErrorWithMessage(msg) => {
                let err_msg = format!("Database error: {}", msg);
                (StatusCode::BAD_REQUEST, err_msg)
            }
            AppError::BadRequest => (StatusCode::BAD_REQUEST, "Invalid request".into()),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (_, msg) = self.to_status_message();
        write!(f, "Application Error: {}", msg)
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = self.to_status_message();
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

pub(crate) fn make_pool(db_url: String) -> postgres::Pool {
    let manager = postgres::Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    postgres::Pool::builder(manager).build().unwrap()
}

#[cfg(test)]
pub(crate) fn test_pool_from_env() -> postgres::Pool {
    let db_url = env::var("DATABASE_URL_TEST").expect("missing DATABASE_URL_TEST");
    make_pool(db_url)
}
