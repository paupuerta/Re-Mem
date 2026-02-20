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

/// FSRS State - represents the spaced repetition state of a card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsrsState {
    pub stability: f32,
    pub difficulty: f32,
    pub elapsed_days: i32,
    pub scheduled_days: i32,
    pub reps: i32,
    pub lapses: i32,
    pub state: CardState,
    pub last_review: Option<DateTime<Utc>>,
}

impl Default for FsrsState {
    fn default() -> Self {
        Self {
            stability: 0.0,
            difficulty: 0.0,
            elapsed_days: 0,
            scheduled_days: 0,
            reps: 0,
            lapses: 0,
            state: CardState::New,
            last_review: None,
        }
    }
}

/// Card State according to FSRS algorithm
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "card_state", rename_all = "lowercase")]
pub enum CardState {
    New,
    Learning,
    Review,
    Relearning,
}

/// Card entity - represents a flashcard for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub user_id: Uuid,
    pub question: String,
    pub answer: String,
    pub fsrs_state: FsrsState,
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
            fsrs_state: FsrsState::default(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Review Log - represents a single review attempt with AI validation
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewLog {
    pub id: Uuid,
    pub card_id: Uuid,
    pub user_id: Uuid,
    pub user_answer: String,
    pub ai_score: Option<f32>,
    pub fsrs_rating: i32,
    pub validation_method: String, // "exact", "embedding", "llm"
    pub created_at: DateTime<Utc>,
}

impl ReviewLog {
    pub fn new(
        card_id: Uuid,
        user_id: Uuid,
        user_answer: String,
        ai_score: Option<f32>,
        fsrs_rating: i32,
        validation_method: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            card_id,
            user_id,
            user_answer,
            ai_score,
            fsrs_rating,
            validation_method,
            created_at: Utc::now(),
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
