//! GroqEngine — transcription via the Groq cloud API.
//! POSTs multipart audio to /audio/transcriptions.

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;

use super::{TranscriptionEngine, TranscriptionRequest, TranscriptionResult};
use crate::circuit_breaker::CircuitBreaker;
use crate::error::TranscriptionError;

pub mod catalog;

const DEFAULT_BASE_URL: &str = "https://api.groq.com/openai/v1";
pub const DEFAULT_MODEL: &str = "whisper-large-v3-turbo";
const REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct GroqEngine {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub client: reqwest::Client,
    pub breaker: Arc<CircuitBreaker>,
}

impl GroqEngine {
    pub fn new(api_key: String) -> Result<Self, TranscriptionError> {
        Self::new_with_model(api_key, DEFAULT_MODEL.to_string())
    }

    pub fn new_with_model(api_key: String, model: String) -> Result<Self, TranscriptionError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|e| TranscriptionError::Other(format!("http client init: {e}")))?;

        Ok(Self {
            api_key,
            model,
            base_url: DEFAULT_BASE_URL.to_string(),
            client,
            breaker: Arc::new(CircuitBreaker::default_groq()),
        })
    }

    /// Test constructor that lets integration tests inject a mock server URL.
    #[cfg(test)]
    pub fn new_for_test(api_key: String, base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .unwrap();
        Self {
            api_key,
            model: DEFAULT_MODEL.to_string(),
            base_url,
            client,
            // Generous threshold in tests so the breaker doesn't interfere
            // with happy-path + error-mapping tests.
            breaker: Arc::new(CircuitBreaker::new(100, 60, 30)),
        }
    }

    async fn do_transcribe(
        &self,
        req: TranscriptionRequest<'_>,
    ) -> Result<TranscriptionResult, TranscriptionError> {
        let url = format!("{}/audio/transcriptions", self.base_url);

        let file_part = Part::bytes(req.audio_wav.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| TranscriptionError::Other(format!("multipart mime: {e}")))?;

        let mut form = Form::new()
            .part("file", file_part)
            .text("model", self.model.clone())
            .text("language", req.language.code().to_string())
            .text("response_format", "json");

        if let Some(prompt) = req.prompt {
            if !prompt.is_empty() {
                form = form.text("prompt", prompt.to_string());
            }
        }

        let started = Instant::now();
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    TranscriptionError::Timeout {
                        seconds: REQUEST_TIMEOUT_SECS,
                    }
                } else if e.is_connect() {
                    TranscriptionError::Network(format!("connection error: {e}"))
                } else {
                    TranscriptionError::Network(e.to_string())
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => TranscriptionError::Unauthorized,
                402 | 429 => TranscriptionError::RateLimited,
                500..=599 => {
                    TranscriptionError::Network(format!("Groq server error {status}: {body_text}"))
                }
                400 => TranscriptionError::AudioRejected(body_text),
                _ => TranscriptionError::Network(format!("HTTP {status}: {body_text}")),
            });
        }

        #[derive(Deserialize)]
        struct GroqResponse {
            text: String,
        }

        let payload: GroqResponse = response
            .json()
            .await
            .map_err(|e| TranscriptionError::Other(format!("response parse: {e}")))?;

        Ok(TranscriptionResult {
            text: payload.text,
            latency_ms: started.elapsed().as_millis() as u64,
            model: self.model.clone(),
        })
    }
}

#[async_trait]
impl TranscriptionEngine for GroqEngine {
    async fn transcribe(
        &self,
        req: TranscriptionRequest<'_>,
    ) -> Result<TranscriptionResult, TranscriptionError> {
        if self.breaker.allow_request().await.is_err() {
            return Err(TranscriptionError::Network(
                "Groq temporalmente desactivado (circuit breaker). Probá local o esperá 30s."
                    .into(),
            ));
        }

        match self.do_transcribe(req).await {
            Ok(r) => {
                self.breaker.record_success().await;
                Ok(r)
            }
            Err(e) => {
                // Only transient/service errors trip the breaker.
                // User errors (Unauthorized, AudioRejected) do NOT.
                if matches!(
                    e,
                    TranscriptionError::Network(_)
                        | TranscriptionError::Timeout { .. }
                        | TranscriptionError::RateLimited
                ) {
                    self.breaker.record_failure().await;
                }
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests;
