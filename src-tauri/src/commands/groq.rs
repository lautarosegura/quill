//! Commands for managing the Groq API key + live engine swap + model catalog.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::engines::groq::catalog::{self, GroqModelInfo};
use crate::engines::groq::GroqEngine;
use crate::error::{QuillError, SerializableError};
use crate::secrets::SecretStore;
use crate::AppState;

/// Returns "sk-ab...cd" if a key is stored, or None if unset.
/// Never returns the raw key — frontend only ever sees the mask.
#[tauri::command]
pub async fn get_groq_key_masked() -> Result<Option<String>, SerializableError> {
    let key = SecretStore::get_groq_key().map_err(SerializableError::from)?;
    Ok(key.map(|k| mask(&k)))
}

#[tauri::command]
pub async fn set_groq_key(
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<(), SerializableError> {
    if key.trim().is_empty() {
        return Err(SerializableError::from(QuillError::Other(
            "API key vacía".into(),
        )));
    }
    SecretStore::set_groq_key(key.trim()).map_err(SerializableError::from)?;
    refresh_engine_from_state(&state).await?;
    Ok(())
}

/// Reads the current API key (from keychain) and model (from config) and
/// rebuilds the GroqEngine inside DispatchingEngine. If no key, clears it.
/// Idempotent — safe to call whenever the user changes key or model.
async fn refresh_engine_from_state(
    state: &tauri::State<'_, AppState>,
) -> Result<(), SerializableError> {
    let key = SecretStore::get_groq_key().map_err(SerializableError::from)?;
    let Some(key) = key else {
        state.dispatch.set_groq(None).await;
        return Ok(());
    };
    let model = state.config.read().await.groq_model.clone();
    let engine = GroqEngine::new_with_model(key, model)
        .map_err(|e| SerializableError::from(QuillError::Other(e.to_string())))?;
    state.dispatch.set_groq(Some(Arc::new(engine))).await;
    Ok(())
}

#[tauri::command]
pub async fn refresh_groq_engine(
    state: tauri::State<'_, AppState>,
) -> Result<(), SerializableError> {
    refresh_engine_from_state(&state).await
}

#[tauri::command]
pub async fn set_groq_model(
    state: tauri::State<'_, AppState>,
    model: String,
) -> Result<(), SerializableError> {
    // Update config first so future reads see the new value.
    {
        let mut cfg = state.config.write().await;
        cfg.groq_model = model;
        // Persist to disk.
        crate::config::ConfigStore::save(&cfg).map_err(SerializableError::from)?;
    }
    refresh_engine_from_state(&state).await
}

#[tauri::command]
pub async fn delete_groq_key(state: tauri::State<'_, AppState>) -> Result<(), SerializableError> {
    SecretStore::delete_groq_key().map_err(SerializableError::from)?;
    state.dispatch.set_groq(None).await;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct GroqTestResult {
    pub valid: bool,
    pub message: String,
}

/// Lightweight key verification: GET /models with the provided key.
/// Does NOT mutate anything (doesn't save the key, doesn't swap engines).
#[tauri::command]
pub async fn test_groq_key(key: String) -> Result<GroqTestResult, SerializableError> {
    if key.trim().is_empty() {
        return Ok(GroqTestResult {
            valid: false,
            message: "API key vacía".into(),
        });
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| SerializableError::from(QuillError::Other(e.to_string())))?;

    let result = client
        .get("https://api.groq.com/openai/v1/models")
        .bearer_auth(key.trim())
        .send()
        .await;

    match result {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                Ok(GroqTestResult {
                    valid: true,
                    message: "Clave válida".into(),
                })
            } else if status.as_u16() == 401 {
                Ok(GroqTestResult {
                    valid: false,
                    message: "API key inválida".into(),
                })
            } else {
                Ok(GroqTestResult {
                    valid: false,
                    message: format!("Respuesta inesperada de Groq: HTTP {status}"),
                })
            }
        }
        Err(e) => Ok(GroqTestResult {
            valid: false,
            message: format!("Error de red: {e}"),
        }),
    }
}

