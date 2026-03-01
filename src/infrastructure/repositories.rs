use crate::domain::{
    entities::{Card, Deck, DeckStats, FsrsState, Review, ReviewLog, User, UserStats},
    repositories::{
        CardRepository, DeckRepository, DeckStatsRepository, ReviewLogRepository,
        ReviewRepository, UserRepository, UserStatsRepository,
    },
};
use crate::AppResult;
use pgvector::Vector;
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
            "INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.name)
        .bind(&user.password_hash)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, name, password_hash, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, name, password_hash, created_at, updated_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn update(&self, user: &User) -> AppResult<()> {
        sqlx::query("UPDATE users SET email = $1, name = $2, password_hash = $3, updated_at = $4 WHERE id = $5")
            .bind(&user.email)
            .bind(&user.name)
            .bind(&user.password_hash)
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
        let fsrs_json = serde_json::to_value(&card.fsrs_state)?;
        let embedding_vec = card.answer_embedding.as_ref().map(|v| Vector::from(v.clone()));

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
            let embedding_vec = card.answer_embedding.as_ref().map(|v| Vector::from(v.clone()));

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
        sqlx::query(
            "UPDATE cards SET answer_embedding = $1, updated_at = NOW() WHERE id = $2",
        )
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
            Some((id, user_id, deck_id, question, answer, embedding_vec, fsrs_state_json, created_at, updated_at)) => {
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
        let rows = sqlx::query_as::<
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
             FROM cards WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut cards = Vec::with_capacity(rows.len());
        for (id, user_id, deck_id, question, answer, embedding_vec, fsrs_state_json, created_at, updated_at) in rows {
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

    async fn find_by_deck(&self, deck_id: Uuid) -> AppResult<Vec<Card>> {
        let rows = sqlx::query_as::<
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
             FROM cards WHERE deck_id = $1",
        )
        .bind(deck_id)
        .fetch_all(&self.pool)
        .await?;

        let mut cards = Vec::with_capacity(rows.len());
        for (id, user_id, deck_id, question, answer, embedding_vec, fsrs_state_json, created_at, updated_at) in rows {
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
             FROM review_logs WHERE card_id = $1 ORDER BY created_at DESC"
        )
        .bind(card_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(logs)
    }

    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<ReviewLog>> {
        let logs = sqlx::query_as::<_, ReviewLog>(
            "SELECT id, card_id, user_id, user_answer, ai_score, fsrs_rating, validation_method, created_at 
             FROM review_logs WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(logs)
    }
}

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
        sqlx::query(
            "UPDATE decks SET name = $1, description = $2, updated_at = $3 WHERE id = $4"
        )
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
        // Try to get existing stats
        let stats = sqlx::query_as::<_, UserStats>(
            "SELECT user_id, total_reviews, correct_reviews, days_studied, last_active_date, created_at, updated_at
             FROM user_stats WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match stats {
            Some(s) => Ok(s),
            None => {
                // Create new stats
                let new_stats = UserStats::new(user_id);
                sqlx::query(
                    "INSERT INTO user_stats (user_id, total_reviews, correct_reviews, days_studied, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6)"
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
        // Get current stats to check if this is a new day
        let current_stats = self.get_or_create(user_id).await?;
        
        let is_new_day = current_stats.last_active_date
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
             WHERE user_id = $4"
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
        // Try to get existing stats
        let stats = sqlx::query_as::<_, DeckStats>(
            "SELECT deck_id, user_id, total_cards, total_reviews, correct_reviews, days_studied, last_active_date, created_at, updated_at
             FROM deck_stats WHERE deck_id = $1"
        )
        .bind(deck_id)
        .fetch_optional(&self.pool)
        .await?;

        match stats {
            Some(s) => Ok(s),
            None => {
                // Create new stats
                let new_stats = DeckStats::new(deck_id, user_id);
                sqlx::query(
                    "INSERT INTO deck_stats (deck_id, user_id, total_cards, total_reviews, correct_reviews, days_studied, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
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
        // Get current stats to check if this is a new day
        let current_stats = sqlx::query_as::<_, DeckStats>(
            "SELECT deck_id, user_id, total_cards, total_reviews, correct_reviews, days_studied, last_active_date, created_at, updated_at
             FROM deck_stats WHERE deck_id = $1"
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
             WHERE deck_id = $4"
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
             WHERE deck_id = $1"
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
             WHERE deck_id = $1"
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
