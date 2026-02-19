use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User entity - represents a learner/user in the system
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            name,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Card entity - represents a flashcard for learning
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Card {
    pub id: Uuid,
    pub user_id: Uuid,
    pub question: String,
    pub answer: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Card {
    pub fn new(user_id: Uuid, question: String, answer: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            question,
            answer,
            created_at: now,
            updated_at: now,
        }
    }
}

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
