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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::io::{Cursor, Write};

    use crate::{
        domain::{
            entities::{Card, Deck, DeckStats},
            repositories::{CardRepository, DeckRepository, DeckStatsRepository},
        },
        AppError,
    };

    // ── Mocks ──────────────────────────────────────────────────────────────────

    struct MockCardRepo;

    #[async_trait]
    impl CardRepository for MockCardRepo {
        async fn create(&self, card: &Card) -> AppResult<Uuid> {
            Ok(card.id)
        }
        async fn bulk_create(&self, cards: &[Card]) -> AppResult<Vec<Uuid>> {
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

    struct MockDeckRepo;

    #[async_trait]
    impl DeckRepository for MockDeckRepo {
        async fn create(&self, deck: &Deck) -> AppResult<Uuid> {
            Ok(deck.id)
        }
        async fn find_by_id(&self, _id: Uuid) -> AppResult<Option<Deck>> {
            Ok(None)
        }
        async fn find_by_user(&self, _user_id: Uuid) -> AppResult<Vec<Deck>> {
            Ok(vec![])
        }
        async fn update(&self, _deck: &Deck) -> AppResult<()> {
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
        async fn update_after_review(&self, _deck_id: Uuid, _is_correct: bool, _review_date: chrono::NaiveDate) -> AppResult<()> {
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

    fn make_use_case() -> ImportAnkiUseCase {
        ImportAnkiUseCase::new(
            Arc::new(MockCardRepo),
            Arc::new(MockDeckRepo),
            Arc::new(MockDeckStatsRepo),
            Arc::new(MockEmbeddingService),
        )
    }

    /// Build a minimal `.apkg` (ZIP containing a SQLite DB) in memory.
    /// The SQLite DB has a `notes` table with the given `(front, back)` pairs.
    fn build_test_apkg(notes: &[(&str, &str)], deck_name: Option<&str>) -> Vec<u8> {

        // 1. Create an in-memory SQLite DB via a temp file
        let tmp = tempfile::Builder::new()
            .suffix(".db")
            .tempfile()
            .expect("tempfile");
        let path = tmp.path().to_owned();

        // Use the rusqlite-independent sqlite3 binary isn't available in tests.
        // We use sqlx's runtime to create the DB — but since this is synchronous
        // test code, use the standard library's approach via the `sqlite3` C API
        // wrapped by sqlx is async. Instead, write raw SQLite bytes directly.
        // A simpler approach: use `std::process::Command` is brittle.
        //
        // We rely on the fact that sqlx sqlite feature is available.
        // Use a blocking Tokio runtime inline.
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
            let opts = sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true);
            let pool = sqlx::SqlitePool::connect_with(opts).await.unwrap();

            sqlx::query(
                "CREATE TABLE notes (id INTEGER PRIMARY KEY, flds TEXT NOT NULL)",
            )
            .execute(&pool)
            .await
            .unwrap();

            for (front, back) in notes {
                let flds = format!("{}\x1f{}", front, back);
                sqlx::query("INSERT INTO notes (flds) VALUES (?)")
                    .bind(flds)
                    .execute(&pool)
                    .await
                    .unwrap();
            }

            if let Some(name) = deck_name {
                let decks_json = format!(
                    r#"{{"1":{{"id":1,"name":"{}","usn":0}}}}"#,
                    name
                );
                sqlx::query("CREATE TABLE col (id INTEGER PRIMARY KEY, decks TEXT)")
                    .execute(&pool)
                    .await
                    .unwrap();
                sqlx::query("INSERT INTO col (id, decks) VALUES (1, ?)")
                    .bind(decks_json)
                    .execute(&pool)
                    .await
                    .unwrap();
            }

            pool.close().await;
            })
        });

        let db_bytes = std::fs::read(&path).expect("read tmp db");

        // 2. Zip the SQLite DB into an in-memory ZIP archive as `collection.anki2`
        let mut zip_buf: Vec<u8> = Vec::new();
        {
            let cursor = Cursor::new(&mut zip_buf);
            let mut zip = zip::ZipWriter::new(cursor);
            let opts: zip::write::FileOptions<'_, ()> =
                zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("collection.anki2", opts).unwrap();
            zip.write_all(&db_bytes).unwrap();
            zip.finish().unwrap();
        }

        zip_buf
    }

    // ── Tests ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_import_anki_file_too_large() {
        let big = vec![0u8; MAX_FILE_BYTES + 1];
        let result = make_use_case()
            .execute(Uuid::new_v4(), Bytes::from(big))
            .await;
        assert!(matches!(result, Err(AppError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_import_anki_invalid_zip() {
        let result = make_use_case()
            .execute(Uuid::new_v4(), Bytes::from("not a zip at all"))
            .await;
        assert!(matches!(result, Err(AppError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_import_anki_zip_missing_collection() {
        // A valid ZIP but no collection.anki2 / collection.anki21 entry
        let mut zip_buf: Vec<u8> = Vec::new();
        {
            let cursor = Cursor::new(&mut zip_buf);
            let mut zip = zip::ZipWriter::new(cursor);
            let opts: zip::write::FileOptions<'_, ()> =
                zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("media", opts).unwrap();
            zip.write_all(b"{}").unwrap();
            zip.finish().unwrap();
        }
        let result = make_use_case()
            .execute(Uuid::new_v4(), Bytes::from(zip_buf))
            .await;
        assert!(matches!(result, Err(AppError::ValidationError(_))));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_import_anki_happy_path() {
        let notes = vec![
            ("Hello", "Hola"),
            ("World", "Mundo"),
            ("Cat", "Gato"),
        ];
        let apkg = build_test_apkg(&notes, Some("Spanish Basics"));
        let result = make_use_case()
            .execute(Uuid::new_v4(), Bytes::from(apkg))
            .await;
        assert!(result.is_ok(), "{:?}", result.err());
        let r = result.unwrap();
        assert_eq!(r.cards_imported, 3);
        assert_eq!(r.cards_skipped, 0);
        assert_eq!(r.deck_name, "Spanish Basics");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_import_anki_strips_html() {
        let notes = vec![("<b>Bold front</b>", "<i>Italic back</i>")];
        let apkg = build_test_apkg(&notes, None);
        let result = make_use_case()
            .execute(Uuid::new_v4(), Bytes::from(apkg))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().cards_imported, 1);
        // strip_html is also tested directly below
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_import_anki_empty_deck() {
        let apkg = build_test_apkg(&[], Some("Empty"));
        let result = make_use_case()
            .execute(Uuid::new_v4(), Bytes::from(apkg))
            .await;
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.cards_imported, 0);
    }

    #[test]
    fn test_strip_html_removes_tags() {
        assert_eq!(strip_html("<b>Bold</b>"), "Bold");
        assert_eq!(strip_html("<i>Italic</i> text"), "Italic text");
        assert_eq!(strip_html("No tags here"), "No tags here");
        assert_eq!(strip_html("<div><p>Nested</p></div>"), "Nested");
        assert_eq!(strip_html("  <br>  "), "");
    }

    #[test]
    fn test_strip_html_empty_input() {
        assert_eq!(strip_html(""), "");
    }
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

