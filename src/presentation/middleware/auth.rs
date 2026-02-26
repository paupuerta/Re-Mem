//! Authentication middleware — Axum extractor and middleware function for JWT validation.

use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::shared::{
    error::{AppError, AppResult},
    jwt::decode_jwt,
};

/// Extractor that validates the `Authorization: Bearer <token>` header
/// and injects the authenticated `user_id` into the handler.
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> AppResult<Self> {
        let token = extract_bearer_token(&parts.headers).ok_or_else(|| {
            AppError::AuthenticationError("Missing Authorization header".to_string())
        })?;
        let user_id = decode_jwt(token)?;
        Ok(AuthenticatedUser { user_id })
    }
}

/// Middleware function that rejects requests without a valid JWT.
/// Apply to protected route groups via `Router::layer(middleware::from_fn(require_auth))`.
pub async fn require_auth(request: Request, next: Next) -> Response {
    let token = extract_bearer_token(request.headers());
    match token.and_then(|t| decode_jwt(t).ok()) {
        Some(_) => next.run(request).await,
        None => {
            AppError::AuthenticationError("Missing or invalid token".to_string()).into_response()
        }
    }
}
