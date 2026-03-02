use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Import result DTO — returned after TSV or Anki import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub cards_imported: u32,
    pub cards_skipped: u32,
}

/// Anki import result DTO — returned after .apkg import (includes created deck info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnkiImportResult {
    pub deck_id: Uuid,
    pub deck_name: String,
    pub cards_imported: u32,
    pub cards_skipped: u32,
}
