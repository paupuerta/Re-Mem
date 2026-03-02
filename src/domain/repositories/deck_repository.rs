use crate::{domain::entities::Deck, AppResult};
use uuid::Uuid;

/// Repository interface for Deck domain
#[async_trait::async_trait]
pub trait DeckRepository: Send + Sync {
    async fn create(&self, deck: &Deck) -> AppResult<Uuid>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Deck>>;
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Deck>>;
    async fn update(&self, deck: &Deck) -> AppResult<()>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}
