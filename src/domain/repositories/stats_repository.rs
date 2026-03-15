use crate::{
    domain::entities::{DeckStats, UserStats},
    AppResult,
};
use uuid::Uuid;

/// Repository interface for UserStats domain
#[async_trait::async_trait]
pub trait UserStatsRepository: Send + Sync {
    async fn get_or_create(&self, user_id: Uuid) -> AppResult<UserStats>;
    async fn update_after_review(
        &self,
        user_id: Uuid,
        is_correct: bool,
        review_date: chrono::NaiveDate,
    ) -> AppResult<()>;
}

/// Repository interface for DeckStats domain
#[async_trait::async_trait]
pub trait DeckStatsRepository: Send + Sync {
    async fn get_or_create(&self, deck_id: Uuid, user_id: Uuid) -> AppResult<DeckStats>;
    async fn update_after_review(
        &self,
        deck_id: Uuid,
        is_correct: bool,
        review_date: chrono::NaiveDate,
    ) -> AppResult<()>;
    async fn increment_card_count(&self, deck_id: Uuid) -> AppResult<()>;
    async fn decrement_card_count(&self, deck_id: Uuid) -> AppResult<()>;
    async fn add_to_card_count(&self, deck_id: Uuid, count: i32) -> AppResult<()>;
}
