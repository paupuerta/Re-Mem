//! RegisterUser use case - create a new account and return a JWT.

use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use crate::{
    application::dtos::{AuthResponse, RegisterRequest, UserDto},
    domain::{entities::User, repositories::UserRepository},
    shared::{
        error::{AppError, AppResult},
        jwt::encode_jwt,
    },
};

pub struct RegisterUserUseCase {
    user_repo: Arc<dyn UserRepository>,
}

impl RegisterUserUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, req: RegisterRequest) -> AppResult<AuthResponse> {
        // Validate password length
        if req.password.len() < 8 {
            return Err(AppError::ValidationError(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        // Check email uniqueness
        if self.user_repo.find_by_email(&req.email).await?.is_some() {
            return Err(AppError::Conflict(
                "An account with this email already exists".to_string(),
            ));
        }

        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|e| AppError::InternalError(format!("Password hashing failed: {e}")))?
            .to_string();

        // Persist user
        let user = User::new_with_password(req.email, req.name, password_hash);
        self.user_repo.create(&user).await?;

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
