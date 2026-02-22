//! CreateDeck use case - create a new deck for organizing cards

use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::{
        entities::Deck,
        repositories::DeckRepository,
    },
    shared::error::AppResult,
};

/// Use case for creating a new deck
pub struct CreateDeckUseCase<R>
where
    R: DeckRepository,
{
    deck_repository: Arc<R>,
}

impl<R> CreateDeckUseCase<R>
where
    R: DeckRepository,
{
    pub fn new(deck_repository: Arc<R>) -> Self {
        Self { deck_repository }
    }

    /// Execute the use case: create a new deck
    pub async fn execute(
        &self,
        user_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> AppResult<Uuid> {
        let deck = Deck::new(user_id, name, description);
        let deck_id = self.deck_repository.create(&deck).await?;
        Ok(deck_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockDeckRepository {
        deck_id: Uuid,
    }

    #[async_trait]
    impl DeckRepository for MockDeckRepository {
        async fn create(&self, _deck: &Deck) -> AppResult<Uuid> {
            Ok(self.deck_id)
        }

        async fn find_by_id(&self, _id: Uuid) -> AppResult<Option<Deck>> {
            Ok(None)
        }

        async fn find_by_user(&self, _user_id: Uuid) -> AppResult<Vec<Deck>> {
            Ok(vec![])
        }

        async fn update(&self, _deck: &Deck) -> AppResult<()> {
            Ok(())
        }

        async fn delete(&self, _id: Uuid) -> AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_deck_success() {
        let expected_deck_id = Uuid::new_v4();
        let repo = Arc::new(MockDeckRepository { deck_id: expected_deck_id });
        let use_case = CreateDeckUseCase::new(repo);

        let user_id = Uuid::new_v4();
        let name = "Spanish Vocabulary".to_string();
        let description = Some("Basic Spanish words and phrases".to_string());

        let result = use_case.execute(user_id, name, description).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_deck_id);
    }

    #[tokio::test]
    async fn test_create_deck_without_description() {
        let expected_deck_id = Uuid::new_v4();
        let repo = Arc::new(MockDeckRepository { deck_id: expected_deck_id });
        let use_case = CreateDeckUseCase::new(repo);

        let user_id = Uuid::new_v4();
        let name = "French Verbs".to_string();

        let result = use_case.execute(user_id, name, None).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_deck_id);
    }
}
