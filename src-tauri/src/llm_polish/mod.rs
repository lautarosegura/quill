//! Optional LLM-based post-processing. Runs after vocabulary substitution
//! and before injection. Off by default; opt-in per-provider with explicit
//! API keys (one per provider, stored in the OS keychain).
//!
//! Mirrors the `engines/` pattern: a trait + per-provider impls + a
//! dispatcher that picks the active provider at runtime based on `Config`.
//! Lives in its own module instead of `engines/` because the input/output
//! shape is text → text, not audio → text.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::types::LlmProvider;

pub mod anthropic;
pub mod catalog;
pub mod error;
pub mod groq;
pub mod openai;

#[cfg(test)]
mod tests;

pub use error::LlmPolishError;

/// Input to a polish call. Borrows everything so callers don't pay an
/// allocation per dictation.
pub struct PolishRequest<'a> {
    pub text: &'a str,
    pub system_prompt: &'a str,
    pub model: &'a str,
}

/// Result of a polish call. `input_tokens` / `output_tokens` are optional
/// because not every provider's API returns them in a usable form (Groq
/// does, Anthropic does, OpenAI does, but a future provider may not).
#[derive(Debug, Clone)]
pub struct PolishResult {
    pub text: String,
    pub latency_ms: u64,
    pub model: String,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

/// Implemented by every cloud LLM provider.
#[async_trait]
pub trait LlmPolisher: Send + Sync {
    async fn polish(&self, req: PolishRequest<'_>) -> Result<PolishResult, LlmPolishError>;
    fn provider(&self) -> LlmProvider;
}

/// Holds the currently-instantiated provider clients, swapped at runtime as
/// keys come and go. Reads the active provider + model from `Config` on
/// every polish call so changes take effect without restart.
pub struct LlmPolishDispatcher {
    pub groq: RwLock<Option<Arc<groq::GroqLlmPolisher>>>,
    pub anthropic: RwLock<Option<Arc<anthropic::AnthropicLlmPolisher>>>,
    pub openai: RwLock<Option<Arc<openai::OpenaiLlmPolisher>>>,
    pub config: Arc<RwLock<Config>>,
}

impl LlmPolishDispatcher {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self {
            groq: RwLock::new(None),
            anthropic: RwLock::new(None),
            openai: RwLock::new(None),
            config,
        }
    }

    pub async fn set_groq(&self, p: Option<Arc<groq::GroqLlmPolisher>>) {
        *self.groq.write().await = p;
    }
    pub async fn set_anthropic(&self, p: Option<Arc<anthropic::AnthropicLlmPolisher>>) {
        *self.anthropic.write().await = p;
    }
    pub async fn set_openai(&self, p: Option<Arc<openai::OpenaiLlmPolisher>>) {
        *self.openai.write().await = p;
    }

    /// Orchestrator entry point. Polishes `text` via the active provider,
    /// or returns it unchanged when `llm_polish_enabled = false`. Errors
    /// bubble up; the orchestrator logs+degrades, never user-facing.
    ///
    /// Snapshots `Config` once so a Settings change mid-call can't
    /// half-apply.
    pub async fn polish_active(&self, text: &str) -> Result<String, LlmPolishError> {
        if !self.config.read().await.llm_polish_enabled {
            return Ok(text.to_string());
        }
        self.do_polish(text).await.map(|r| r.text)
    }

    /// Preview entry point. Bypasses the enable toggle (the user explicitly
    /// hit "Probar" — clearly they want a result). Returns the full
    /// `PolishResult` so the UI can show latency + token counts.
    pub async fn preview(&self, text: &str) -> Result<PolishResult, LlmPolishError> {
        self.do_polish(text).await
    }

    async fn do_polish(&self, text: &str) -> Result<PolishResult, LlmPolishError> {
        let (provider, system_prompt, model, max_input) = {
            let cfg = self.config.read().await;
            let provider = cfg.llm_polish_provider;
            let model = cfg
                .llm_polish_models
                .get(&provider)
                .cloned()
                .unwrap_or_else(|| catalog::default_model_id(provider).to_string());
            (
                provider,
                cfg.llm_polish_system_prompt.clone(),
                model,
                cfg.llm_polish_max_input_chars as usize,
            )
        };

        // Cheap safety: don't ship 100k char texts to the LLM by accident.
        if text.chars().count() > max_input {
            return Err(LlmPolishError::BadRequest(format!(
                "text exceeds max_input_chars ({} > {})",
                text.chars().count(),
                max_input
            )));
        }

        let req = PolishRequest {
            text,
            system_prompt: &system_prompt,
            model: &model,
        };

        let result = match provider {
            LlmProvider::Groq => {
                let g = self.groq.read().await.clone();
                match g {
                    Some(p) => p.polish(req).await?,
                    None => return Err(LlmPolishError::NotConfigured(provider)),
                }
            }
            LlmProvider::Anthropic => {
                let a = self.anthropic.read().await.clone();
                match a {
                    Some(p) => p.polish(req).await?,
                    None => return Err(LlmPolishError::NotConfigured(provider)),
                }
            }
            LlmProvider::Openai => {
                let o = self.openai.read().await.clone();
                match o {
                    Some(p) => p.polish(req).await?,
                    None => return Err(LlmPolishError::NotConfigured(provider)),
                }
            }
        };

        log::debug!(
            "llm_polish: {} {} → {}ms",
            provider.label(),
            result.model,
            result.latency_ms
        );
        Ok(result)
    }
}
