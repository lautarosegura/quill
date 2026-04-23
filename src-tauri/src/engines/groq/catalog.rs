//! Static catalog of Groq Whisper models — our curated info (display name,
//! description, price, language support). Groq's /v1/models endpoint only
//! gives IDs, so we enrich via this table.

pub struct GroqModelInfo {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub cost_per_hour_usd: f64,
    /// "multilingual" means ES + EN + many others.
    pub languages: &'static [&'static str],
}

pub const KNOWN_GROQ_MODELS: &[GroqModelInfo] = &[
    GroqModelInfo {
        name: "whisper-large-v3-turbo",
        display_name: "Turbo",
        description: "El más rápido y económico. Multilingüe. Recomendado para dictado.",
        cost_per_hour_usd: 0.04,
        languages: &["multilingual"],
    },
    GroqModelInfo {
        name: "whisper-large-v3",
        display_name: "Large v3",
        description: "Máxima precisión. Más caro y algo más lento.",
        cost_per_hour_usd: 0.11,
        languages: &["multilingual"],
    },
    GroqModelInfo {
        name: "distil-whisper-large-v3-en",
        display_name: "Distil EN",
        description: "Sólo inglés. El más barato. No sirve para español.",
        cost_per_hour_usd: 0.02,
        languages: &["en"],
    },
];

pub fn find(name: &str) -> Option<&'static GroqModelInfo> {
    KNOWN_GROQ_MODELS.iter().find(|m| m.name == name)
}

/// Cost per hour for a known model, or `None` if unknown.
/// Used by usage aggregation for cost estimation.
pub fn cost_per_hour_for(name: &str) -> Option<f64> {
    find(name).map(|m| m.cost_per_hour_usd)
}
