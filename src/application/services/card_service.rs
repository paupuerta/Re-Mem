use crate::{
    domain::{entities::Card, repositories::CardRepository},
    shared::event_bus::{DomainEvent, EventBus},
    AppResult,
};
use std::sync::Arc;
use uuid::Uuid;

use super::super::dtos::{CardDto, CreateCardRequest};

/// Card service - handles card (flashcard) operations
pub struct CardService {
    card_repo: Arc<dyn CardRepository>,
    event_bus: Arc<EventBus>,
}

impl CardService {
    pub fn new(card_repo: Arc<dyn CardRepository>, event_bus: Arc<EventBus>) -> Self {
        Self {
            card_repo,
            event_bus,
        }
    }

    pub async fn create_card(&self, user_id: Uuid, req: CreateCardRequest) -> AppResult<CardDto> {
        let mut card = Card::new(user_id, req.question, req.answer);
        let deck_id = req.deck_id;
        if let Some(deck_id) = deck_id {
            card = card.with_deck(deck_id);
        }
        let card_id = self.card_repo.create(&card).await?;

        self.event_bus
            .publish(DomainEvent::CardCreated {
                card_id,
                user_id,
                deck_id,
            })
            .await;

        Ok(CardDto {
            id: card_id,
            user_id: card.user_id,
            deck_id: card.deck_id,
            question: card.question,
            answer: card.answer,
            fsrs_state: card.fsrs_state,
        })
    }

    pub async fn get_user_cards(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<Vec<CardDto>> {
        let cards = self
            .card_repo
            .find_by_user_paginated(user_id, limit, offset)
            .await?;

        Ok(cards
            .into_iter()
            .map(|card| CardDto {
                id: card.id,
                user_id: card.user_id,
                deck_id: card.deck_id,
                question: card.question,
                answer: card.answer,
                fsrs_state: card.fsrs_state,
            })
            .collect())
    }

    pub async fn get_deck_cards(
        &self,
        deck_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<Vec<CardDto>> {
        let cards = self
            .card_repo
            .find_by_deck_paginated(deck_id, limit, offset)
            .await?;

        Ok(cards
            .into_iter()
            .map(|card| CardDto {
                id: card.id,
                user_id: card.user_id,
                deck_id: card.deck_id,
                question: card.question,
                answer: card.answer,
                fsrs_state: card.fsrs_state,
            })
            .collect())
    }

    pub async fn delete_card(&self, card_id: Uuid, user_id: Uuid) -> AppResult<()> {
        let card = self.card_repo.find_by_id(card_id).await?.ok_or_else(|| {
            crate::AppError::NotFound(format!("Card with id {} not found", card_id))
        })?;

        if card.user_id != user_id {
            return Err(crate::AppError::AuthorizationError(
                "Cannot delete card belonging to another user".to_string(),
            ));
        }

        self.card_repo.delete(card_id).await
    }
}
