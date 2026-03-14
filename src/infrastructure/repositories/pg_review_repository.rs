use crate::{
    domain::{entities::Review, repositories::ReviewRepository},
    AppResult,
};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL Review Repository implementation
pub struct PgReviewRepository {
    pool: PgPool,
}

impl PgReviewRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ReviewRepository for PgReviewRepository {
    async fn create(&self, review: &Review) -> AppResult<Uuid> {
        sqlx::query_scalar(
            "INSERT INTO reviews (id, card_id, user_id, grade, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(review.id)
        .bind(review.card_id)
        .bind(review.user_id)
        .bind(review.grade)
        .bind(review.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_card(&self, card_id: Uuid) -> AppResult<Vec<Review>> {
        let reviews = sqlx::query_as::<_, Review>(
            "SELECT id, card_id, user_id, grade, created_at FROM reviews WHERE card_id = $1",
        )
        .bind(card_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(reviews)
    }

    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Review>> {
        let reviews = sqlx::query_as::<_, Review>(
            "SELECT id, card_id, user_id, grade, created_at FROM reviews WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(reviews)
    }
}
