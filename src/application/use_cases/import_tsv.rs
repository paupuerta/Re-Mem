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
