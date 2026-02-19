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

/// Create Card DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCardRequest {
    pub question: String,
    pub answer: String,
}

/// Card response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub question: String,
    pub answer: String,
}

/// Review Card DTO - for submitting a review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCardRequest {
    pub grade: i32,
}

/// Review response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDto {
    pub id: Uuid,
    pub card_id: Uuid,
    pub grade: i32,
}
