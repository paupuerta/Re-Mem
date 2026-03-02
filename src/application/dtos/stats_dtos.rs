use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
