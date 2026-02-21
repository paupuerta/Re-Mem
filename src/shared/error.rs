use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Result type for the application
pub type AppResult<T> = Result<T, AppError>;

/// Main application error type
/// SOLID: Follows Single Responsibility (error handling only)
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Authorization failed: {0}")]
    AuthorizationError(String),

    #[error("External API error: {0}")]
    ExternalApiError(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
    pub status: u16,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::AuthenticationError(_) => StatusCode::UNAUTHORIZED,
            AppError::AuthorizationError(_) => StatusCode::FORBIDDEN,
            AppError::DatabaseError(_)
            | AppError::InternalError(_)
            | AppError::ExternalApiError(_)
            | AppError::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_response(&self) -> ErrorResponse {
        let status = self.status_code().as_u16();
        ErrorResponse {
            error: self.to_string(),
            details: None,
            status,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_response = self.error_response();
        let status = self.status_code();

        tracing::error!("Error: {}", self);

        (status, Json(error_response)).into_response()
    }
}
