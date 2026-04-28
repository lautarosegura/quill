//! Tauri commands for the LLM polish feature: per-provider API key
//! management, model catalog lookup, and a sample-text preview.
//!
//! Mirrors the shape of `commands/groq.rs`. Each provider has its own
//! keychain slot and its own polisher instance inside the dispatcher; the
//! dispatcher reads `Config.llm_polish_provider` at polish time to pick
//! which one to call.

use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;

use crate::error::{QuillError, SerializableError};
use crate::llm_polish::{
    anthropic::AnthropicLlmPolisher,
    catalog::{self, LlmModelInfo},
    groq::GroqLlmPolisher,
    openai::OpenaiLlmPolisher,
};
use crate::secrets::SecretStore;
use crate::types::LlmProvider;
use crate::AppState;

/// Returns "sk-ab...cd" if a key is stored for `provider`, or None otherwise.
/// Never returns the raw key — frontend only ever sees the mask.
#[tauri::command]
pub async fn get_llm_polish_key_masked(
    provider: LlmProvider,
) -> Result<Option<String>, SerializableError> {
    let key = SecretStore::get_llm_key(provider).map_err(SerializableError::from)?;
    Ok(key.map(|k| mask(&k)))
}

#[tauri::command]
pub async fn set_llm_polish_key(
    state: tauri::State<'_, AppState>,
    provider: LlmProvider,
    key: String,
) -> Result<(), SerializableError> {
    if key.trim().is_empty() {
        return Err(SerializableError::from(QuillError::Other(
            "API key vacía".into(),
        )));
    }
    SecretStore::set_llm_key(provider, key.trim()).map_err(SerializableError::from)?;
    refresh_polisher(&state, provider).await
}

#[tauri::command]
pub async fn delete_llm_polish_key(
    state: tauri::State<'_, AppState>,
    provider: LlmProvider,
) -> Result<(), SerializableError> {
    SecretStore::delete_llm_key(provider).map_err(SerializableError::from)?;
    // Refresh anyway — pulls a None from the keychain and clears the slot.
    refresh_polisher(&state, provider).await
}

/// Returns the static catalog for `provider`. We don't do live `/v1/models`
/// fetches in v0.5.0 — model lists are stable enough across the three
/// providers that hard-coding is fine.
#[tauri::command]
pub fn list_llm_polish_models(provider: LlmProvider) -> Vec<LlmModelInfo> {
    catalog::models_for(provider)
}

/// Pulls the current key from the keychain, builds a fresh polisher, and
/// swaps it into the dispatcher slot. If the key is gone, clears the slot.
/// Idempotent — safe to call whenever the user changes a key.
async fn refresh_polisher(
    state: &tauri::State<'_, AppState>,
    provider: LlmProvider,
) -> Result<(), SerializableError> {
    let key = SecretStore::get_llm_key(provider).map_err(SerializableError::from)?;
    match (provider, key) {
        (LlmProvider::Groq, Some(k)) => {
            let p = GroqLlmPolisher::new(k)
                .map_err(|e| SerializableError::from(QuillError::Other(e.to_string())))?;
            state.polish_dispatch.set_groq(Some(Arc::new(p))).await;
        }
        (LlmProvider::Groq, None) => state.polish_dispatch.set_groq(None).await,
        (LlmProvider::Anthropic, Some(k)) => {
            let p = AnthropicLlmPolisher::new(k)
                .map_err(|e| SerializableError::from(QuillError::Other(e.to_string())))?;
            state.polish_dispatch.set_anthropic(Some(Arc::new(p))).await;
        }
        (LlmProvider::Anthropic, None) => state.polish_dispatch.set_anthropic(None).await,
        (LlmProvider::Openai, Some(k)) => {
            let p = OpenaiLlmPolisher::new(k)
                .map_err(|e| SerializableError::from(QuillError::Other(e.to_string())))?;
            state.polish_dispatch.set_openai(Some(Arc::new(p))).await;
        }
        (LlmProvider::Openai, None) => state.polish_dispatch.set_openai(None).await,
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct LlmKeyTestResult {
    pub valid: bool,
    pub message: String,
}

/// Lightweight key verification — no mutation. Each provider has a
/// different "smallest possible auth check":
///   - Groq: GET /openai/v1/models  (Bearer auth)
///   - OpenAI: GET /v1/models  (Bearer auth)
///   - Anthropic: GET /v1/models  (x-api-key + anthropic-version)
#[tauri::command]
pub async fn test_llm_polish_key(
    provider: LlmProvider,
    key: String,
) -> Result<LlmKeyTestResult, SerializableError> {
    if key.trim().is_empty() {
        return Ok(LlmKeyTestResult {
            valid: false,
            message: "API key vacía".into(),
        });
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| SerializableError::from(QuillError::Other(e.to_string())))?;

    let result = match provider {
        LlmProvider::Groq => {
            client
                .get("https://api.groq.com/openai/v1/models")
                .bearer_auth(key.trim())
                .send()
                .await
        }
        LlmProvider::Openai => {
            client
                .get("https://api.openai.com/v1/models")
                .bearer_auth(key.trim())
                .send()
                .await
        }
        LlmProvider::Anthropic => {
            client
                .get("https://api.anthropic.com/v1/models")
                .header("x-api-key", key.trim())
                .header("anthropic-version", "2023-06-01")
                .send()
                .await
        }
    };

    match result {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                Ok(LlmKeyTestResult {
                    valid: true,
                    message: "Clave válida".into(),
                })
            } else if status.as_u16() == 401 {
                Ok(LlmKeyTestResult {
                    valid: false,
                    message: "API key inválida".into(),
                })
            } else {
                Ok(LlmKeyTestResult {
                    valid: false,
                    message: format!("Respuesta inesperada: HTTP {status}"),
                })
            }
        }
        Err(e) => Ok(LlmKeyTestResult {
            valid: false,
            message: format!("Error de red: {e}"),
        }),
    }
}

#[derive(Debug, Serialize)]
pub struct PolishPreview {
    pub original: String,
    pub polished: String,
    pub latency_ms: u64,
    pub model: String,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

/// Runs a polish call with the user's currently-saved provider, model, and
/// system prompt. Bypasses the global `llm_polish_enabled` toggle — the
/// preview is for testing settings, not for production polish.
#[tauri::command]
pub async fn test_llm_polish(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<PolishPreview, SerializableError> {
    let result = state
        .polish_dispatch
        .preview(&text)
        .await
        .map_err(|e| SerializableError::from(QuillError::Other(e.to_string())))?;

    Ok(PolishPreview {
        original: text,
        polished: result.text,
        latency_ms: result.latency_ms,
        model: result.model,
        input_tokens: result.input_tokens,
        output_tokens: result.output_tokens,
    })
}

fn mask(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 8 {
        return format!("{} chars", chars.len());
    }
    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars
        .iter()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{prefix}...{suffix}")
}
