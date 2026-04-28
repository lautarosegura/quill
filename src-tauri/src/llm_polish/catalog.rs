//! Static catalog of LLM models we know about, per provider. No live
//! `/v1/models` fetch in v0.5.0 — model lists are stable enough across
//! the three providers that a hard-coded list is fine and avoids an
//! extra round-trip on every Settings load.
//!
//! Update entries here when a provider's pricing/quality balance shifts.

use serde::Serialize;

use crate::types::LlmProvider;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LlmModelInfo {
    pub provider: LlmProvider,
    /// API model identifier (sent in the request body).
    pub id: &'static str,
    /// UI label.
    pub display_name: &'static str,
    /// One-line description shown next to / below the dropdown.
    pub blurb: &'static str,
    /// Marks the recommended default for this provider. Exactly one model
    /// per provider should have this set to true.
    pub recommended: bool,
}

pub const KNOWN_LLM_MODELS: &[LlmModelInfo] = &[
    // ── Groq ────────────────────────────────────────────────────────────
    LlmModelInfo {
        provider: LlmProvider::Groq,
        id: "llama-3.3-70b-versatile",
        display_name: "Llama 3.3 70B",
        blurb: "Rápido y preciso. Recomendado.",
        recommended: true,
    },
    LlmModelInfo {
        provider: LlmProvider::Groq,
        id: "llama-3.1-8b-instant",
        display_name: "Llama 3.1 8B Instant",
        blurb: "Ultra-rápido pero menos preciso.",
        recommended: false,
    },
    LlmModelInfo {
        provider: LlmProvider::Groq,
        id: "mixtral-8x7b-32768",
        display_name: "Mixtral 8×7B",
        blurb: "Alternativa con contexto largo.",
        recommended: false,
    },
    // ── Anthropic ───────────────────────────────────────────────────────
    LlmModelInfo {
        provider: LlmProvider::Anthropic,
        id: "claude-haiku-4-5-20251001",
        display_name: "Claude Haiku 4.5",
        blurb: "Rápido y barato. Recomendado.",
        recommended: true,
    },
    LlmModelInfo {
        provider: LlmProvider::Anthropic,
        id: "claude-sonnet-4-6",
        display_name: "Claude Sonnet 4.6",
        blurb: "Más capaz, ~3× el costo.",
        recommended: false,
    },
    LlmModelInfo {
        provider: LlmProvider::Anthropic,
        id: "claude-opus-4-7",
        display_name: "Claude Opus 4.7",
        blurb: "Máxima calidad, ~15× el costo.",
        recommended: false,
    },
    // ── OpenAI ──────────────────────────────────────────────────────────
    LlmModelInfo {
        provider: LlmProvider::Openai,
        id: "gpt-4o-mini",
        display_name: "GPT-4o mini",
        blurb: "Rápido y barato. Recomendado.",
        recommended: true,
    },
    LlmModelInfo {
        provider: LlmProvider::Openai,
        id: "gpt-4o",
        display_name: "GPT-4o",
        blurb: "Más capaz, ~10× el costo.",
        recommended: false,
    },
];

pub fn models_for(provider: LlmProvider) -> Vec<LlmModelInfo> {
    KNOWN_LLM_MODELS
        .iter()
        .filter(|m| m.provider == provider)
        .copied()
        .collect()
}

/// Fallback model id when `Config.llm_polish_models` has no entry for a
/// provider. Matches the `recommended: true` model in the catalog above.
pub fn default_model_id(provider: LlmProvider) -> &'static str {
    match provider {
        LlmProvider::Groq => "llama-3.3-70b-versatile",
        LlmProvider::Anthropic => "claude-haiku-4-5-20251001",
        LlmProvider::Openai => "gpt-4o-mini",
    }
}

pub fn find_model(id: &str) -> Option<&'static LlmModelInfo> {
    KNOWN_LLM_MODELS.iter().find(|m| m.id == id)
}
