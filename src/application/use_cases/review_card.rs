//! ReviewCard use case - AI-powered flashcard review with FSRS scheduling

use anyhow::{Context, Result};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    entities::{CardState, FsrsState, ReviewLog},
    ports::{AIValidator, ValidationMethod},
    repositories::{CardRepository, ReviewLogRepository},
};
use crate::shared::event_bus::{DomainEvent, EventBus};

/// Use case for reviewing a card with AI-powered validation
pub struct ReviewCardUseCase<R: CardRepository, L: ReviewLogRepository, V: AIValidator> {
    card_repository: Arc<R>,
    review_log_repository: Arc<L>,
    ai_validator: Arc<V>,
    event_bus: Arc<EventBus>,
}

impl<R: CardRepository, L: ReviewLogRepository, V: AIValidator> ReviewCardUseCase<R, L, V> {
    pub fn new(
        card_repository: Arc<R>,
        review_log_repository: Arc<L>,
        ai_validator: Arc<V>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            card_repository,
            review_log_repository,
            ai_validator,
            event_bus,
        }
    }

    /// Execute the review card use case
    pub async fn execute(
        &self,
        card_id: Uuid,
        user_id: Uuid,
        user_answer: String,
    ) -> Result<ReviewResult> {
        // 1. Get the card
        let mut card = self
            .card_repository
            .find_by_id(card_id)
            .await?
            .context("Card not found")?;

        // 2. Validate the answer using AI
        let validation = self
            .ai_validator
            .validate(&card.answer, &user_answer, &card.question)
            .await?;

        // 3. Convert AI score to FSRS rating (1-4)
        let fsrs_rating = score_to_fsrs_rating(validation.score);

        // 4. Update FSRS state
        card.fsrs_state = update_fsrs_state(&card.fsrs_state, fsrs_rating);
        card.updated_at = Utc::now();

        // 5. Save updated card
        self.card_repository.update(&card).await?;

        // 6. Create review log
        let review_log = ReviewLog::new(
            card_id,
            user_id,
            user_answer.clone(),
            card.answer.clone(),
            validation.score,
            validation.method.as_str().to_string(),
            fsrs_rating,
        );
        self.review_log_repository.create(&review_log).await?;

        // 7. Emit domain event
        self.event_bus
            .publish(DomainEvent::CardReviewed {
                card_id,
                user_id,
                score: validation.score,
                rating: fsrs_rating,
            })
            .await;

        Ok(ReviewResult {
            card_id,
            ai_score: validation.score,
            fsrs_rating,
            validation_method: validation.method,
            next_review_in_days: card.fsrs_state.scheduled_days,
        })
    }
}

/// Result of a card review
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub card_id: Uuid,
    pub ai_score: f32,
    pub fsrs_rating: i32,
    pub validation_method: ValidationMethod,
    pub next_review_in_days: i32,
}

/// Convert AI score (0.0-1.0) to FSRS rating (1-4)
fn score_to_fsrs_rating(score: f32) -> i32 {
    match score {
        s if s >= 0.9 => 4, // Easy
        s if s >= 0.7 => 3, // Good
        s if s >= 0.5 => 2, // Hard
        _ => 1,             // Again
    }
}

