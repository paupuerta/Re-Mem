use std::sync::Arc;
use uuid::Uuid;

use crate::domain::repositories::DeckRepository;
use crate::AppResult;

/// Use case for deleting a deck
/// Note: Cards in the deck will have their deck_id set to NULL (ON DELETE SET NULL)
pub struct DeleteDeckUseCase {
    deck_repository: Arc<dyn DeckRepository>,
}

impl DeleteDeckUseCase {
    pub fn new(deck_repository: Arc<dyn DeckRepository>) -> Self {
        Self { deck_repository }
    }

    pub async fn execute(&self, deck_id: Uuid, user_id: Uuid) -> AppResult<()> {
        // Verify deck exists and belongs to user
        let deck = self.deck_repository.find_by_id(deck_id).await?;
        
        match deck {
            Some(d) if d.user_id == user_id => {
                self.deck_repository.delete(deck_id).await?;
                Ok(())
            }
            Some(_) => Err(crate::AppError::AuthorizationError(
                "Cannot delete deck belonging to another user".to_string()
            )),
            None => Err(crate::AppError::NotFound(
                format!("Deck with id {} not found", deck_id)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Deck;
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockDeckRepository {
        decks: Mutex<Vec<Deck>>,
    }

    impl MockDeckRepository {
        fn new() -> Self {
            Self {
                decks: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl DeckRepository for MockDeckRepository {
        async fn create(&self, deck: &Deck) -> AppResult<Uuid> {
            self.decks.lock().unwrap().push(deck.clone());
            Ok(deck.id)
        }

        async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Deck>> {
            Ok(self.decks.lock().unwrap().iter().find(|d| d.id == id).cloned())
        }

        async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Deck>> {
            Ok(self
                .decks
                .lock()
                .unwrap()
                .iter()
                .filter(|d| d.user_id == user_id)
                .cloned()
                .collect())
        }

        async fn update(&self, deck: &Deck) -> AppResult<()> {
            let mut decks = self.decks.lock().unwrap();
            if let Some(d) = decks.iter_mut().find(|d| d.id == deck.id) {
                *d = deck.clone();
            }
            Ok(())
        }

        async fn delete(&self, id: Uuid) -> AppResult<()> {
            self.decks.lock().unwrap().retain(|d| d.id != id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_delete_deck_success() {
        let user_id = Uuid::new_v4();
        let deck = Deck::new(user_id, "Test Deck".to_string(), Some("Description".to_string()));
        let deck_id = deck.id;

        let repo = Arc::new(MockDeckRepository::new());
        repo.create(&deck).await.unwrap();

        let use_case = DeleteDeckUseCase::new(repo.clone());
        let result = use_case.execute(deck_id, user_id).await;

        assert!(result.is_ok());
        
        // Verify deck was deleted
        let found = repo.find_by_id(deck_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_deck_not_found() {
        let user_id = Uuid::new_v4();
        let deck_id = Uuid::new_v4();
        
        let repo = Arc::new(MockDeckRepository::new());
        let use_case = DeleteDeckUseCase::new(repo);

        let result = use_case.execute(deck_id, user_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_delete_deck_wrong_user() {
        let owner_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let deck = Deck::new(owner_id, "Test Deck".to_string(), None);
        let deck_id = deck.id;

        let repo = Arc::new(MockDeckRepository::new());
        repo.create(&deck).await.unwrap();

        let use_case = DeleteDeckUseCase::new(repo.clone());
        let result = use_case.execute(deck_id, other_user_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::AppError::AuthorizationError(_)));
        
        // Verify deck was NOT deleted
        let found = repo.find_by_id(deck_id).await.unwrap();
        assert!(found.is_some());
    }
}
