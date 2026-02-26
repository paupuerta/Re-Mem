use crate::AppResult;
use uuid::Uuid;

use super::entities::{Card, Deck, DeckStats, Review, ReviewLog, User, UserStats};

/// Repository interface for User domain
/// SOLID: Interface Segregation and Dependency Inversion
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> AppResult<Uuid>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn update(&self, user: &User) -> AppResult<()>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

/// Repository interface for Deck domain
#[async_trait::async_trait]
pub trait DeckRepository: Send + Sync {
    async fn create(&self, deck: &Deck) -> AppResult<Uuid>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Deck>>;
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Deck>>;
    async fn update(&self, deck: &Deck) -> AppResult<()>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

/// Repository interface for Card domain
#[async_trait::async_trait]
pub trait CardRepository: Send + Sync {
    async fn create(&self, card: &Card) -> AppResult<Uuid>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Card>>;
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Card>>;
    async fn find_by_deck(&self, deck_id: Uuid) -> AppResult<Vec<Card>>;
    async fn update(&self, card: &Card) -> AppResult<()>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

/// Repository interface for Review domain
#[async_trait::async_trait]
pub trait ReviewRepository: Send + Sync {
    async fn create(&self, review: &Review) -> AppResult<Uuid>;
    async fn find_by_card(&self, card_id: Uuid) -> AppResult<Vec<Review>>;
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Review>>;
}

/// Repository interface for ReviewLog domain
#[async_trait::async_trait]
pub trait ReviewLogRepository: Send + Sync {
    async fn create(&self, review_log: &ReviewLog) -> AppResult<Uuid>;
    async fn find_by_card(&self, card_id: Uuid) -> AppResult<Vec<ReviewLog>>;
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<ReviewLog>>;
}

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
}
