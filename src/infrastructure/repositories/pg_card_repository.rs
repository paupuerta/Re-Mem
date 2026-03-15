use crate::{
    domain::{
        entities::{Card, FsrsState},
        repositories::CardRepository,
    },
    AppResult,
};
use pgvector::Vector;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

/// PostgreSQL Card Repository implementation
pub struct PgCardRepository {
    pool: PgPool,
}

impl PgCardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_card_rows(
        rows: Vec<(
            Uuid,
            Uuid,
            Option<Uuid>,
            String,
            String,
            Option<Vector>,
            serde_json::Value,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        )>,
    ) -> AppResult<Vec<Card>> {
        let mut cards = Vec::with_capacity(rows.len());

        for (
            id,
            user_id,
            deck_id,
            question,
            answer,
            embedding_vec,
            fsrs_state_json,
            created_at,
            updated_at,
        ) in rows
        {
            let fsrs_state: FsrsState = serde_json::from_value(fsrs_state_json)?;
            let answer_embedding = embedding_vec.map(|v| v.to_vec());
            cards.push(Card {
                id,
                user_id,
                deck_id,
                question,
                answer,
                answer_embedding,
                fsrs_state,
                created_at,
                updated_at,
            });
        }

        Ok(cards)
    }
}

#[async_trait::async_trait]
impl CardRepository for PgCardRepository {
    async fn create(&self, card: &Card) -> AppResult<Uuid> {
        let fsrs_json = serde_json::to_value(&card.fsrs_state)?;
        let embedding_vec = card
            .answer_embedding
            .as_ref()
            .map(|v| Vector::from(v.clone()));

        sqlx::query_scalar(
            "INSERT INTO cards (id, user_id, deck_id, question, answer, answer_embedding, fsrs_state, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
        )
        .bind(card.id)
        .bind(card.user_id)
        .bind(card.deck_id)
        .bind(&card.question)
        .bind(&card.answer)
        .bind(embedding_vec)
        .bind(fsrs_json)
        .bind(card.created_at)
        .bind(card.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn bulk_create(&self, cards: &[Card]) -> AppResult<Vec<Uuid>> {
        let mut tx = self.pool.begin().await?;
        let mut ids = Vec::with_capacity(cards.len());

        for card in cards {
            let fsrs_json = serde_json::to_value(&card.fsrs_state)?;
            let embedding_vec = card
                .answer_embedding
                .as_ref()
                .map(|v| Vector::from(v.clone()));

            let id: Uuid = sqlx::query_scalar(
                "INSERT INTO cards (id, user_id, deck_id, question, answer, answer_embedding, fsrs_state, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
            )
            .bind(card.id)
            .bind(card.user_id)
            .bind(card.deck_id)
            .bind(&card.question)
            .bind(&card.answer)
            .bind(embedding_vec)
            .bind(fsrs_json)
            .bind(card.created_at)
            .bind(card.updated_at)
            .fetch_one(&mut *tx)
            .await?;

            ids.push(id);
        }

        tx.commit().await?;
        Ok(ids)
    }

    async fn update_embedding(&self, id: Uuid, embedding: Vec<f32>) -> AppResult<()> {
        let embedding_vec = Vector::from(embedding);
        sqlx::query("UPDATE cards SET answer_embedding = $1, updated_at = NOW() WHERE id = $2")
            .bind(embedding_vec)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Card>> {
        let row = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<Uuid>,
                String,
                String,
                Option<Vector>,
                serde_json::Value,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            "SELECT id, user_id, deck_id, question, answer, answer_embedding, fsrs_state, created_at, updated_at 
             FROM cards WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((
                id,
                user_id,
                deck_id,
                question,
                answer,
                embedding_vec,
                fsrs_state_json,
                created_at,
                updated_at,
            )) => {
                let fsrs_state: FsrsState = serde_json::from_value(fsrs_state_json)?;
                let answer_embedding = embedding_vec.map(|v| v.to_vec());
                Ok(Some(Card {
                    id,
                    user_id,
                    deck_id,
                    question,
                    answer,
                    answer_embedding,
                    fsrs_state,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Card>> {
        self.find_by_user_paginated(user_id, None, None).await
    }

    async fn find_by_deck(&self, deck_id: Uuid) -> AppResult<Vec<Card>> {
        self.find_by_deck_paginated(deck_id, None, None).await
    }

    async fn find_by_user_paginated(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<Vec<Card>> {
        let mut query = QueryBuilder::<Postgres>::new(
            "SELECT id, user_id, deck_id, question, answer, answer_embedding, fsrs_state, created_at, updated_at \
             FROM cards WHERE user_id = ",
        );
        query.push_bind(user_id);
        query.push(" ORDER BY created_at, id");
        if let Some(limit) = limit {
            query.push(" LIMIT ");
            query.push_bind(limit);
        }
        if let Some(offset) = offset {
            query.push(" OFFSET ");
            query.push_bind(offset);
        }

        let rows = query
            .build_query_as::<(
                Uuid,
                Uuid,
                Option<Uuid>,
                String,
                String,
                Option<Vector>,
                serde_json::Value,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            )>()
            .fetch_all(&self.pool)
            .await?;

        Self::map_card_rows(rows)
    }

    async fn find_by_deck_paginated(
        &self,
        deck_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<Vec<Card>> {
        let mut query = QueryBuilder::<Postgres>::new(
            "SELECT id, user_id, deck_id, question, answer, answer_embedding, fsrs_state, created_at, updated_at \
             FROM cards WHERE deck_id = ",
        );
        query.push_bind(deck_id);
        query.push(" ORDER BY created_at, id");
        if let Some(limit) = limit {
            query.push(" LIMIT ");
            query.push_bind(limit);
        }
        if let Some(offset) = offset {
            query.push(" OFFSET ");
            query.push_bind(offset);
        }

        let rows = query
            .build_query_as::<(
                Uuid,
                Uuid,
                Option<Uuid>,
                String,
                String,
                Option<Vector>,
                serde_json::Value,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            )>()
            .fetch_all(&self.pool)
            .await?;

        Self::map_card_rows(rows)
    }

    async fn update(&self, card: &Card) -> AppResult<()> {
        let fsrs_json = serde_json::to_value(&card.fsrs_state)?;

        sqlx::query(
            "UPDATE cards SET question = $1, answer = $2, fsrs_state = $3, updated_at = $4 WHERE id = $5",
        )
        .bind(&card.question)
        .bind(&card.answer)
        .bind(fsrs_json)
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
