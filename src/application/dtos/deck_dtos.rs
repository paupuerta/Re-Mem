use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
