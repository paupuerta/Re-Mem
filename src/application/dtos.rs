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
    pub deck_id: Option<Uuid>,
    pub question: String,
    pub answer: String,
}

/// Card response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub deck_id: Option<Uuid>,
    pub question: String,
    pub answer: String,
    pub fsrs_state: FsrsState,
}

/// Create Deck DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeckRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Deck response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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

/// User statistics response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatsDto {
    pub user_id: Uuid,
    pub total_reviews: i32,
    pub correct_reviews: i32,
    pub days_studied: i32,
    pub accuracy_percentage: f64,
    pub last_active_date: Option<String>, // ISO 8601 date string
}

/// Deck statistics response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckStatsDto {
    pub deck_id: Uuid,
    pub deck_name: String,
    pub total_cards: i32,
    pub total_reviews: i32,
    pub correct_reviews: i32,
    pub days_studied: i32,
    pub accuracy_percentage: f64,
    pub last_active_date: Option<String>, // ISO 8601 date string
}


/// Auth: Register request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

/// Auth: Login request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Auth: Response DTO (register + login)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserDto,
}

/// Import result DTO — returned after TSV or Anki import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub cards_imported: u32,
    pub cards_skipped: u32,
}

/// Anki import result DTO — returned after .apkg import (includes created deck info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnkiImportResult {
    pub deck_id: Uuid,
    pub deck_name: String,
    pub cards_imported: u32,
    pub cards_skipped: u32,
}
