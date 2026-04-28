//! Error taxonomy for the LLM polish stage. Mirrors `TranscriptionError`
//! in shape but stays a separate enum because the failure modes are
//! distinct (chat completions vs audio transcription) and we don't want
//! a misclassified error to confuse the orchestrator's dispatch logic.

use thiserror::Error;

use crate::types::LlmProvider;

#[derive(Debug, Error)]
pub enum LlmPolishError {
    #[error("LLM polish: {0:?} provider has no API key configured")]
    NotConfigured(LlmProvider),

    #[error("LLM polish: feature disabled in config")]
    Disabled,

    #[error("LLM polish network error: {0}")]
    Network(String),

    #[error("LLM polish unauthorized: invalid API key")]
    Unauthorized,

    #[error("LLM polish rate limited")]
    RateLimited,

    #[error("LLM polish timeout after {secs}s")]
    Timeout { secs: u64 },

    #[error("LLM polish bad request: {0}")]
    BadRequest(String),

    #[error("LLM polish: {0}")]
    Other(String),
}
