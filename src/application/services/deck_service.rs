use crate::{
    domain::{entities::Deck, repositories::DeckRepository},
    AppResult,
};
use std::sync::Arc;
use uuid::Uuid;

use super::super::dtos::{CreateDeckRequest, DeckDto};

/// Deck service - handles deck operations
pub struct DeckService {
    deck_repo: Arc<dyn DeckRepository>,
}

impl DeckService {
    pub fn new(deck_repo: Arc<dyn DeckRepository>) -> Self {
        Self { deck_repo }
    }

    pub async fn create_deck(&self, user_id: Uuid, req: CreateDeckRequest) -> AppResult<DeckDto> {
        let deck = Deck::new(user_id, req.name, req.description);
        let deck_id = self.deck_repo.create(&deck).await?;

        Ok(DeckDto {
            id: deck_id,
            user_id: deck.user_id,
            name: deck.name,
            description: deck.description,
            created_at: deck.created_at,
            updated_at: deck.updated_at,
        })
    }

    pub async fn get_user_decks(&self, user_id: Uuid) -> AppResult<Vec<DeckDto>> {
        let decks = self.deck_repo.find_by_user(user_id).await?;

        Ok(decks
            .into_iter()
            .map(|deck| DeckDto {
                id: deck.id,
                user_id: deck.user_id,
                name: deck.name,
                description: deck.description,
                created_at: deck.created_at,
                updated_at: deck.updated_at,
            })
            .collect())
    }

    pub async fn delete_deck(&self, deck_id: Uuid, user_id: Uuid) -> AppResult<()> {
        let deck = self.deck_repo.find_by_id(deck_id).await?
            .ok_or_else(|| crate::AppError::NotFound(format!("Deck with id {} not found", deck_id)))?;

        if deck.user_id != user_id {
            return Err(crate::AppError::AuthorizationError(
                "Cannot delete deck belonging to another user".to_string(),
            ));
        }

        self.deck_repo.delete(deck_id).await
    }
}
