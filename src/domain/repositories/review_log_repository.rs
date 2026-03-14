use crate::{domain::entities::ReviewLog, AppResult};
use uuid::Uuid;

/// Repository interface for ReviewLog domain
#[async_trait::async_trait]
pub trait ReviewLogRepository: Send + Sync {
    async fn create(&self, review_log: &ReviewLog) -> AppResult<Uuid>;
    async fn find_by_card(&self, card_id: Uuid) -> AppResult<Vec<ReviewLog>>;
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<ReviewLog>>;
}
