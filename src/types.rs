use crate::auth::{get_claims, validate_claims};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{convert::Infallible, env};
use tracing::error;

#[derive(Clone)]
pub struct AppState {
    db: FakeDB,
    keys: Keys,
}

impl AppState {
    fn new(secret: &[u8], client_secret: String) -> Self {
        Self {
            db: FakeDB {},
            keys: Keys::new(secret, client_secret),
        }
    }
    pub fn from_env() -> Self {
        // panics
        let secret = env::var("JWT_SECRET").expect("missing JWT_SECRET");
        let client_secret = env::var("CLIENT_SECRET").expect("missing CLIENT_SECRET");
        Self::new(secret.as_bytes(), client_secret)
    }
    pub fn client_secret(&self) -> &str {
        self.keys.client_secret.as_ref()
    }
    pub fn encoding(&self) -> &EncodingKey {
        self.keys.encoding()
    }
    pub fn decoding(&self) -> &DecodingKey {
        self.keys.decoding()
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
pub(crate) struct FakeDB {}

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
    pub(crate) exp: i64,
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
        // let st = parts
        //     .extract_with_state::<AppState, _>(state)
        //     .await
        //     .map_err(|e| {
        //         error!("error extracting claims state {:?}", e);
        //         AppError::InternalServerError
        //     })?;
        let claims = get_claims(parts, st.decoding()).await?;
        validate_claims(&claims).await?;
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

#[derive(Debug)]
pub enum AppError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    ExpiredToken,
    InternalServerError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub expiration: DateTime<Utc>,
    pub token: String,
}
// #[async_trait]
// impl<AppState> FromRequestParts<AppState> for Keys {
//     type Rejection = AuthError;
//
//     async fn from_request_parts(
//         _parts: &mut Parts,
//         state: &AppState,
//     ) -> Result<Self, Self::Rejection> {
//         Ok(state.0.lock().unwrap().keys.clone())
//     }
// }

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
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
