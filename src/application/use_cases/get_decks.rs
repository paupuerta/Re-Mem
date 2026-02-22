//! GetDecks use case - retrieve all decks for a user

use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::{
        entities::Deck,
        repositories::DeckRepository,
    },
    shared::error::AppResult,
};

/// Use case for getting all decks for a user
pub struct GetDecksUseCase<R>
where
    R: DeckRepository,
{
    deck_repository: Arc<R>,
}

impl<R> GetDecksUseCase<R>
where
    R: DeckRepository,
{
    pub fn new(deck_repository: Arc<R>) -> Self {
        Self { deck_repository }
    }

    /// Execute the use case: get all decks for a user
    pub async fn execute(&self, user_id: Uuid) -> AppResult<Vec<Deck>> {
        self.deck_repository.find_by_user(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockDeckRepository {
        decks: Vec<Deck>,
    }

    #[async_trait]
    impl DeckRepository for MockDeckRepository {
        async fn create(&self, _deck: &Deck) -> AppResult<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn find_by_id(&self, _id: Uuid) -> AppResult<Option<Deck>> {
            Ok(None)
        }

        async fn find_by_user(&self, _user_id: Uuid) -> AppResult<Vec<Deck>> {
            Ok(self.decks.clone())
        }

        async fn update(&self, _deck: &Deck) -> AppResult<()> {
            Ok(())
        }

        async fn delete(&self, _id: Uuid) -> AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_get_decks_returns_user_decks() {
        let user_id = Uuid::new_v4();
        let deck1 = Deck::new(user_id, "Spanish".to_string(), Some("Vocab".to_string()));
        let deck2 = Deck::new(user_id, "French".to_string(), None);
        
        let repo = Arc::new(MockDeckRepository {
            decks: vec![deck1.clone(), deck2.clone()],
        });
        let use_case = GetDecksUseCase::new(repo);

        let result = use_case.execute(user_id).await;

        assert!(result.is_ok());
        let decks = result.unwrap();
        assert_eq!(decks.len(), 2);
        assert_eq!(decks[0].name, "Spanish");
        assert_eq!(decks[1].name, "French");
    }

    #[tokio::test]
    async fn test_get_decks_returns_empty_list_when_no_decks() {
        let user_id = Uuid::new_v4();
        let repo = Arc::new(MockDeckRepository { decks: vec![] });
        let use_case = GetDecksUseCase::new(repo);

        let result = use_case.execute(user_id).await;

        assert!(result.is_ok());
        let decks = result.unwrap();
        assert_eq!(decks.len(), 0);
    }
}
