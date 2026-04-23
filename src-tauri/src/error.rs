use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuillError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("keyring error: {0}")]
    Keyring(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("transcription error: {0}")]
    Transcription(String),

    #[error("injection error: {0}")]
    Injection(String),

    #[error("audio error: {0}")]
    Audio(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

/// Serializable error payload returned to the frontend via Tauri commands.
/// Frontends can branch on `code`; `message` is for display/log.
#[derive(Debug, Serialize)]
pub struct SerializableError {
    pub code: String,
    pub message: String,
}

impl From<QuillError> for SerializableError {
    fn from(e: QuillError) -> Self {
        let code = match &e {
            QuillError::Io(_) => "io",
            QuillError::Json(_) => "json",
            QuillError::Keyring(_) => "keyring",
            QuillError::Config(_) => "config",
            QuillError::Transcription(_) => "transcription",
            QuillError::Injection(_) => "injection",
            QuillError::Audio(_) => "audio",
            QuillError::PermissionDenied(_) => "permission_denied",
            QuillError::NotFound(_) => "not_found",
            QuillError::Other(_) => "other",
        };
        SerializableError {
            code: code.to_string(),
            message: e.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, QuillError>;

/// Richer error taxonomy for transcription engines. Gets converted into
/// `QuillError::Transcription` at the boundary with the orchestrator.
#[derive(Debug, Error)]
pub enum TranscriptionError {
    #[error("engine not configured: {0}")]
    NotConfigured(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("unauthorized: invalid API key")]
    Unauthorized,

    #[error("rate limited")]
    RateLimited,

    #[error("engine timeout after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("engine failed with exit code {code}: {stderr}")]
    SubprocessFailed { code: i32, stderr: String },

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("audio rejected: {0}")]
    AudioRejected(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl From<TranscriptionError> for QuillError {
    fn from(e: TranscriptionError) -> Self {
        QuillError::Transcription(e.to_string())
    }
}
