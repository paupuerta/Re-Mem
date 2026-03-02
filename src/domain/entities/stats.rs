use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User-level statistics - precalculated for performance
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserStats {
    pub user_id: Uuid,
    pub total_reviews: i32,
    pub correct_reviews: i32,
    pub days_studied: i32,
    pub last_active_date: Option<chrono::NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserStats {
    pub fn new(user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            total_reviews: 0,
            correct_reviews: 0,
            days_studied: 0,
            last_active_date: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate accuracy percentage (0-100)
    pub fn accuracy_percentage(&self) -> f64 {
        if self.total_reviews == 0 {
            0.0
        } else {
            (self.correct_reviews as f64 / self.total_reviews as f64) * 100.0
        }
    }
}

/// Deck-level statistics - precalculated for performance
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeckStats {
    pub deck_id: Uuid,
    pub user_id: Uuid,
    pub total_cards: i32,
    pub total_reviews: i32,
    pub correct_reviews: i32,
    pub days_studied: i32,
    pub last_active_date: Option<chrono::NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DeckStats {
    pub fn new(deck_id: Uuid, user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            deck_id,
            user_id,
            total_cards: 0,
            total_reviews: 0,
            correct_reviews: 0,
            days_studied: 0,
            last_active_date: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate accuracy percentage (0-100)
    pub fn accuracy_percentage(&self) -> f64 {
        if self.total_reviews == 0 {
            0.0
        } else {
            (self.correct_reviews as f64 / self.total_reviews as f64) * 100.0
        }
    }
}
