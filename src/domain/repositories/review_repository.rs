use crate::{domain::entities::Review, AppResult};
use uuid::Uuid;

/// Repository interface for Review domain
#[async_trait::async_trait]
pub trait ReviewRepository: Send + Sync {
    async fn create(&self, review: &Review) -> AppResult<Uuid>;
    async fn find_by_card(&self, card_id: Uuid) -> AppResult<Vec<Review>>;
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Review>>;
}
