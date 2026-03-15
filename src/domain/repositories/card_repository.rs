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
    async fn find_by_user_paginated(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<Vec<Card>> {
        let cards = self.find_by_user(user_id).await?;
        Ok(paginate_cards(cards, limit, offset))
    }
    async fn find_by_deck_paginated(
        &self,
        deck_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<Vec<Card>> {
        let cards = self.find_by_deck(deck_id).await?;
        Ok(paginate_cards(cards, limit, offset))
    }
    async fn update(&self, card: &Card) -> AppResult<()>;
    async fn update_embedding(&self, id: Uuid, embedding: Vec<f32>) -> AppResult<()>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

fn paginate_cards(cards: Vec<Card>, limit: Option<i64>, offset: Option<i64>) -> Vec<Card> {
    let start = offset.unwrap_or(0).max(0) as usize;
    let max_items = limit.unwrap_or(i64::MAX).max(0) as usize;

    cards.into_iter().skip(start).take(max_items).collect()
}
