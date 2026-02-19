use crate::domain::{
    entities::{Card, Review, User},
    repositories::{CardRepository, ReviewRepository, UserRepository},
};
use crate::AppResult;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL User Repository implementation
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &User) -> AppResult<Uuid> {
        sqlx::query_scalar(
            "INSERT INTO users (id, email, name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.name)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT id, email, name, created_at, updated_at FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT id, email, name, created_at, updated_at FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    async fn update(&self, user: &User) -> AppResult<()> {
        sqlx::query(
            "UPDATE users SET email = $1, name = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(&user.email)
        .bind(&user.name)
        .bind(user.updated_at)
        .bind(user.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// PostgreSQL Card Repository implementation
pub struct PgCardRepository {
    pool: PgPool,
}

impl PgCardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CardRepository for PgCardRepository {
    async fn create(&self, card: &Card) -> AppResult<Uuid> {
        sqlx::query_scalar(
            "INSERT INTO cards (id, user_id, question, answer, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(card.id)
        .bind(card.user_id)
        .bind(&card.question)
        .bind(&card.answer)
        .bind(card.created_at)
        .bind(card.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Card>> {
        let card = sqlx::query_as::<_, Card>("SELECT id, user_id, question, answer, created_at, updated_at FROM cards WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(card)
    }

    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Card>> {
        let cards = sqlx::query_as::<_, Card>("SELECT id, user_id, question, answer, created_at, updated_at FROM cards WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(cards)
    }

    async fn update(&self, card: &Card) -> AppResult<()> {
        sqlx::query(
            "UPDATE cards SET question = $1, answer = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(&card.question)
        .bind(&card.answer)
        .bind(card.updated_at)
        .bind(card.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM cards WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

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
        let reviews = sqlx::query_as::<_, Review>("SELECT id, card_id, user_id, grade, created_at FROM reviews WHERE card_id = $1")
            .bind(card_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(reviews)
    }

    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Review>> {
        let reviews = sqlx::query_as::<_, Review>("SELECT id, card_id, user_id, grade, created_at FROM reviews WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(reviews)
    }
}
