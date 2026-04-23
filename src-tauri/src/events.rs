//! IPC event types emitted by the backend to the frontend.

use serde::{Deserialize, Serialize};

pub const TRANSCRIPTION_STATE_CHANGED: &str = "transcription_state_changed";
/// Fired after every history-row insert so the Historial UI (and sidebar
/// count) can refresh in real time without polling.
pub const HISTORY_CHANGED: &str = "history_changed";
pub const MODEL_DOWNLOAD_PROGRESS: &str = "model_download_progress";
pub const MODEL_DOWNLOAD_COMPLETE: &str = "model_download_complete";
pub const MODEL_DOWNLOAD_ERROR: &str = "model_download_error";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum TranscriptionState {
    Idle,
    Recording,
    Transcribing,
    Injecting,
    /// User aborted a locked/in-progress recording via Escape. The audio is
    /// discarded. The overlay shows this briefly then auto-transitions to Idle.
    Cancelled,
    /// Wayland compositor denied programmatic input injection so Quill copied
    /// the text to the clipboard instead. The UI surfaces a "press Ctrl+V to
    /// paste" toast; auto-transitions to Idle after a short window.
    #[serde(rename = "clipboard-only")]
    ClipboardOnly {
        text_len: usize,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDownloadProgress {
    pub name: String,
    pub downloaded: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDownloadComplete {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDownloadError {
    pub name: String,
    pub message: String,
}
