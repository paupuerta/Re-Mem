//! CreateCard use case - create a new flashcard for a user with AI embedding

use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::{
        entities::Card,
        ports::EmbeddingService,
        repositories::CardRepository,
    },
    shared::{error::AppResult, event_bus::{DomainEvent, EventBus}},
};

/// Use case for creating a new card with AI-generated embeddings
pub struct CreateCardUseCase<R, E>
where
    R: CardRepository,
    E: EmbeddingService,
{
    card_repository: Arc<R>,
    embedding_service: Arc<E>,
    event_bus: Arc<EventBus>,
}

impl<R, E> CreateCardUseCase<R, E>
where
    R: CardRepository,
    E: EmbeddingService,
{
    pub fn new(card_repository: Arc<R>, embedding_service: Arc<E>, event_bus: Arc<EventBus>) -> Self {
        Self {
            card_repository,
            embedding_service,
            event_bus,
        }
    }

    /// Execute the use case: create a card and generate its answer embedding
    pub async fn execute(
        &self,
        user_id: Uuid,
        deck_id: Option<Uuid>,
        question: String,
        answer: String,
    ) -> AppResult<Uuid> {
        // Create the card entity
        let mut card = Card::new(user_id, question, answer.clone());

        // Set deck if provided
        if let Some(deck_id) = deck_id {
            card = card.with_deck(deck_id);
        }

        // Generate embedding for the answer
        match self.embedding_service.generate_embedding(&answer).await {
            Ok(embedding) => {
                card = card.with_embedding(embedding);
                tracing::info!("Generated embedding for card answer");
            }
            Err(e) => {
                tracing::warn!("Failed to generate embedding: {}, continuing without it", e);
                // Continue without embedding - it's not critical for card creation
            }
        }

        // Save to repository
        let card_id = self.card_repository.create(&card).await?;

        // Publish CardCreated event
        self.event_bus
            .publish(DomainEvent::CardCreated {
                card_id,
                user_id,
                deck_id,
            })
            .await;

        Ok(card_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockCardRepository {
        card_id: Uuid,
    }

    #[async_trait]
    impl CardRepository for MockCardRepository {
        async fn create(&self, _card: &Card) -> AppResult<Uuid> {
            Ok(self.card_id)
        }

        async fn find_by_id(&self, _id: Uuid) -> AppResult<Option<Card>> {
            Ok(None)
        }

        async fn find_by_user(&self, _user_id: Uuid) -> AppResult<Vec<Card>> {
            Ok(vec![])
        }

        async fn find_by_deck(&self, _deck_id: Uuid) -> AppResult<Vec<Card>> {
            Ok(vec![])
        }

        async fn bulk_create(&self, cards: &[Card]) -> AppResult<Vec<Uuid>> {
            Ok(cards.iter().map(|_| Uuid::new_v4()).collect())
        }

        async fn update(&self, _card: &Card) -> AppResult<()> {
            Ok(())
        }

        async fn update_embedding(&self, _id: Uuid, _embedding: Vec<f32>) -> AppResult<()> {
            Ok(())
        }

        async fn delete(&self, _id: Uuid) -> AppResult<()> {
            Ok(())
        }
    }

    struct MockEmbeddingService {
        should_succeed: bool,
    }

    #[async_trait]
    impl EmbeddingService for MockEmbeddingService {
        async fn generate_embedding(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
            if self.should_succeed {
                Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5])
            } else {
                Err(anyhow::anyhow!("Embedding service failed"))
            }
        }
    }

    #[tokio::test]
    async fn test_create_card_with_embedding_success() {
        let expected_card_id = Uuid::new_v4();
        let card_repo = Arc::new(MockCardRepository { card_id: expected_card_id });
        let embedding_service = Arc::new(MockEmbeddingService { should_succeed: true });
        let event_bus = Arc::new(EventBus::new());
        let use_case = CreateCardUseCase::new(card_repo, embedding_service, event_bus);

        let user_id = Uuid::new_v4();
        let deck_id = Some(Uuid::new_v4());
        let question = "What is the Spanish word for 'hello'?".to_string();
        let answer = "hola".to_string();

        let result = use_case.execute(user_id, deck_id, question, answer).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_card_id);
    }

    #[tokio::test]
    async fn test_create_card_without_deck() {
        let expected_card_id = Uuid::new_v4();
        let card_repo = Arc::new(MockCardRepository { card_id: expected_card_id });
        let embedding_service = Arc::new(MockEmbeddingService { should_succeed: true });
        let event_bus = Arc::new(EventBus::new());
        let use_case = CreateCardUseCase::new(card_repo, embedding_service, event_bus);

        let user_id = Uuid::new_v4();
        let question = "What is 2 + 2?".to_string();
        let answer = "4".to_string();

        let result = use_case.execute(user_id, None, question, answer).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_card_id);
    }

    #[tokio::test]
    async fn test_create_card_embedding_failure_continues() {
        let expected_card_id = Uuid::new_v4();
        let card_repo = Arc::new(MockCardRepository { card_id: expected_card_id });
        let embedding_service = Arc::new(MockEmbeddingService { should_succeed: false });
        let event_bus = Arc::new(EventBus::new());
        let use_case = CreateCardUseCase::new(card_repo, embedding_service, event_bus);

        let user_id = Uuid::new_v4();
        let question = "Test question".to_string();
        let answer = "Test answer".to_string();

        // Should succeed even if embedding generation fails
        let result = use_case.execute(user_id, None, question, answer).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_card_id);
    }
}
