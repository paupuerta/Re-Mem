use crate::{domain::entities::Card, AppResult};
use uuid::Uuid;

/// Repository interface for Card domain
#[async_trait::async_trait]
pub trait CardRepository: Send + Sync {
    async fn create(&self, card: &Card) -> AppResult<Uuid>;
    async fn bulk_create(&self, cards: &[Card]) -> AppResult<Vec<Uuid>>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Card>>;
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Card>>;
    async fn find_by_deck(&self, deck_id: Uuid) -> AppResult<Vec<Card>>;
    async fn update(&self, card: &Card) -> AppResult<()>;
    async fn update_embedding(&self, id: Uuid, embedding: Vec<f32>) -> AppResult<()>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}
