use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Review Log - tracks AI validation results for analytics
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewLog {
    pub id: Uuid,
    pub card_id: Uuid,
    pub user_id: Uuid,
    pub user_answer: String,
    pub expected_answer: String,
    pub ai_score: f32,
    pub validation_method: String,
    pub fsrs_rating: i32,
    pub created_at: DateTime<Utc>,
}

impl ReviewLog {
    pub fn new(
        card_id: Uuid,
        user_id: Uuid,
        user_answer: String,
        expected_answer: String,
        ai_score: f32,
        validation_method: String,
        fsrs_rating: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            card_id,
            user_id,
            user_answer,
            expected_answer,
            ai_score,
            validation_method,
            fsrs_rating,
            created_at: Utc::now(),
        }
    }
}
