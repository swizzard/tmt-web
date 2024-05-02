use crate::types::{AppError, Claims};
use axum::{http::request::Parts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use diesel::expression::functions::sql_function;
use diesel::sql_types::Text;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

pub(crate) async fn get_claims(parts: &mut Parts, key: &DecodingKey) -> Result<Claims, AppError> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|e| {
            tracing::error!("error parsing token: {:?}", e);
            AppError::InvalidToken
        })?;
    decode_claims(bearer.token(), key)
}

pub(crate) fn decode_claims(token: &str, key: &DecodingKey) -> Result<Claims, AppError> {
    let validation = Validation::new(Algorithm::HS256);
    _decode_claims(token, key, validation)
}
pub(crate) fn decode_claims_no_expiry(token: &str, key: &DecodingKey) -> Result<Claims, AppError> {
    use std::collections::HashSet;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;
    validation.validate_nbf = false;
    validation.required_spec_claims = HashSet::new();
    _decode_claims(token, key, validation)
}
fn _decode_claims(
    token: &str,
    key: &DecodingKey,
    validation: Validation,
) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(token, key, &validation).map_err(|e| {
        tracing::error!("error decoding token: {:?}", e);
        AppError::InvalidToken
    })?;
    let claims = token_data.claims;
    Ok(claims)
}

pub(crate) fn encode_jwt(claims: Claims, key: &EncodingKey) -> Result<String, AppError> {
    let headers = Header {
        alg: Algorithm::HS256,
        ..Header::default()
    };
    encode(&headers, &claims, key).map_err(|_| AppError::TokenCreation)
}

sql_function!(fn check_user_pwd(email: Text, pwd: Text) -> Bool);
