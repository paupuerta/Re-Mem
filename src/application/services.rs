use crate::{
    domain::{entities::*, repositories::*},
    shared::event_bus::{DomainEvent, EventBus},
    AppResult,
};
use std::sync::Arc;
use uuid::Uuid;

use super::dtos::*;

/// User service - handles user-related operations
/// SOLID: Single Responsibility - only handles user operations
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> AppResult<UserDto> {
        let user = User::new(req.email, req.name);
        let user_id = self.user_repo.create(&user).await?;
        Ok(UserDto {
            id: user_id,
            email: user.email,
            name: user.name,
        })
    }

    pub async fn get_user(&self, user_id: Uuid) -> AppResult<UserDto> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| crate::AppError::NotFound("User not found".to_string()))?;

        Ok(UserDto {
            id: user.id,
            email: user.email,
            name: user.name,
        })
    }
}

/// Card service - handles card (flashcard) operations
pub struct CardService {
    card_repo: Arc<dyn CardRepository>,
    event_bus: Arc<EventBus>,
}

impl CardService {
    pub fn new(card_repo: Arc<dyn CardRepository>, event_bus: Arc<EventBus>) -> Self {
        Self { card_repo, event_bus }
    }

    pub async fn create_card(&self, user_id: Uuid, req: CreateCardRequest) -> AppResult<CardDto> {
        let mut card = Card::new(user_id, req.question, req.answer);
        let deck_id = req.deck_id;
        if let Some(deck_id) = deck_id {
            card = card.with_deck(deck_id);
        }
        let card_id = self.card_repo.create(&card).await?;

        // Publish CardCreated event
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

    pub async fn get_user_cards(&self, user_id: Uuid) -> AppResult<Vec<CardDto>> {
        let cards = self.card_repo.find_by_user(user_id).await?;

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

    pub async fn get_deck_cards(&self, deck_id: Uuid) -> AppResult<Vec<CardDto>> {
        let cards = self.card_repo.find_by_deck(deck_id).await?;

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
        // Verify card exists and belongs to user
        let card = self.card_repo.find_by_id(card_id).await?
            .ok_or_else(|| crate::AppError::NotFound(format!("Card with id {} not found", card_id)))?;
        
        if card.user_id != user_id {
            return Err(crate::AppError::AuthorizationError("Cannot delete card belonging to another user".to_string()));
        }

        self.card_repo.delete(card_id).await
    }
}

/// Review service - handles review/study operations using FSRS
pub struct ReviewService {
    review_repo: Arc<dyn ReviewRepository>,
}

impl ReviewService {
    pub fn new(review_repo: Arc<dyn ReviewRepository>) -> Self {
        Self { review_repo }
    }

    pub async fn submit_review(
        &self,
        card_id: Uuid,
        user_id: Uuid,
        req: LegacyReviewCardRequest,
    ) -> AppResult<ReviewDto> {
        let review = Review::new(card_id, user_id, req.grade);
        let review_id = self.review_repo.create(&review).await?;

        Ok(ReviewDto {
            id: review_id,
            card_id: review.card_id,
            grade: review.grade,
        })
    }
}

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
        // Verify deck exists and belongs to user
        let deck = self.deck_repo.find_by_id(deck_id).await?
            .ok_or_else(|| crate::AppError::NotFound(format!("Deck with id {} not found", deck_id)))?;
        
        if deck.user_id != user_id {
            return Err(crate::AppError::AuthorizationError("Cannot delete deck belonging to another user".to_string()));
        }

        self.deck_repo.delete(deck_id).await
    }
}
