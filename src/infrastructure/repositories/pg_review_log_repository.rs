use crate::{
    domain::{entities::ReviewLog, repositories::ReviewLogRepository},
    AppResult,
};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL ReviewLog Repository implementation
pub struct PgReviewLogRepository {
    pool: PgPool,
}

impl PgReviewLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ReviewLogRepository for PgReviewLogRepository {
    async fn create(&self, review_log: &ReviewLog) -> AppResult<Uuid> {
        sqlx::query_scalar(
            "INSERT INTO review_logs (id, card_id, user_id, user_answer, ai_score, fsrs_rating, validation_method, created_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        )
        .bind(review_log.id)
        .bind(review_log.card_id)
        .bind(review_log.user_id)
        .bind(&review_log.user_answer)
        .bind(review_log.ai_score)
        .bind(review_log.fsrs_rating)
        .bind(&review_log.validation_method)
        .bind(review_log.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_card(&self, card_id: Uuid) -> AppResult<Vec<ReviewLog>> {
        let logs = sqlx::query_as::<_, ReviewLog>(
            "SELECT id, card_id, user_id, user_answer, ai_score, fsrs_rating, validation_method, created_at 
             FROM review_logs WHERE card_id = $1 ORDER BY created_at DESC",
        )
        .bind(card_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(logs)
    }

    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<ReviewLog>> {
        let logs = sqlx::query_as::<_, ReviewLog>(
            "SELECT id, card_id, user_id, user_answer, ai_score, fsrs_rating, validation_method, created_at 
             FROM review_logs WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(logs)
    }
}
