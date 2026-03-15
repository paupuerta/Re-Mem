use crate::{
    domain::{entities::Deck, repositories::DeckRepository},
    AppResult,
};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL Deck Repository implementation
pub struct PgDeckRepository {
    pool: PgPool,
}

impl PgDeckRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DeckRepository for PgDeckRepository {
    async fn create(&self, deck: &Deck) -> AppResult<Uuid> {
        sqlx::query_scalar(
            "INSERT INTO decks (id, user_id, name, description, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(deck.id)
        .bind(deck.user_id)
        .bind(&deck.name)
        .bind(&deck.description)
        .bind(deck.created_at)
        .bind(deck.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Deck>> {
        let deck = sqlx::query_as::<_, Deck>(
            "SELECT id, user_id, name, description, created_at, updated_at 
             FROM decks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(deck)
    }

    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Deck>> {
        let decks = sqlx::query_as::<_, Deck>(
            "SELECT id, user_id, name, description, created_at, updated_at 
             FROM decks WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(decks)
    }

    async fn update(&self, deck: &Deck) -> AppResult<()> {
        sqlx::query("UPDATE decks SET name = $1, description = $2, updated_at = $3 WHERE id = $4")
            .bind(&deck.name)
            .bind(&deck.description)
            .bind(deck.updated_at)
            .bind(deck.id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM decks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
