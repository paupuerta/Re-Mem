use crate::AppResult;
use uuid::Uuid;

use super::entities::{Card, Review, User};

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

/// Repository interface for Card domain
#[async_trait::async_trait]
pub trait CardRepository: Send + Sync {
    async fn create(&self, card: &Card) -> AppResult<Uuid>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Card>>;
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Card>>;
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
