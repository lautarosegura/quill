use async_trait::async_trait;

use crate::error::TranscriptionError;
use crate::types::Language;

pub mod dispatch;
pub mod groq;
pub mod local;

/// Input to every transcription engine.
pub struct TranscriptionRequest<'a> {
    /// WAV bytes: 16 kHz mono f32 PCM (format produced by AudioRecorder).
    pub audio_wav: &'a [u8],
    /// Language hint — passed to whisper so it can skip auto-detect.
    pub language: Language,
    /// Optional initial prompt (custom vocabulary). `None` means no prompt.
    pub prompt: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub latency_ms: u64,
    pub model: String,
}

/// Every concrete engine (LocalEngine, GroqEngine, future ones) implements this.
#[async_trait]
pub trait TranscriptionEngine: Send + Sync {
    async fn transcribe(
        &self,
        req: TranscriptionRequest<'_>,
    ) -> Result<TranscriptionResult, TranscriptionError>;
}