fn mask(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 8 {
        // Too short to mask meaningfully — just show length.
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

// ── Groq model listing (live + static catalog merge) ───────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroqModelKind {
    /// In our static catalog AND present in the live `/v1/models` response.
    Verified,
    /// In the catalog but we couldn't verify live (no key, or API down).
    CatalogOnly,
    /// In the live response but NOT in our catalog (Groq added it recently).
    NewUnknown,
}

#[derive(Debug, Serialize)]
pub struct GroqModelEntry {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub cost_per_hour_usd: Option<f64>,
    pub languages: Vec<String>,
    pub kind: GroqModelKind,
}

#[derive(Debug, Serialize)]
pub struct GroqModelsResult {
    /// True if we got a valid response from Groq's /models endpoint.
    pub live_check_succeeded: bool,
    /// Populated on failure — for UI banners.
    pub error: Option<String>,
    pub models: Vec<GroqModelEntry>,
}

fn catalog_to_entry(info: &GroqModelInfo, kind: GroqModelKind) -> GroqModelEntry {
    GroqModelEntry {
        name: info.name.to_string(),
        display_name: info.display_name.to_string(),
        description: info.description.to_string(),
        cost_per_hour_usd: Some(info.cost_per_hour_usd),
        languages: info.languages.iter().map(|s| s.to_string()).collect(),
        kind,
    }
}

/// Is this a Whisper-family transcription model? Groq's /models returns
/// everything (LLMs included), so we filter by ID prefix.
fn is_whisper_id(id: &str) -> bool {
    let lower = id.to_ascii_lowercase();
    lower.starts_with("whisper-")
        || lower.starts_with("distil-whisper")
        || lower.contains("whisper")
}

#[derive(Deserialize)]
struct GroqModelsApiResponse {
    data: Vec<GroqModelsApiEntry>,
}

#[derive(Deserialize)]
struct GroqModelsApiEntry {
    id: String,
}

/// Lists Groq Whisper models merging live API response with static catalog.
/// If no API key or live call fails, returns the full catalog with `live_check_succeeded = false`.
#[tauri::command]
pub async fn list_groq_models() -> Result<GroqModelsResult, SerializableError> {
    let key = SecretStore::get_groq_key().map_err(SerializableError::from)?;
    let Some(key) = key else {
        return Ok(GroqModelsResult {
            live_check_succeeded: false,
            error: Some("Sin clave API configurada — mostrando catálogo estático.".into()),
            models: catalog::KNOWN_GROQ_MODELS
                .iter()
                .map(|m| catalog_to_entry(m, GroqModelKind::CatalogOnly))
                .collect(),
        });
    };

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return Ok(fallback_catalog(format!("http client: {e}")));
        }
    };

    let response = match client
        .get("https://api.groq.com/openai/v1/models")
        .bearer_auth(&key)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Ok(fallback_catalog(format!("network: {e}"))),
    };

    if !response.status().is_success() {
        return Ok(fallback_catalog(format!("Groq HTTP {}", response.status())));
    }

    let parsed: GroqModelsApiResponse = match response.json().await {
        Ok(p) => p,
        Err(e) => return Ok(fallback_catalog(format!("parse: {e}"))),
    };

    let live_whisper_ids: Vec<String> = parsed
        .data
        .into_iter()
        .map(|e| e.id)
        .filter(|id| is_whisper_id(id))
        .collect();

    let mut models: Vec<GroqModelEntry> = Vec::new();

    // For each catalog entry: Verified if the live response includes it; else drop.
    for info in catalog::KNOWN_GROQ_MODELS {
        if live_whisper_ids.iter().any(|id| id == info.name) {
            models.push(catalog_to_entry(info, GroqModelKind::Verified));
        }
    }

    // For live entries not in catalog: add as NewUnknown with minimal info.
    for id in &live_whisper_ids {
        if catalog::find(id).is_none() {
            models.push(GroqModelEntry {
                name: id.clone(),
                display_name: id.clone(),
                description: "Nuevo en Groq — sin info extra en nuestro catálogo.".into(),
                cost_per_hour_usd: None,
                languages: vec![],
                kind: GroqModelKind::NewUnknown,
            });
        }
    }

    Ok(GroqModelsResult {
        live_check_succeeded: true,
        error: None,
        models,
    })
}

fn fallback_catalog(reason: String) -> GroqModelsResult {
    GroqModelsResult {
        live_check_succeeded: false,
        error: Some(format!(
            "No pudimos verificar con Groq ({reason}) — catálogo estático."
        )),
        models: catalog::KNOWN_GROQ_MODELS
            .iter()
            .map(|m| catalog_to_entry(m, GroqModelKind::CatalogOnly))
            .collect(),
    }
}
