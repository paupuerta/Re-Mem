//! LoginUser use case - verify credentials and return a JWT.

use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::{
    application::dtos::{AuthResponse, LoginRequest, UserDto},
    domain::repositories::UserRepository,
    shared::{
        error::{AppError, AppResult},
        jwt::encode_jwt,
    },
};

pub struct LoginUserUseCase {
    user_repo: Arc<dyn UserRepository>,
}

impl LoginUserUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, req: LoginRequest) -> AppResult<AuthResponse> {
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await?
            .ok_or_else(|| {
                AppError::AuthenticationError("Invalid email or password".to_string())
            })?;

        let hash = user.password_hash.as_deref().ok_or_else(|| {
            AppError::AuthenticationError("Invalid email or password".to_string())
        })?;

        let parsed = PasswordHash::new(hash)
            .map_err(|_| AppError::InternalError("Password hash corrupted".to_string()))?;

        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed)
            .map_err(|_| AppError::AuthenticationError("Invalid email or password".to_string()))?;

        let token = encode_jwt(user.id)?;
        Ok(AuthResponse {
            token,
            user: UserDto {
                id: user.id,
                email: user.email,
                name: user.name,
            },
        })
    }
}
