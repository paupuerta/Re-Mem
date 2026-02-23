use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::dtos::DeckStatsDto,
    domain::repositories::{DeckRepository, DeckStatsRepository},
    AppResult,
};

/// Use case for retrieving deck statistics
pub struct GetDeckStatsUseCase {
    deck_stats_repository: Arc<dyn DeckStatsRepository>,
    deck_repository: Arc<dyn DeckRepository>,
}

impl GetDeckStatsUseCase {
    pub fn new(
        deck_stats_repository: Arc<dyn DeckStatsRepository>,
        deck_repository: Arc<dyn DeckRepository>,
    ) -> Self {
        Self {
            deck_stats_repository,
            deck_repository,
        }
    }

    pub async fn execute(&self, deck_id: Uuid) -> AppResult<DeckStatsDto> {
        // Get the deck to get its name
        let deck = self
            .deck_repository
            .find_by_id(deck_id)
            .await?
            .ok_or_else(|| crate::AppError::NotFound(format!("Deck with id {} not found", deck_id)))?;

        // Get or create stats for this deck
        let stats = self
            .deck_stats_repository
            .get_or_create(deck_id, deck.user_id)
            .await?;

        Ok(DeckStatsDto {
            deck_id: stats.deck_id,
            deck_name: deck.name,
            total_cards: stats.total_cards,
            total_reviews: stats.total_reviews,
            correct_reviews: stats.correct_reviews,
            days_studied: stats.days_studied,
            accuracy_percentage: stats.accuracy_percentage(),
            last_active_date: stats.last_active_date.map(|d| d.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Deck, DeckStats};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockDeckRepository {
        deck: Mutex<Option<Deck>>,
    }

    impl MockDeckRepository {
        fn with_deck(deck: Deck) -> Self {
            Self {
                deck: Mutex::new(Some(deck)),
            }
        }
    }

    #[async_trait]
    impl DeckRepository for MockDeckRepository {
        async fn create(&self, _deck: &Deck) -> AppResult<Uuid> {
            unimplemented!()
        }

        async fn find_by_id(&self, _id: Uuid) -> AppResult<Option<Deck>> {
            Ok(self.deck.lock().unwrap().clone())
        }

        async fn find_by_user(&self, _user_id: Uuid) -> AppResult<Vec<Deck>> {
            unimplemented!()
        }

        async fn update(&self, _deck: &Deck) -> AppResult<()> {
            unimplemented!()
        }

        async fn delete(&self, _id: Uuid) -> AppResult<()> {
            unimplemented!()
        }
    }

    struct MockDeckStatsRepository {
        stats: Mutex<Option<DeckStats>>,
    }

    impl MockDeckStatsRepository {
        fn with_stats(stats: DeckStats) -> Self {
            Self {
                stats: Mutex::new(Some(stats)),
            }
        }
    }

    #[async_trait]
    impl DeckStatsRepository for MockDeckStatsRepository {
        async fn get_or_create(&self, deck_id: Uuid, user_id: Uuid) -> AppResult<DeckStats> {
            let stats = self.stats.lock().unwrap();
            Ok(stats
                .clone()
                .unwrap_or_else(|| DeckStats::new(deck_id, user_id)))
        }

        async fn update_after_review(
            &self,
            _deck_id: Uuid,
            _is_correct: bool,
            _review_date: chrono::NaiveDate,
        ) -> AppResult<()> {
            Ok(())
        }

        async fn increment_card_count(&self, _deck_id: Uuid) -> AppResult<()> {
            Ok(())
        }

        async fn decrement_card_count(&self, _deck_id: Uuid) -> AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_get_deck_stats_success() {
        let user_id = Uuid::new_v4();
        let deck_id = Uuid::new_v4();
        
        let deck = Deck::new(user_id, "Spanish Vocabulary".to_string(), None);
        let mut stats = DeckStats::new(deck_id, user_id);
        stats.total_cards = 50;
        stats.total_reviews = 200;
        stats.correct_reviews = 160;
        stats.days_studied = 10;

        let deck_repo = Arc::new(MockDeckRepository::with_deck(deck));
        let stats_repo = Arc::new(MockDeckStatsRepository::with_stats(stats));
        
        let use_case = GetDeckStatsUseCase::new(stats_repo, deck_repo);
        let result = use_case.execute(deck_id).await.unwrap();

        assert_eq!(result.deck_name, "Spanish Vocabulary");
        assert_eq!(result.total_cards, 50);
        assert_eq!(result.total_reviews, 200);
        assert_eq!(result.correct_reviews, 160);
        assert_eq!(result.days_studied, 10);
        assert_eq!(result.accuracy_percentage, 80.0);
    }

    #[tokio::test]
    async fn test_get_deck_stats_deck_not_found() {
        let deck_id = Uuid::new_v4();
        
        let deck_repo = Arc::new(MockDeckRepository { deck: Mutex::new(None) });
        let stats_repo = Arc::new(MockDeckStatsRepository {
            stats: Mutex::new(None),
        });
        
        let use_case = GetDeckStatsUseCase::new(stats_repo, deck_repo);
        let result = use_case.execute(deck_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::AppError::NotFound(_)));
    }
}
