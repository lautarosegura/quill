//! HistoryStore — persists every dictation to ~/.quill/history.db via rusqlite.
//!
//! Owned end-to-end by the backend; the frontend never talks to sqlite directly.
//! Commands return already-shaped `HistoryEntry` structs.

use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::QuillError;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS transcriptions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at      TEXT    NOT NULL,
    engine          TEXT    NOT NULL CHECK (engine IN ('local', 'groq')),
    language        TEXT    NOT NULL,
    model           TEXT,
    duration_ms     INTEGER,
    latency_ms      INTEGER,
    text            TEXT    NOT NULL,
    status          TEXT    NOT NULL CHECK (status IN ('success', 'failed')) DEFAULT 'success',
    failure_reason  TEXT,
    failed_wav_path TEXT,
    source_app      TEXT
);
CREATE INDEX IF NOT EXISTS idx_created_at ON transcriptions(created_at DESC);
"#;

/// Additive migrations applied after [`SCHEMA`]. Each statement must be
/// idempotent; we ignore errors because SQLite's ALTER TABLE ADD COLUMN
/// fails when the column already exists and there's no IF NOT EXISTS guard
/// for that op. Keep the list short and append-only.
const MIGRATIONS: &[&str] = &["ALTER TABLE transcriptions ADD COLUMN source_app TEXT"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub created_at: String, // ISO-8601 UTC
    pub engine: String,
    pub language: String,
    pub model: Option<String>,
    pub duration_ms: Option<i64>,
    pub latency_ms: Option<i64>,
    pub text: String,
    pub status: String,
    pub failure_reason: Option<String>,
    pub failed_wav_path: Option<String>,
    /// Foreground-window title captured when the user pressed the hotkey.
    /// Windows only today; `None` elsewhere.
    pub source_app: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewHistoryEntry {
    pub engine: String,
    pub language: String,
    pub model: Option<String>,
    pub duration_ms: Option<i64>,
    pub latency_ms: Option<i64>,
    pub text: String,
    pub status: String,
    pub failure_reason: Option<String>,
    pub failed_wav_path: Option<String>,
    pub source_app: Option<String>,
}

pub struct HistoryStore {
    conn: Arc<Mutex<Connection>>,
}

impl HistoryStore {
    pub fn open(path: &Path) -> Result<Self, QuillError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn =
            Connection::open(path).map_err(|e| QuillError::Other(format!("sqlite open: {e}")))?;
        conn.execute_batch(SCHEMA)
            .map_err(|e| QuillError::Other(format!("sqlite schema: {e}")))?;

        // Apply additive migrations. Each failure is silently tolerated
        // because "column already exists" is the common case on upgrades.
        for stmt in MIGRATIONS {
            let _ = conn.execute(stmt, []);
        }

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn insert(&self, entry: &NewHistoryEntry) -> Result<i64, QuillError> {
        let now: DateTime<Utc> = Utc::now();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO transcriptions
             (created_at, engine, language, model, duration_ms, latency_ms,
              text, status, failure_reason, failed_wav_path, source_app)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                now.to_rfc3339(),
                entry.engine,
                entry.language,
                entry.model,
                entry.duration_ms,
                entry.latency_ms,
                entry.text,
                entry.status,
                entry.failure_reason,
                entry.failed_wav_path,
                entry.source_app,
            ],
        )
        .map_err(|e| QuillError::Other(format!("sqlite insert: {e}")))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn recent(&self, limit: i64, offset: i64) -> Result<Vec<HistoryEntry>, QuillError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, created_at, engine, language, model, duration_ms, latency_ms,
                        text, status, failure_reason, failed_wav_path, source_app
                 FROM transcriptions
                 ORDER BY created_at DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e| QuillError::Other(format!("sqlite prepare: {e}")))?;
        let rows = stmt
            .query_map(params![limit, offset], row_to_entry)
            .map_err(|e| QuillError::Other(format!("sqlite query: {e}")))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| QuillError::Other(format!("sqlite row: {e}")))
    }

    pub fn search(&self, query: &str, limit: i64) -> Result<Vec<HistoryEntry>, QuillError> {
        let conn = self.conn.lock().unwrap();
        let like = format!("%{query}%");
        let mut stmt = conn
            .prepare(
                "SELECT id, created_at, engine, language, model, duration_ms, latency_ms,
                        text, status, failure_reason, failed_wav_path, source_app
                 FROM transcriptions
                 WHERE text LIKE ?1 COLLATE NOCASE
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| QuillError::Other(format!("sqlite prepare: {e}")))?;
        let rows = stmt
            .query_map(params![like, limit], row_to_entry)
            .map_err(|e| QuillError::Other(format!("sqlite query: {e}")))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| QuillError::Other(format!("sqlite row: {e}")))
    }

    pub fn get(&self, id: i64) -> Result<Option<HistoryEntry>, QuillError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, created_at, engine, language, model, duration_ms, latency_ms,
                        text, status, failure_reason, failed_wav_path, source_app
                 FROM transcriptions WHERE id = ?1",
            )
            .map_err(|e| QuillError::Other(format!("sqlite prepare: {e}")))?;
        let mut rows = stmt
            .query_map(params![id], row_to_entry)
            .map_err(|e| QuillError::Other(format!("sqlite query: {e}")))?;
        match rows.next() {
            Some(r) => Ok(Some(r.map_err(|e| QuillError::Other(e.to_string()))?)),
            None => Ok(None),
        }
    }

    pub fn delete(&self, id: i64) -> Result<(), QuillError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM transcriptions WHERE id = ?1", params![id])
            .map_err(|e| QuillError::Other(format!("sqlite delete: {e}")))?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<(), QuillError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM transcriptions", [])
            .map_err(|e| QuillError::Other(format!("sqlite clear: {e}")))?;
        Ok(())
    }

    pub fn count(&self) -> Result<i64, QuillError> {
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM transcriptions", [], |r| r.get(0))
            .map_err(|e| QuillError::Other(format!("sqlite count: {e}")))?;
        Ok(n)
    }

    /// Exposes the raw pooled connection for cross-module queries (usage.rs).
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryEntry> {
    Ok(HistoryEntry {
        id: row.get(0)?,
        created_at: row.get(1)?,
        engine: row.get(2)?,
        language: row.get(3)?,
        model: row.get(4)?,
        duration_ms: row.get(5)?,
        latency_ms: row.get(6)?,
        text: row.get(7)?,
        status: row.get(8)?,
        failure_reason: row.get(9)?,
        failed_wav_path: row.get(10)?,
        source_app: row.get(11)?,
    })
}

#[cfg(test)]
mod tests;
