use crate::{
    domain::{entities::*, repositories::*},
    AppResult,
};
use uuid::Uuid;

use super::dtos::*;

/// User service - handles user-related operations
/// SOLID: Single Responsibility - only handles user operations
pub struct UserService {
    user_repo: std::sync::Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repo: std::sync::Arc<dyn UserRepository>) -> Self {
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
    card_repo: std::sync::Arc<dyn CardRepository>,
}

impl CardService {
    pub fn new(card_repo: std::sync::Arc<dyn CardRepository>) -> Self {
        Self { card_repo }
    }

    pub async fn create_card(&self, user_id: Uuid, req: CreateCardRequest) -> AppResult<CardDto> {
        let card = Card::new(user_id, req.question, req.answer);
        let card_id = self.card_repo.create(&card).await?;

        Ok(CardDto {
            id: card_id,
            user_id: card.user_id,
            question: card.question,
            answer: card.answer,
        })
    }

    pub async fn get_user_cards(&self, user_id: Uuid) -> AppResult<Vec<CardDto>> {
        let cards = self.card_repo.find_by_user(user_id).await?;

        Ok(cards
            .into_iter()
            .map(|card| CardDto {
                id: card.id,
                user_id: card.user_id,
                question: card.question,
                answer: card.answer,
            })
            .collect())
    }
}

/// Review service - handles review/study operations using FSRS
pub struct ReviewService {
    review_repo: std::sync::Arc<dyn ReviewRepository>,
}

impl ReviewService {
    pub fn new(review_repo: std::sync::Arc<dyn ReviewRepository>) -> Self {
        Self { review_repo }
    }

    pub async fn submit_review(
        &self,
        card_id: Uuid,
        user_id: Uuid,
        req: ReviewCardRequest,
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
