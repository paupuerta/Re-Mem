//! Tests for deck and card repository functionality

#[cfg(test)]
mod deck_repository_tests {
    use re_mem::domain::{entities::Deck, repositories::DeckRepository};
    use async_trait::async_trait;
    use uuid::Uuid;

    struct MockDeckRepo {
        should_fail: bool,
    }

    #[async_trait]
    impl DeckRepository for MockDeckRepo {
        async fn create(&self, deck: &Deck) -> re_mem::AppResult<Uuid> {
            if self.should_fail {
                Err(re_mem::AppError::InternalError("Test error".to_string()))
            } else {
                Ok(deck.id)
            }
        }

        async fn find_by_id(&self, id: Uuid) -> re_mem::AppResult<Option<Deck>> {
            if self.should_fail {
                Err(re_mem::AppError::NotFound("Deck not found".to_string()))
            } else {
                let deck = Deck::new(Uuid::new_v4(), "Test".to_string(), None);
                Ok(Some(deck))
            }
        }

        async fn find_by_user(&self, user_id: Uuid) -> re_mem::AppResult<Vec<Deck>> {
            if self.should_fail {
                return Ok(vec![]);
            }
            Ok(vec![
                Deck::new(user_id, "Deck 1".to_string(), Some("Description 1".to_string())),
                Deck::new(user_id, "Deck 2".to_string(), None),
            ])
        }

        async fn update(&self, _deck: &Deck) -> re_mem::AppResult<()> {
            if self.should_fail {
                Err(re_mem::AppError::NotFound("Deck not found".to_string()))
            } else {
                Ok(())
            }
        }

        async fn delete(&self, _id: Uuid) -> re_mem::AppResult<()> {
            if self.should_fail {
                Err(re_mem::AppError::NotFound("Deck not found".to_string()))
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_create_deck_success() {
        let repo = MockDeckRepo { should_fail: false };
        let user_id = Uuid::new_v4();
        let deck = Deck::new(user_id, "Spanish".to_string(), Some("Vocabulary".to_string()));

        let result = repo.create(&deck).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_deck_failure() {
        let repo = MockDeckRepo { should_fail: true };
        let user_id = Uuid::new_v4();
        let deck = Deck::new(user_id, "Spanish".to_string(), None);

        let result = repo.create(&deck).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_by_user_returns_decks() {
        let repo = MockDeckRepo { should_fail: false };
        let user_id = Uuid::new_v4();

        let result = repo.find_by_user(user_id).await;
        assert!(result.is_ok());
        let decks = result.unwrap();
        assert_eq!(decks.len(), 2);
        assert_eq!(decks[0].name, "Deck 1");
        assert_eq!(decks[1].name, "Deck 2");
    }

    #[tokio::test]
    async fn test_find_by_user_empty() {
        let repo = MockDeckRepo { should_fail: true };
        let user_id = Uuid::new_v4();

        let result = repo.find_by_user(user_id).await;
        assert!(result.is_ok());
        let decks = result.unwrap();
        assert_eq!(decks.len(), 0);
    }
}

#[cfg(test)]
mod card_repository_tests {
    use re_mem::domain::{entities::Card, repositories::CardRepository};
    use async_trait::async_trait;
    use uuid::Uuid;

    struct MockCardRepo {
        cards: Vec<Card>,
    }

    #[async_trait]
    impl CardRepository for MockCardRepo {
        async fn create(&self, card: &Card) -> re_mem::AppResult<Uuid> {
            Ok(card.id)
        }

        async fn find_by_id(&self, _id: Uuid) -> re_mem::AppResult<Option<Card>> {
            Ok(self.cards.first().cloned())
        }

        async fn find_by_user(&self, _user_id: Uuid) -> re_mem::AppResult<Vec<Card>> {
            Ok(self.cards.clone())
        }

        async fn find_by_deck(&self, deck_id: Uuid) -> re_mem::AppResult<Vec<Card>> {
            Ok(self.cards.iter().filter(|c| c.deck_id == Some(deck_id)).cloned().collect())
        }

        async fn update(&self, _card: &Card) -> re_mem::AppResult<()> {
            Ok(())
        }

        async fn delete(&self, _id: Uuid) -> re_mem::AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_find_by_deck_returns_only_deck_cards() {
        let user_id = Uuid::new_v4();
        let deck_id = Uuid::new_v4();
        let other_deck_id = Uuid::new_v4();

        let card1 = Card::new(user_id, "Q1".to_string(), "A1".to_string()).with_deck(deck_id);
        let card2 = Card::new(user_id, "Q2".to_string(), "A2".to_string()).with_deck(deck_id);
        let card3 = Card::new(user_id, "Q3".to_string(), "A3".to_string()).with_deck(other_deck_id);
        let card4 = Card::new(user_id, "Q4".to_string(), "A4".to_string()); // No deck

        let repo = MockCardRepo {
            cards: vec![card1, card2, card3, card4],
        };

        let result = repo.find_by_deck(deck_id).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert_eq!(cards.len(), 2);
        assert!(cards.iter().all(|c| c.deck_id == Some(deck_id)));
    }

    #[tokio::test]
    async fn test_find_by_deck_returns_empty_when_no_cards() {
        let deck_id = Uuid::new_v4();
        let repo = MockCardRepo { cards: vec![] };

        let result = repo.find_by_deck(deck_id).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_card_with_embedding() {
        let user_id = Uuid::new_v4();
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        
        let card = Card::new(user_id, "Test".to_string(), "Answer".to_string())
            .with_embedding(embedding.clone());

        assert_eq!(card.answer_embedding, Some(embedding));
    }
}
