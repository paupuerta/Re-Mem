//! ImportAnki use case — bulk import cards from an Anki .apkg archive.
//!
//! An .apkg is a ZIP file containing `collection.anki21` (or `collection.anki2`),
//! a SQLite database. We extract notes from it, strip HTML, create a new deck,
//! and bulk-insert the cards.

use std::sync::Arc;

use bytes::Bytes;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    application::dtos::AnkiImportResult,
    domain::{
        entities::{Card, Deck},
        ports::EmbeddingService,
        repositories::{CardRepository, DeckRepository, DeckStatsRepository},
    },
    shared::error::{AppError, AppResult},
};

use super::import_tsv::spawn_embedding_worker;

const MAX_FILE_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const MAX_CARDS: usize = 2_000;

pub struct ImportAnkiUseCase {
    card_repo: Arc<dyn CardRepository>,
    deck_repo: Arc<dyn DeckRepository>,
    deck_stats_repo: Arc<dyn DeckStatsRepository>,
    embedding_service: Arc<dyn EmbeddingService>,
}

impl ImportAnkiUseCase {
    pub fn new(
        card_repo: Arc<dyn CardRepository>,
        deck_repo: Arc<dyn DeckRepository>,
        deck_stats_repo: Arc<dyn DeckStatsRepository>,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> Self {
        Self {
            card_repo,
            deck_repo,
            deck_stats_repo,
            embedding_service,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        file_bytes: Bytes,
    ) -> AppResult<AnkiImportResult> {
        if file_bytes.len() > MAX_FILE_BYTES {
            return Err(AppError::ValidationError(
                "File exceeds the 10 MB size limit".to_string(),
            ));
        }

        let raw = file_bytes.to_vec();

        // Unzip is synchronous — extract the collection DB bytes in a blocking thread
        let tmp_path =
            tokio::task::spawn_blocking(move || extract_collection_to_tempfile(raw))
                .await
                .map_err(|e| {
                    AppError::InternalError(format!("Anki unzip task panicked: {}", e))
                })??;

        // Open the SQLite collection file with sqlx (async, read-only)
        let opts = SqliteConnectOptions::new()
            .filename(&tmp_path)
            .read_only(true);

        let pool = SqlitePool::connect_with(opts).await.map_err(|e| {
            AppError::ValidationError(format!("Failed to open Anki collection DB: {}", e))
        })?;

        let deck_name = extract_deck_name(&pool).await;

        // Fetch up to MAX_CARDS + 1 rows to detect truncation
        let rows: Vec<(String,)> =
            sqlx::query_as(&format!("SELECT flds FROM notes LIMIT {}", MAX_CARDS + 1))
                .fetch_all(&pool)
                .await
                .map_err(|e| {
                    AppError::InternalError(format!("Failed to query Anki notes: {}", e))
                })?;

        pool.close().await;

        let mut pairs: Vec<(String, String)> = Vec::new();
        let mut skipped: u32 = 0;

        for (flds,) in &rows {
            if pairs.len() >= MAX_CARDS {
                skipped += 1;
                continue;
            }
            let parts: Vec<&str> = flds.splitn(3, '\x1f').collect();
            if parts.len() < 2 {
                tracing::warn!("Skipping Anki note with fewer than 2 fields");
                skipped += 1;
                continue;
            }
            let front = strip_html(parts[0]);
            let back = strip_html(parts[1]);
            if front.is_empty() || back.is_empty() {
                skipped += 1;
                continue;
            }
            pairs.push((front, back));
        }

        // Create a new deck from the extracted name
        let deck = Deck::new(user_id, deck_name.clone(), None);
        let deck_id = self.deck_repo.create(&deck).await?;

        if pairs.is_empty() {
            return Ok(AnkiImportResult {
                deck_id,
                deck_name,
                cards_imported: 0,
                cards_skipped: skipped,
            });
        }

        let cards: Vec<Card> = pairs
            .iter()
            .map(|(front, back)| Card::new(user_id, front.clone(), back.clone()).with_deck(deck_id))
            .collect();

        let card_ids = self.card_repo.bulk_create(&cards).await?;
        let imported = card_ids.len() as u32;

        self.deck_stats_repo
            .add_to_card_count(deck_id, imported as i32)
            .await?;

        spawn_embedding_worker(
            cards
                .into_iter()
                .zip(card_ids)
                .map(|(c, id)| (id, c.answer))
                .collect(),
            self.card_repo.clone(),
            self.embedding_service.clone(),
        );

        Ok(AnkiImportResult {
            deck_id,
            deck_name,
            cards_imported: imported,
            cards_skipped: skipped,
        })
    }
}

/// Unzip the .apkg and write `collection.anki21` / `collection.anki2` to a temp file.
/// Returns the path to the temp file.
fn extract_collection_to_tempfile(file_bytes: Vec<u8>) -> AppResult<std::path::PathBuf> {
    use std::io::{Cursor, Read, Write};

    let cursor = Cursor::new(file_bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
        AppError::ValidationError(format!("Not a valid ZIP/APKG file: {}", e))
    })?;

    let collection_name = (0..archive.len())
        .find_map(|i| {
            archive.by_index(i).ok().and_then(|f| {
                let name = f.name().to_string();
                if name == "collection.anki21" || name == "collection.anki2" {
                    Some(name)
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| {
            AppError::ValidationError(
                "No collection file found in .apkg (expected collection.anki21 or collection.anki2)".to_string(),
            )
        })?;

    let mut entry = archive.by_name(&collection_name).map_err(|e| {
        AppError::InternalError(format!("Failed to read collection from archive: {}", e))
    })?;

    let mut db_bytes = Vec::new();
    entry.read_to_end(&mut db_bytes).map_err(|e| {
        AppError::InternalError(format!("Failed to read collection bytes: {}", e))
    })?;
    drop(entry);

    let mut tmp = tempfile::Builder::new()
        .suffix(".db")
        .tempfile()
        .map_err(|e| AppError::InternalError(format!("Failed to create temp file: {}", e)))?;

    tmp.write_all(&db_bytes).map_err(|e| {
        AppError::InternalError(format!("Failed to write temp file: {}", e))
    })?;
    tmp.flush().map_err(|e| {
        AppError::InternalError(format!("Failed to flush temp file: {}", e))
    })?;

    // Keep the file on disk (persist it) so sqlx can open it
    let (_, path) = tmp.keep().map_err(|e| {
        AppError::InternalError(format!("Failed to persist temp file: {}", e))
    })?;

    Ok(path)
}

/// Extract the first non-"Default" deck name from the `col` table.
async fn extract_deck_name(pool: &SqlitePool) -> String {
    let result: Result<(String,), sqlx::Error> =
        sqlx::query_as("SELECT decks FROM col LIMIT 1")
            .fetch_one(pool)
            .await;

    let json_str = match result {
        Ok((s,)) => s,
        Err(_) => return "Imported Deck".to_string(),
    };

    let decks: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return "Imported Deck".to_string(),
    };

    if let Some(map) = decks.as_object() {
        let mut first_name: Option<String> = None;
        for (_, deck) in map {
            if let Some(name) = deck.get("name").and_then(|n| n.as_str()) {
                if first_name.is_none() {
                    first_name = Some(name.to_string());
                }
                if name != "Default" {
                    return name.to_string();
                }
            }
        }
        if let Some(name) = first_name {
            return name;
        }
    }

    "Imported Deck".to_string()
}

/// Strip HTML tags using `ammonia` (allow no tags → only text content remains).
fn strip_html(html: &str) -> String {
    ammonia::Builder::new()
        .tags(std::collections::HashSet::new())
        .clean(html)
        .to_string()
        .trim()
        .to_string()
}

