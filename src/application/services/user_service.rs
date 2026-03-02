use crate::{
    domain::{entities::User, repositories::UserRepository},
    AppResult,
};
use std::sync::Arc;
use uuid::Uuid;

use super::super::dtos::{CreateUserRequest, UserDto};

/// User service - handles user-related operations
/// SOLID: Single Responsibility - only handles user operations
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> AppResult<UserDto> {
        let user = User::new(req.email, req.name);
        let user_id = self.user_repo.create(&user).await?;
        Ok(UserDto {
            id: user_id,
            email: user.email,
            name: user.name,
        })
    }

    pub async fn get_user(&self, user_id: Uuid) -> AppResult<UserDto> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| crate::AppError::NotFound("User not found".to_string()))?;

        Ok(UserDto {
            id: user.id,
            email: user.email,
            name: user.name,
        })
    }
}
