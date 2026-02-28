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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::User;
    use async_trait::async_trait;
    use uuid::Uuid;

    struct MockUserRepo {
        existing_email: Option<String>,
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn create(&self, _user: &User) -> AppResult<Uuid> {
            Ok(Uuid::new_v4())
        }
        async fn find_by_id(&self, _id: Uuid) -> AppResult<Option<User>> {
            Ok(None)
        }
        async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
            if self.existing_email.as_deref() == Some(email) {
                Ok(Some(User::new(email.to_string(), "Existing".to_string())))
            } else {
                Ok(None)
            }
        }
        async fn update(&self, _user: &User) -> AppResult<()> { Ok(()) }
        async fn delete(&self, _id: Uuid) -> AppResult<()> { Ok(()) }
    }

    fn repo(existing_email: Option<&str>) -> Arc<MockUserRepo> {
        Arc::new(MockUserRepo {
            existing_email: existing_email.map(str::to_string),
        })
    }

    #[tokio::test]
    async fn test_register_success() {
        let uc = RegisterUserUseCase::new(repo(None));
        let result = uc.execute(RegisterRequest {
            email: "new@example.com".to_string(),
            name: "Alice".to_string(),
            password: "securepassword".to_string(),
        }).await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(!res.token.is_empty());
        assert_eq!(res.user.email, "new@example.com");
    }

    #[tokio::test]
    async fn test_register_duplicate_email_returns_conflict() {
        let uc = RegisterUserUseCase::new(repo(Some("taken@example.com")));
        let result = uc.execute(RegisterRequest {
            email: "taken@example.com".to_string(),
            name: "Bob".to_string(),
            password: "securepassword".to_string(),
        }).await;
        assert!(matches!(result, Err(AppError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_register_short_password_returns_validation_error() {
        let uc = RegisterUserUseCase::new(repo(None));
        let result = uc.execute(RegisterRequest {
            email: "user@example.com".to_string(),
            name: "Carol".to_string(),
            password: "short".to_string(),
        }).await;
        assert!(matches!(result, Err(AppError::ValidationError(_))));
    }
}
