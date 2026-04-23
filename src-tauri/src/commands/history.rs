//! Tauri commands for reading + mutating the history store.

use crate::engines::{TranscriptionEngine, TranscriptionRequest};
use crate::error::{QuillError, SerializableError};
use crate::history::{HistoryEntry, NewHistoryEntry};
use crate::injector::TextInjector;
use crate::post_process;
use crate::types::Engine;
use crate::AppState;

#[tauri::command]
pub async fn list_history(
    state: tauri::State<'_, AppState>,
    limit: i64,
    offset: i64,
) -> Result<Vec<HistoryEntry>, SerializableError> {
    state
        .history
        .recent(limit, offset)
        .map_err(SerializableError::from)
}

#[tauri::command]
pub async fn search_history(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: i64,
) -> Result<Vec<HistoryEntry>, SerializableError> {
    state
        .history
        .search(&query, limit)
        .map_err(SerializableError::from)
}

#[tauri::command]
pub async fn delete_history_entry(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), SerializableError> {
    state.history.delete(id).map_err(SerializableError::from)
}

#[tauri::command]
pub async fn clear_all_history(
    state: tauri::State<'_, AppState>,
) -> Result<(), SerializableError> {
    state.history.clear_all().map_err(SerializableError::from)
}

#[tauri::command]
pub async fn count_history(
    state: tauri::State<'_, AppState>,
) -> Result<i64, SerializableError> {
    state.history.count().map_err(SerializableError::from)
}

#[tauri::command]
pub async fn reinject_history_entry(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), SerializableError> {
    let entry = state
        .history
        .get(id)
        .map_err(SerializableError::from)?
        .ok_or_else(|| {
            SerializableError::from(QuillError::NotFound(format!("entry {id}")))
        })?;
    TextInjector::inject(&entry.text)
        .await
        .map_err(SerializableError::from)
}

/// Re-runs transcription against a preserved failed-WAV, injects the text,
/// and writes a fresh success row. The original failed row stays in history
/// for audit. If the retry also fails, we write a new failed row with its own
/// preserved WAV — the old one is deleted.
#[tauri::command]
pub async fn retry_history_entry(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), SerializableError> {
    let entry = state
        .history
        .get(id)
        .map_err(SerializableError::from)?
        .ok_or_else(|| SerializableError::from(QuillError::NotFound(format!("entry {id}"))))?;

    if entry.status != "failed" {
        return Err(SerializableError::from(QuillError::Other(
            "only failed entries can be retried".into(),
        )));
    }
    let wav_path = entry.failed_wav_path.as_deref().ok_or_else(|| {
        SerializableError::from(QuillError::NotFound(
            "no preserved audio for this entry".into(),
        ))
    })?;

    let wav = std::fs::read(wav_path).map_err(|e| {
        SerializableError::from(QuillError::Io(e))
    })?;

    let (language, vocabulary, engine_choice) = {
        let cfg = state.config.read().await;
        (
            cfg.language,
            if cfg.vocabulary.is_empty() {
                None
            } else {
                Some(cfg.vocabulary.clone())
            },
            cfg.engine,
        )
    };

    let req = TranscriptionRequest {
        audio_wav: &wav,
        language,
        prompt: vocabulary.as_deref(),
    };

    let result = state
        .dispatch
        .transcribe(req)
        .await
        .map_err(|e| SerializableError::from(QuillError::Transcription(e.to_string())))?;

    let text = post_process::clean(&result.text);
    if text.is_empty() {
        return Err(SerializableError::from(QuillError::Other(
            "empty transcription on retry".into(),
        )));
    }

    TextInjector::inject(&text)
        .await
        .map_err(SerializableError::from)?;

    let new_row = NewHistoryEntry {
        engine: match engine_choice {
            Engine::Local => "local".into(),
            Engine::Groq => "groq".into(),
        },
        language: language.code().to_string(),
        model: Some(result.model),
        duration_ms: entry.duration_ms,
        latency_ms: Some(result.latency_ms as i64),
        text,
        status: "success".into(),
        failure_reason: None,
        failed_wav_path: None,
        // Carry over the original source_app so the retry row lines up with
        // the failed row that preceded it.
        source_app: entry.source_app.clone(),
    };
    let _ = state.history.insert(&new_row);

    // Retry succeeded — the preserved WAV is no longer needed.
    let _ = std::fs::remove_file(wav_path);

    Ok(())
}
