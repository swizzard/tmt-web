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
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(bearer.token(), key, &validation).map_err(|e| {
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
