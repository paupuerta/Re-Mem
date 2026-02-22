use std::sync::Arc;
use uuid::Uuid;

use crate::domain::repositories::CardRepository;
use crate::AppResult;

/// Use case for deleting a card
pub struct DeleteCardUseCase {
    card_repository: Arc<dyn CardRepository>,
}

impl DeleteCardUseCase {
    pub fn new(card_repository: Arc<dyn CardRepository>) -> Self {
        Self { card_repository }
    }

    pub async fn execute(&self, card_id: Uuid, user_id: Uuid) -> AppResult<()> {
        // Verify card exists and belongs to user
        let card = self.card_repository.find_by_id(card_id).await?;
        
        match card {
            Some(c) if c.user_id == user_id => {
                self.card_repository.delete(card_id).await?;
                Ok(())
            }
            Some(_) => Err(crate::AppError::AuthorizationError(
                "Cannot delete card belonging to another user".to_string()
            )),
            None => Err(crate::AppError::NotFound(
                format!("Card with id {} not found", card_id)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Card;
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockCardRepository {
        cards: Mutex<Vec<Card>>,
    }

    impl MockCardRepository {
        fn new() -> Self {
            Self {
                cards: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl CardRepository for MockCardRepository {
        async fn create(&self, card: &Card) -> AppResult<Uuid> {
            self.cards.lock().unwrap().push(card.clone());
            Ok(card.id)
        }

        async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Card>> {
            Ok(self.cards.lock().unwrap().iter().find(|c| c.id == id).cloned())
        }

        async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Card>> {
            Ok(self
                .cards
                .lock()
                .unwrap()
                .iter()
                .filter(|c| c.user_id == user_id)
                .cloned()
                .collect())
        }

        async fn find_by_deck(&self, deck_id: Uuid) -> AppResult<Vec<Card>> {
            Ok(self
                .cards
                .lock()
                .unwrap()
                .iter()
                .filter(|c| c.deck_id == Some(deck_id))
                .cloned()
                .collect())
        }

        async fn update(&self, card: &Card) -> AppResult<()> {
            let mut cards = self.cards.lock().unwrap();
            if let Some(c) = cards.iter_mut().find(|c| c.id == card.id) {
                *c = card.clone();
            }
            Ok(())
        }

        async fn delete(&self, id: Uuid) -> AppResult<()> {
            self.cards.lock().unwrap().retain(|c| c.id != id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_delete_card_success() {
        let user_id = Uuid::new_v4();
        let card = Card::new(
            user_id,
            "Question".to_string(),
            "Answer".to_string(),
        );
        let card_id = card.id;

        let repo = Arc::new(MockCardRepository::new());
        repo.create(&card).await.unwrap();

        let use_case = DeleteCardUseCase::new(repo.clone());
        let result = use_case.execute(card_id, user_id).await;

        assert!(result.is_ok());
        
        // Verify card was deleted
        let found = repo.find_by_id(card_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_card_not_found() {
        let user_id = Uuid::new_v4();
        let card_id = Uuid::new_v4();
        
        let repo = Arc::new(MockCardRepository::new());
        let use_case = DeleteCardUseCase::new(repo);

        let result = use_case.execute(card_id, user_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_delete_card_wrong_user() {
        let owner_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let card = Card::new(
            owner_id,
            "Question".to_string(),
            "Answer".to_string(),
        );
        let card_id = card.id;

        let repo = Arc::new(MockCardRepository::new());
        repo.create(&card).await.unwrap();

        let use_case = DeleteCardUseCase::new(repo.clone());
        let result = use_case.execute(card_id, other_user_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::AppError::AuthorizationError(_)));
        
        // Verify card was NOT deleted
        let found = repo.find_by_id(card_id).await.unwrap();
        assert!(found.is_some());
    }
}
