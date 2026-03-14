use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Review entity - represents a review attempt on a card using FSRS
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Review {
    pub id: Uuid,
    pub card_id: Uuid,
    pub user_id: Uuid,
    pub grade: i32,
    pub created_at: DateTime<Utc>,
}

impl Review {
    pub fn new(card_id: Uuid, user_id: Uuid, grade: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            card_id,
            user_id,
            grade,
            created_at: Utc::now(),
        }
    }
}
