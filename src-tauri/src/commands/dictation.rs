use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::events::{TranscriptionState, TRANSCRIPTION_STATE_CHANGED};

/// Emits a scripted sequence of state events so the frontend overlay can be
/// tested without real audio. Debug/dev helper.
#[tauri::command]
pub async fn trigger_test_dictation(app: AppHandle) -> Result<(), String> {
    let _ = app.emit(TRANSCRIPTION_STATE_CHANGED, TranscriptionState::Recording);
    tokio::time::sleep(Duration::from_secs(2)).await;
    let _ = app.emit(
        TRANSCRIPTION_STATE_CHANGED,
        TranscriptionState::Transcribing,
    );
    tokio::time::sleep(Duration::from_secs(1)).await;
    let _ = app.emit(TRANSCRIPTION_STATE_CHANGED, TranscriptionState::Idle);
    Ok(())
}
