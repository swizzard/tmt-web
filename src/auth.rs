use crate::types::{AppError, Claims};
use axum::{http::request::Parts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::prelude::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

pub(crate) async fn get_claims(parts: &mut Parts, key: &DecodingKey) -> Result<Claims, AppError> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|e| {
            tracing::error!("error parsing token: {:?}", e);
            AppError::InvalidToken
        })?;
    let token_data = decode::<Claims>(bearer.token(), key, &Validation::default())
        .map_err(|_| AppError::InvalidToken)?;
    let claims = token_data.claims;
    Ok(claims)
}
pub(crate) async fn validate_claims(claims: &Claims) -> Result<(), AppError> {
    if let Some(exp) = DateTime::from_timestamp(claims.exp, 0) {
        if exp < Utc::now() {
            Err(AppError::ExpiredToken)
        } else {
            Ok(())
        }
    } else {
        Err(AppError::InvalidToken)
    }
}
pub(crate) fn generate_claims(client_id: String) -> Claims {
    let exp = Utc::now() + chrono::TimeDelta::minutes(15);
    Claims {
        exp: exp.timestamp(),
        sub: client_id,
    }
}

pub(crate) fn encode_jwt(claims: Claims, key: &EncodingKey) -> Result<String, AppError> {
    encode(&Header::default(), &claims, key).map_err(|_| AppError::TokenCreation)
}
