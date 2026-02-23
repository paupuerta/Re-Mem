use std::sync::Arc;
use chrono::Utc;

use crate::{
    domain::repositories::{DeckStatsRepository, UserStatsRepository, CardRepository},
    shared::event_bus::{DomainEvent, EventHandler},
    AppResult,
};

/// Event handler that updates precalculated statistics when cards are reviewed
pub struct StatisticsEventHandler {
    user_stats_repo: Arc<dyn UserStatsRepository>,
    deck_stats_repo: Arc<dyn DeckStatsRepository>,
    card_repo: Arc<dyn CardRepository>,
}

impl StatisticsEventHandler {
    pub fn new(
        user_stats_repo: Arc<dyn UserStatsRepository>,
        deck_stats_repo: Arc<dyn DeckStatsRepository>,
        card_repo: Arc<dyn CardRepository>,
    ) -> Self {
        Self {
            user_stats_repo,
            deck_stats_repo,
            card_repo,
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for StatisticsEventHandler {
    async fn handle(&self, event: DomainEvent) -> AppResult<()> {
        match event {
            DomainEvent::CardReviewed {
                card_id,
                user_id,
                score,
                rating: _,
            } => {
                // Determine if the review was correct (score >= 70%)
                let is_correct = score >= 0.7;
                
                // Get current UTC date for tracking "days studied"
                let review_date = Utc::now().date_naive();

                // Update user-level statistics
                self.user_stats_repo
                    .update_after_review(user_id, is_correct, review_date)
                    .await?;

                // Get the card to find its deck (if any)
                if let Some(card) = self.card_repo.find_by_id(card_id).await? {
                    if let Some(deck_id) = card.deck_id {
                        // Update deck-level statistics
                        self.deck_stats_repo
                            .update_after_review(deck_id, is_correct, review_date)
                            .await?;
                    }
                }

                tracing::info!(
                    "Statistics updated for user {} after reviewing card {}",
                    user_id,
                    card_id
                );
            }
            DomainEvent::CardCreated { card_id, user_id: _ } => {
                // When a card is created, update deck card count if it belongs to a deck
                if let Some(card) = self.card_repo.find_by_id(card_id).await? {
                    if let Some(deck_id) = card.deck_id {
                        self.deck_stats_repo.increment_card_count(deck_id).await?;
                        tracing::info!("Deck {} card count incremented", deck_id);
                    }
                }
            }
        }
        Ok(())
    }
}
