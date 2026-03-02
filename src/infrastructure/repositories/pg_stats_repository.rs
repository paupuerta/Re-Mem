use crate::{
    domain::{
        entities::{DeckStats, UserStats},
        repositories::{DeckStatsRepository, UserStatsRepository},
    },
    AppResult,
};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL UserStats Repository implementation
pub struct PgUserStatsRepository {
    pool: PgPool,
}

impl PgUserStatsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStatsRepository for PgUserStatsRepository {
    async fn get_or_create(&self, user_id: Uuid) -> AppResult<UserStats> {
        let stats = sqlx::query_as::<_, UserStats>(
            "SELECT user_id, total_reviews, correct_reviews, days_studied, last_active_date, created_at, updated_at
             FROM user_stats WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match stats {
            Some(s) => Ok(s),
            None => {
                let new_stats = UserStats::new(user_id);
                sqlx::query(
                    "INSERT INTO user_stats (user_id, total_reviews, correct_reviews, days_studied, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(new_stats.user_id)
                .bind(new_stats.total_reviews)
                .bind(new_stats.correct_reviews)
                .bind(new_stats.days_studied)
                .bind(new_stats.created_at)
                .bind(new_stats.updated_at)
                .execute(&self.pool)
                .await?;
                Ok(new_stats)
            }
        }
    }

    async fn update_after_review(
        &self,
        user_id: Uuid,
        is_correct: bool,
        review_date: chrono::NaiveDate,
    ) -> AppResult<()> {
        let current_stats = self.get_or_create(user_id).await?;

        let is_new_day = current_stats
            .last_active_date
            .map(|last_date| last_date != review_date)
            .unwrap_or(true);

        let days_increment = if is_new_day { 1 } else { 0 };
        let correct_increment = if is_correct { 1 } else { 0 };

        sqlx::query(
            "UPDATE user_stats 
             SET total_reviews = total_reviews + 1,
                 correct_reviews = correct_reviews + $1,
                 days_studied = days_studied + $2,
                 last_active_date = $3,
                 updated_at = NOW()
             WHERE user_id = $4",
        )
        .bind(correct_increment)
        .bind(days_increment)
        .bind(review_date)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// PostgreSQL DeckStats Repository implementation
pub struct PgDeckStatsRepository {
    pool: PgPool,
}

impl PgDeckStatsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DeckStatsRepository for PgDeckStatsRepository {
    async fn get_or_create(&self, deck_id: Uuid, user_id: Uuid) -> AppResult<DeckStats> {
        let stats = sqlx::query_as::<_, DeckStats>(
            "SELECT deck_id, user_id, total_cards, total_reviews, correct_reviews, days_studied, last_active_date, created_at, updated_at
             FROM deck_stats WHERE deck_id = $1",
        )
        .bind(deck_id)
        .fetch_optional(&self.pool)
        .await?;

        match stats {
            Some(s) => Ok(s),
            None => {
                let new_stats = DeckStats::new(deck_id, user_id);
                sqlx::query(
                    "INSERT INTO deck_stats (deck_id, user_id, total_cards, total_reviews, correct_reviews, days_studied, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(new_stats.deck_id)
                .bind(new_stats.user_id)
                .bind(new_stats.total_cards)
                .bind(new_stats.total_reviews)
                .bind(new_stats.correct_reviews)
                .bind(new_stats.days_studied)
                .bind(new_stats.created_at)
                .bind(new_stats.updated_at)
                .execute(&self.pool)
                .await?;
                Ok(new_stats)
            }
        }
    }

    async fn update_after_review(
        &self,
        deck_id: Uuid,
        is_correct: bool,
        review_date: chrono::NaiveDate,
    ) -> AppResult<()> {
        let current_stats = sqlx::query_as::<_, DeckStats>(
            "SELECT deck_id, user_id, total_cards, total_reviews, correct_reviews, days_studied, last_active_date, created_at, updated_at
             FROM deck_stats WHERE deck_id = $1",
        )
        .bind(deck_id)
        .fetch_optional(&self.pool)
        .await?;

        let is_new_day = current_stats
            .and_then(|s| s.last_active_date)
            .map(|last_date| last_date != review_date)
            .unwrap_or(true);

        let days_increment = if is_new_day { 1 } else { 0 };
        let correct_increment = if is_correct { 1 } else { 0 };

        sqlx::query(
            "UPDATE deck_stats 
             SET total_reviews = total_reviews + 1,
                 correct_reviews = correct_reviews + $1,
                 days_studied = days_studied + $2,
                 last_active_date = $3,
                 updated_at = NOW()
             WHERE deck_id = $4",
        )
        .bind(correct_increment)
        .bind(days_increment)
        .bind(review_date)
        .bind(deck_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn increment_card_count(&self, deck_id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE deck_stats 
             SET total_cards = total_cards + 1,
                 updated_at = NOW()
             WHERE deck_id = $1",
        )
        .bind(deck_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn decrement_card_count(&self, deck_id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE deck_stats 
             SET total_cards = GREATEST(total_cards - 1, 0),
                 updated_at = NOW()
             WHERE deck_id = $1",
        )
        .bind(deck_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn add_to_card_count(&self, deck_id: Uuid, count: i32) -> AppResult<()> {
        sqlx::query(
            "UPDATE deck_stats SET total_cards = total_cards + $1, updated_at = NOW() WHERE deck_id = $2",
        )
        .bind(count)
        .bind(deck_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
