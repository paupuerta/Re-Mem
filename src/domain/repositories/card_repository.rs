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
        exclude_card_ids: Option<Vec<Uuid>>,
    ) -> AppResult<Vec<Card>> {
        let cards = self.find_by_user(user_id).await?;
        Ok(paginate_cards(cards, limit, offset, exclude_card_ids))
    }
    async fn find_by_deck_paginated(
        &self,
        deck_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
        exclude_card_ids: Option<Vec<Uuid>>,
    ) -> AppResult<Vec<Card>> {
        let cards = self.find_by_deck(deck_id).await?;
        Ok(paginate_cards(cards, limit, offset, exclude_card_ids))
    }
    async fn update(&self, card: &Card) -> AppResult<()>;
    async fn update_embedding(&self, id: Uuid, embedding: Vec<f32>) -> AppResult<()>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

fn paginate_cards(
    cards: Vec<Card>,
    limit: Option<i64>,
    offset: Option<i64>,
    exclude_card_ids: Option<Vec<Uuid>>,
) -> Vec<Card> {
    let start = offset.unwrap_or(0).max(0) as usize;
    let max_items = limit.unwrap_or(i64::MAX).max(0) as usize;
    let exclude_card_ids = exclude_card_ids.unwrap_or_default();

    cards
        .into_iter()
        .filter(|card| !exclude_card_ids.contains(&card.id))
        .skip(start)
        .take(max_items)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::paginate_cards;
    use crate::domain::entities::Card;
    use uuid::Uuid;

    #[test]
    fn paginate_cards_excludes_loaded_cards() {
        let user_id = Uuid::new_v4();
        let card1 = Card::new(user_id, "Q1".to_string(), "A1".to_string());
        let card2 = Card::new(user_id, "Q2".to_string(), "A2".to_string());
        let card3 = Card::new(user_id, "Q3".to_string(), "A3".to_string());

        let paginated = paginate_cards(
            vec![card1.clone(), card2.clone(), card3.clone()],
            Some(10),
            Some(0),
            Some(vec![card1.id, card3.id]),
        );

        assert_eq!(paginated.len(), 1);
        assert_eq!(paginated[0].id, card2.id);
    }
}
