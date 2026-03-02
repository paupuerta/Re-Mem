use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Create User DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
}

/// User response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}
