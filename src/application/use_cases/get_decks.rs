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
