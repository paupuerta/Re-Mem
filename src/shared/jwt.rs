//! JWT utilities for encoding and decoding authentication tokens.

use crate::shared::error::{AppError, AppResult};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,
    pub iat: usize,
}

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-in-production".to_string())
}

fn expiration_days() -> i64 {
    std::env::var("JWT_EXPIRATION_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7)
}

pub fn encode_jwt(user_id: Uuid) -> AppResult<String> {
    let now = Utc::now().timestamp() as usize;
    let exp = (Utc::now() + chrono::Duration::days(expiration_days())).timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now,
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("JWT encode error: {e}")))
}

pub fn decode_jwt(token: &str) -> AppResult<Uuid> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::AuthenticationError(format!("Invalid token: {e}")))?;

    Uuid::parse_str(&data.claims.sub)
        .map_err(|_| AppError::AuthenticationError("Invalid user id in token".to_string()))
}
