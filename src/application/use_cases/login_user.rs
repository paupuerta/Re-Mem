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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::User;
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    use async_trait::async_trait;
    use uuid::Uuid;

    fn hash_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    struct MockUserRepo {
        user: Option<User>,
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn create(&self, _user: &User) -> AppResult<Uuid> { Ok(Uuid::new_v4()) }
        async fn find_by_id(&self, _id: Uuid) -> AppResult<Option<User>> { Ok(None) }
        async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
            Ok(self.user.clone().filter(|u| u.email == email))
        }
        async fn update(&self, _user: &User) -> AppResult<()> { Ok(()) }
        async fn delete(&self, _id: Uuid) -> AppResult<()> { Ok(()) }
    }

    #[tokio::test]
    async fn test_login_success() {
        let hash = hash_password("correctpassword");
        let user = User::new_with_password("user@example.com".to_string(), "Alice".to_string(), hash);
        let repo = Arc::new(MockUserRepo { user: Some(user) });
        let uc = LoginUserUseCase::new(repo);

        let result = uc.execute(LoginRequest {
            email: "user@example.com".to_string(),
            password: "correctpassword".to_string(),
        }).await;

        assert!(result.is_ok());
        assert!(!result.unwrap().token.is_empty());
    }

    #[tokio::test]
    async fn test_login_wrong_password_returns_auth_error() {
        let hash = hash_password("correctpassword");
        let user = User::new_with_password("user@example.com".to_string(), "Alice".to_string(), hash);
        let repo = Arc::new(MockUserRepo { user: Some(user) });
        let uc = LoginUserUseCase::new(repo);

        let result = uc.execute(LoginRequest {
            email: "user@example.com".to_string(),
            password: "wrongpassword".to_string(),
        }).await;

        assert!(matches!(result, Err(AppError::AuthenticationError(_))));
    }

    #[tokio::test]
    async fn test_login_unknown_email_returns_auth_error() {
        let repo = Arc::new(MockUserRepo { user: None });
        let uc = LoginUserUseCase::new(repo);

        let result = uc.execute(LoginRequest {
            email: "nobody@example.com".to_string(),
            password: "anypassword".to_string(),
        }).await;

        assert!(matches!(result, Err(AppError::AuthenticationError(_))));
    }
}
