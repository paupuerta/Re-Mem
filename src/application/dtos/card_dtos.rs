use crate::domain::entities::FsrsState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