/// Update FSRS state based on rating
fn update_fsrs_state(current: &FsrsState, rating: i32) -> FsrsState {
    let mut next = FsrsState {
        stability: current.stability,
        difficulty: current.difficulty,
        elapsed_days: 0,
        scheduled_days: current.scheduled_days,
        reps: current.reps + 1,
        lapses: current.lapses,
        state: current.state.clone(),
        last_review: Some(Utc::now()),
    };

    // Initialize for first review
    if current.reps == 0 {
        next.stability = 1.0;
        next.difficulty = 5.0;
    }

    match rating {
        1 => {
            // Again - reset card to learning
            next.lapses += 1;
            next.stability = (next.stability * 0.5).max(0.1);
            next.difficulty = (next.difficulty + 1.0).min(10.0);
            next.scheduled_days = 1;
            next.state = CardState::Relearning;
        }
        2 => {
            // Hard - slightly increase interval
            next.stability *= 1.2;
            next.difficulty = (next.difficulty + 0.15).min(10.0);
            next.scheduled_days = ((next.stability * 1.2) as i32).max(1);
            next.state = if next.reps <= 1 {
                CardState::Learning
            } else {
                CardState::Review
            };
        }
        3 => {
            // Good - normal progression
            next.stability *= 2.5;
            // difficulty unchanged
            next.scheduled_days = ((next.stability * 2.5) as i32).max(1);
            next.state = if next.reps <= 1 {
                CardState::Learning
            } else {
                CardState::Review
            };
        }
        4 => {
            // Easy - large increase
            next.stability *= 4.0;
            next.difficulty = (next.difficulty - 0.15).max(1.0);
            next.scheduled_days = ((next.stability * 4.0) as i32).max(1);
            next.state = CardState::Review;
        }
        _ => {
            // Default to Good
            next.stability *= 2.5;
            next.scheduled_days = ((next.stability * 2.5) as i32).max(1);
            next.state = CardState::Review;
        }
    }

    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            entities::Card,
            ports::{ValidationMethod, ValidationResult},
        },
        shared::error::AppResult,
    };
    use async_trait::async_trait;
    use std::sync::Arc;

    #[test]
    fn test_score_to_fsrs_rating() {
        assert_eq!(score_to_fsrs_rating(0.95), 4); // Easy
        assert_eq!(score_to_fsrs_rating(0.75), 3); // Good
        assert_eq!(score_to_fsrs_rating(0.55), 2); // Hard
        assert_eq!(score_to_fsrs_rating(0.30), 1); // Again
    }

    #[test]
    fn test_update_fsrs_state_new_card() {
        let mut state = FsrsState::default();

        // First review with Good rating
        state = update_fsrs_state(&state, 3);

        assert_eq!(state.state, CardState::Learning);
        assert_eq!(state.reps, 1);
        assert!(state.stability > 0.0);
        assert!(state.difficulty > 0.0);
    }

    #[test]
    fn test_update_fsrs_state_progression() {
        let mut state = FsrsState::default();

        // First review - Good
        state = update_fsrs_state(&state, 3);
        assert_eq!(state.state, CardState::Learning);
        assert_eq!(state.reps, 1);

        // Second review - Good
        state = update_fsrs_state(&state, 3);
        assert_eq!(state.state, CardState::Review);
        assert_eq!(state.reps, 2);

        // Third review - Easy
        let prev_stability = state.stability;
        state = update_fsrs_state(&state, 4);
        assert!(state.stability > prev_stability);
    }

    #[test]
    fn test_update_fsrs_state_lapses() {
        let mut state = FsrsState::default();

        // Build up some progress
        state = update_fsrs_state(&state, 3);
        state = update_fsrs_state(&state, 3);
        assert_eq!(state.state, CardState::Review);

        // Fail the card
        state = update_fsrs_state(&state, 1);
        assert_eq!(state.state, CardState::Relearning);
        assert_eq!(state.lapses, 1);
    }

    // Mock implementations for testing
    struct MockCardRepository {
        card: Option<Card>,
    }

    #[async_trait]
    impl CardRepository for MockCardRepository {
        async fn create(&self, _card: &Card) -> AppResult<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn find_by_id(&self, _id: Uuid) -> AppResult<Option<Card>> {
            Ok(self.card.clone())
        }

        async fn find_by_user(&self, _user_id: Uuid) -> AppResult<Vec<Card>> {
            Ok(vec![])
        }

        async fn update(&self, _card: &Card) -> AppResult<()> {
            Ok(())
        }

        async fn delete(&self, _id: Uuid) -> AppResult<()> {
            Ok(())
        }
    }

    struct MockReviewLogRepository;

    #[async_trait]
    impl ReviewLogRepository for MockReviewLogRepository {
        async fn create(&self, _log: &ReviewLog) -> AppResult<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn find_by_card(&self, _card_id: Uuid) -> AppResult<Vec<ReviewLog>> {
            Ok(vec![])
        }

        async fn find_by_user(&self, _user_id: Uuid) -> AppResult<Vec<ReviewLog>> {
            Ok(vec![])
        }
    }

    struct MockAIValidator {
        score: f32,
        method: ValidationMethod,
    }

    #[async_trait]
    impl AIValidator for MockAIValidator {
        async fn validate(
            &self,
            _expected: &str,
            _actual: &str,
            _question: &str,
        ) -> anyhow::Result<ValidationResult> {
            Ok(ValidationResult {
                score: self.score,
                method: self.method.clone(),
            })
        }
    }

    #[tokio::test]
    async fn test_review_card_use_case_success() {
        let card_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let card = Card {
            id: card_id,
            user_id,
            question: "What is 2+2?".to_string(),
            answer: "4".to_string(),
            fsrs_state: FsrsState::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let card_repo = Arc::new(MockCardRepository { card: Some(card) });
        let log_repo = Arc::new(MockReviewLogRepository);
        let validator = Arc::new(MockAIValidator {
            score: 0.95,
            method: ValidationMethod::Exact,
        });
        let event_bus = Arc::new(crate::shared::event_bus::EventBus::new());

        let use_case = ReviewCardUseCase::new(card_repo, log_repo, validator, event_bus);

        let result = use_case.execute(card_id, user_id, "4".to_string()).await;

        assert!(result.is_ok());
        let review_result = result.unwrap();
        assert_eq!(review_result.ai_score, 0.95);
        assert!(matches!(
            review_result.validation_method,
            ValidationMethod::Exact
        ));
        assert_eq!(review_result.fsrs_rating, 4); // Easy
    }

    #[tokio::test]
    async fn test_review_card_use_case_card_not_found() {
        let card_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let card_repo = Arc::new(MockCardRepository { card: None });
        let log_repo = Arc::new(MockReviewLogRepository);
        let validator = Arc::new(MockAIValidator {
            score: 0.95,
            method: ValidationMethod::Exact,
        });
        let event_bus = Arc::new(crate::shared::event_bus::EventBus::new());

        let use_case = ReviewCardUseCase::new(card_repo, log_repo, validator, event_bus);

        let result = use_case
            .execute(card_id, user_id, "answer".to_string())
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_review_card_different_scores() {
        let test_cases = vec![
            (0.95, 4, ValidationMethod::Exact),     // Easy
            (0.75, 3, ValidationMethod::Embedding), // Good
            (0.55, 2, ValidationMethod::Llm),       // Hard
            (0.30, 1, ValidationMethod::Llm),       // Again
        ];

        for (score, expected_rating, method) in test_cases {
            let card_id = Uuid::new_v4();
            let user_id = Uuid::new_v4();

            let card = Card {
                id: card_id,
                user_id,
                question: "Test".to_string(),
                answer: "Answer".to_string(),
                fsrs_state: FsrsState::default(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            let card_repo = Arc::new(MockCardRepository { card: Some(card) });
            let log_repo = Arc::new(MockReviewLogRepository);
            let validator = Arc::new(MockAIValidator {
                score,
                method: method.clone(),
            });
            let event_bus = Arc::new(crate::shared::event_bus::EventBus::new());

            let use_case = ReviewCardUseCase::new(card_repo, log_repo, validator, event_bus);

            let result = use_case
                .execute(card_id, user_id, "test answer".to_string())
                .await;

            assert!(result.is_ok());
            let review_result = result.unwrap();
            assert_eq!(review_result.fsrs_rating, expected_rating);
        }
    }
}
