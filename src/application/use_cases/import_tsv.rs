//! ImportTsv use case — bulk import cards from a TSV file into an existing deck.

use std::sync::Arc;

use bytes::Bytes;
use uuid::Uuid;

use crate::{
    application::dtos::ImportResult,
    domain::{
        entities::Card,
        ports::EmbeddingService,
        repositories::{CardRepository, DeckStatsRepository},
    },
    shared::error::{AppError, AppResult},
};

const MAX_FILE_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const MAX_CARDS: usize = 2_000;

pub struct ImportTsvUseCase {
    card_repo: Arc<dyn CardRepository>,
    deck_stats_repo: Arc<dyn DeckStatsRepository>,
    embedding_service: Arc<dyn EmbeddingService>,
}

impl ImportTsvUseCase {
    pub fn new(
        card_repo: Arc<dyn CardRepository>,
        deck_stats_repo: Arc<dyn DeckStatsRepository>,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> Self {
        Self {
            card_repo,
            deck_stats_repo,
            embedding_service,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        deck_id: Uuid,
        file_bytes: Bytes,
    ) -> AppResult<ImportResult> {
        if file_bytes.len() > MAX_FILE_BYTES {
            return Err(AppError::ValidationError(
                "File exceeds the 10 MB size limit".to_string(),
            ));
        }

        let text = std::str::from_utf8(&file_bytes)
            .map_err(|_| AppError::ValidationError("File is not valid UTF-8".to_string()))?;

        let mut cards: Vec<Card> = Vec::new();
        let mut skipped: u32 = 0;

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.splitn(2, '\t');
            let front = match parts.next() {
                Some(f) if !f.trim().is_empty() => f.trim().to_string(),
                _ => {
                    tracing::warn!("Skipping TSV line (missing front): {:?}", line);
                    skipped += 1;
                    continue;
                }
            };
            let back = match parts.next() {
                Some(b) if !b.trim().is_empty() => b.trim().to_string(),
                _ => {
                    tracing::warn!("Skipping TSV line (missing back): {:?}", line);
                    skipped += 1;
                    continue;
                }
            };

            if cards.len() >= MAX_CARDS {
                skipped += 1;
                continue;
            }

            cards.push(Card::new(user_id, front, back).with_deck(deck_id));
        }

        if cards.is_empty() {
            return Ok(ImportResult {
                cards_imported: 0,
                cards_skipped: skipped,
            });
        }

        let card_ids = self.card_repo.bulk_create(&cards).await?;
        let imported = card_ids.len() as u32;

        // Update deck stats card count
        self.deck_stats_repo
            .add_to_card_count(deck_id, imported as i32)
            .await?;

        // Spawn background task to generate embeddings without blocking the response
        spawn_embedding_worker(
            cards
                .into_iter()
                .zip(card_ids)
                .map(|(c, id)| (id, c.answer))
                .collect(),
            self.card_repo.clone(),
            self.embedding_service.clone(),
        );

        Ok(ImportResult {
            cards_imported: imported,
            cards_skipped: skipped,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    use crate::{
        domain::{
            entities::{Card, DeckStats},
            repositories::{CardRepository, DeckStatsRepository},
        },
        AppError,
    };

    // ── Mocks ──────────────────────────────────────────────────────────────────

    struct MockCardRepo {
        fail: bool,
    }

    #[async_trait]
    impl CardRepository for MockCardRepo {
        async fn create(&self, card: &Card) -> AppResult<Uuid> {
            Ok(card.id)
        }
        async fn bulk_create(&self, cards: &[Card]) -> AppResult<Vec<Uuid>> {
            if self.fail {
                return Err(AppError::InternalError("db error".to_string()));
            }
            Ok(cards.iter().map(|c| c.id).collect())
        }
        async fn find_by_id(&self, _id: Uuid) -> AppResult<Option<Card>> {
            Ok(None)
        }
        async fn find_by_user(&self, _user_id: Uuid) -> AppResult<Vec<Card>> {
            Ok(vec![])
        }
        async fn find_by_deck(&self, _deck_id: Uuid) -> AppResult<Vec<Card>> {
            Ok(vec![])
        }
        async fn update(&self, _card: &Card) -> AppResult<()> {
            Ok(())
        }
        async fn update_embedding(&self, _id: Uuid, _embedding: Vec<f32>) -> AppResult<()> {
            Ok(())
        }
        async fn delete(&self, _id: Uuid) -> AppResult<()> {
            Ok(())
        }
    }

    struct MockDeckStatsRepo;

    #[async_trait]
    impl DeckStatsRepository for MockDeckStatsRepo {
        async fn get_or_create(&self, deck_id: Uuid, user_id: Uuid) -> AppResult<DeckStats> {
            Ok(DeckStats::new(deck_id, user_id))
        }
        async fn update_after_review(
            &self,
            _deck_id: Uuid,
            _is_correct: bool,
            _review_date: chrono::NaiveDate,
        ) -> AppResult<()> {
            Ok(())
        }
        async fn increment_card_count(&self, _deck_id: Uuid) -> AppResult<()> {
            Ok(())
        }
        async fn decrement_card_count(&self, _deck_id: Uuid) -> AppResult<()> {
            Ok(())
        }
        async fn add_to_card_count(&self, _deck_id: Uuid, _count: i32) -> AppResult<()> {
            Ok(())
        }
    }

    struct MockEmbeddingService;

    #[async_trait]
    impl crate::domain::ports::EmbeddingService for MockEmbeddingService {
        async fn generate_embedding(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
            Ok(vec![0.1, 0.2, 0.3])
        }
    }

    fn make_use_case(fail_repo: bool) -> ImportTsvUseCase {
        ImportTsvUseCase::new(
            Arc::new(MockCardRepo { fail: fail_repo }),
            Arc::new(MockDeckStatsRepo),
            Arc::new(MockEmbeddingService),
        )
    }

    // ── Tests ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_import_tsv_happy_path() {
        let tsv = "Hello\tHola\nWorld\tMundo\n";
        let result = make_use_case(false)
            .execute(Uuid::new_v4(), Uuid::new_v4(), Bytes::from(tsv))
            .await;
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.cards_imported, 2);
        assert_eq!(r.cards_skipped, 0);
    }

    #[tokio::test]
    async fn test_import_tsv_skips_malformed_lines() {
        // Line 1: valid, Line 2: no tab (malformed), Line 3: empty, Line 4: valid
        let tsv = "Cat\tGato\nno_tab_here\n\nDog\tPerro\n";
        let result = make_use_case(false)
            .execute(Uuid::new_v4(), Uuid::new_v4(), Bytes::from(tsv))
            .await;
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.cards_imported, 2);
        assert_eq!(r.cards_skipped, 1); // "no_tab_here" — empty lines don't increment skipped
    }

    #[tokio::test]
    async fn test_import_tsv_empty_file() {
        let result = make_use_case(false)
            .execute(Uuid::new_v4(), Uuid::new_v4(), Bytes::from(""))
            .await;
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.cards_imported, 0);
        assert_eq!(r.cards_skipped, 0);
    }

