use crate::domain::entities::FsrsState;
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
    pub fsrs_state: FsrsState,
}

/// Review Card DTO - for submitting a review with user answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCardRequest {
    pub user_answer: String,
}

/// Legacy Review Card DTO - for submitting a review with grade (deprecated)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyReviewCardRequest {
    pub grade: i32,
}

/// Review response DTO with AI validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResponseDto {
    pub card_id: Uuid,
    pub ai_score: f32,
    pub fsrs_rating: i32,
    pub validation_method: String,
    pub next_review_in_days: i32,
}

/// Legacy Review response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDto {
    pub id: Uuid,
    pub card_id: Uuid,
    pub grade: i32,
}
