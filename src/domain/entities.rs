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

/// Deck entity - represents a collection of cards
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Deck {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Deck {
    pub fn new(user_id: Uuid, name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            name,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

/// FSRS (Free Spaced Repetition Scheduler) state for a card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsrsState {
    /// Stability - represents how well the memory is retained
    pub stability: f32,
    /// Difficulty - intrinsic difficulty of the card
    pub difficulty: f32,
    /// Days elapsed since last review
    pub elapsed_days: i32,
    /// Days scheduled until next review
    pub scheduled_days: i32,
    /// Number of times the card has been reviewed
    pub reps: i32,
    /// Number of times the card was forgotten (lapsed)
    pub lapses: i32,
    /// Current state of the card in the learning process
    pub state: CardState,
    /// Timestamp of last review
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
#[serde(rename_all = "lowercase")]
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
    pub deck_id: Option<Uuid>,
    pub question: String,
    pub answer: String,
    pub answer_embedding: Option<Vec<f32>>,
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
            deck_id: None,
            question,
            answer,
            answer_embedding: None,
            fsrs_state: FsrsState::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_deck(mut self, deck_id: Uuid) -> Self {
        self.deck_id = Some(deck_id);
        self
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.answer_embedding = Some(embedding);
        self
    }
}

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