    #[tokio::test]
    async fn test_import_tsv_file_too_large() {
        let big = vec![b'a'; MAX_FILE_BYTES + 1];
        let result = make_use_case(false)
            .execute(Uuid::new_v4(), Uuid::new_v4(), Bytes::from(big))
            .await;
        assert!(matches!(result, Err(AppError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_import_tsv_invalid_utf8() {
        let bad = Bytes::from(vec![0xFF, 0xFE, 0x00]);
        let result = make_use_case(false)
            .execute(Uuid::new_v4(), Uuid::new_v4(), bad)
            .await;
        assert!(matches!(result, Err(AppError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_import_tsv_repo_failure_propagates() {
        let tsv = "A\tB\n";
        let result = make_use_case(true)
            .execute(Uuid::new_v4(), Uuid::new_v4(), Bytes::from(tsv))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_import_tsv_strips_whitespace() {
        // Leading/trailing whitespace around front/back should be trimmed
        let tsv = "  Apple  \t  Manzana  \n";
        let result = make_use_case(false)
            .execute(Uuid::new_v4(), Uuid::new_v4(), Bytes::from(tsv))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().cards_imported, 1);
    }
}

/// Spawns a detached Tokio task that generates embeddings for newly imported cards.
pub fn spawn_embedding_worker(
    tasks: Vec<(Uuid, String)>,
    card_repo: Arc<dyn CardRepository>,
    embedding_service: Arc<dyn EmbeddingService>,
) {
    tokio::spawn(async move {
        for (card_id, answer_text) in tasks {
            match embedding_service.generate_embedding(&answer_text).await {
                Ok(embedding) => {
                    if let Err(e) = card_repo.update_embedding(card_id, embedding).await {
                        tracing::warn!("Failed to store embedding for card {}: {}", card_id, e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to generate embedding for card {}: {}", card_id, e);
                }
            }
        }
    });
}
