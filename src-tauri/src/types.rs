use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Es,
    En,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::Es => "es",
            Language::En => "en",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Language::Es => "Español",
            Language::En => "English",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Local,
    Groq,
}

/// Cloud LLM providers used by the optional post-transcription polish stage.
/// Each provider has its own API key (stored in the OS keychain) and its
/// own preferred model (stored in `Config.llm_polish_models`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    Groq,
    Anthropic,
    Openai,
}

impl LlmProvider {
    /// Keychain account name used to store this provider's API key. Each
    /// provider gets its own slot — picking one provider doesn't reveal
    /// keys for the others, and revoking one key doesn't affect the rest.
    pub fn key_id(&self) -> &'static str {
        match self {
            LlmProvider::Groq => "groq_llm_key",
            LlmProvider::Anthropic => "anthropic_llm_key",
            LlmProvider::Openai => "openai_llm_key",
        }
    }

    /// Human-facing label for UI summaries.
    pub fn label(&self) -> &'static str {
        match self {
            LlmProvider::Groq => "Groq",
            LlmProvider::Anthropic => "Anthropic",
            LlmProvider::Openai => "OpenAI",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OverlayPosition {
    BottomCenter,
    BottomLeft,
    BottomRight,
    TopCenter,
    TopLeft,
    TopRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Meta,
}

/// Global push-to-talk hotkey.
///
/// Can be either a "modifiers + trigger" combo (e.g. Ctrl+Shift+Space) or
/// a modifier-only chord (e.g. Ctrl+Win). For the modifier-only case `key`
/// is `None` and the hotkey activates the moment every required modifier
/// is held; see [`crate::hotkey`] for the event-suppression we do on
/// Windows to prevent the Start menu from popping on Win release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keybind {
    pub modifiers: Vec<Modifier>,
    pub key: Option<String>,
}

impl Keybind {
    pub fn default_push_to_talk() -> Self {
        #[cfg(target_os = "windows")]
        {
            // Windows: modifier-only chord that feels one-handed and avoids
            // clashing with most app shortcuts. Relies on the grab-based
            // event suppression in hotkey.rs to stop the Start menu.
            Keybind {
                modifiers: vec![Modifier::Ctrl, Modifier::Meta],
                key: None,
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Keybind {
                modifiers: vec![Modifier::Ctrl, Modifier::Shift],
                key: Some("Space".to_string()),
            }
        }
    }
}

/// Post-transcription exact-match replacement. Vocabulary as a Whisper
/// prompt biases the decoder but doesn't always overcome the model's
/// stronger priors — `from` ends up transcribed wrong consistently. A
/// substitution catches those persistent errors with a regex
/// word-boundary replace AFTER the transcription comes back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Substitution {
    /// Pattern to find in the transcription. Wrapped in `\b...\b` (word
    /// boundaries) at apply time so e.g. "ai" doesn't replace inside
    /// "rain". Special regex chars are escaped automatically.
    pub from: String,
    /// Replacement string. Inserted verbatim — no regex backrefs.
    pub to: String,
    /// Case-sensitive matching. Default false (i.e. "Mokia" / "mokia"
    /// both match a `from = "mokia"` rule).
    #[serde(default)]
    pub case_sensitive: bool,
}

/// User-switchable Whisper prompt preset. Lets the user bias transcription
/// for different contexts ("Email" → formal punctuation, "Código" →
/// snake_case identifiers, "Casual" → contractions and lunfardo) without
/// editing the global vocabulary every time.
///
/// At transcribe time the active preset's `prompt` is concatenated with
/// `Config::vocabulary` (truncated together to ~220 tokens). They coexist
/// rather than replace each other: preset = tone, vocabulary = words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptPreset {
    /// Stable identifier ("general", "code", "email", "casual"). Used as
    /// `Config::active_preset_id` and as the key for tray-menu lookups.
    pub id: String,
    /// User-visible label ("Modo email").
    pub name: String,
    /// The Whisper-decoder prompt — usually 1-3 sentences of the target
    /// style + a handful of representative words.
    pub prompt: String,
    /// Built-in (shipped with the app, can be edited but not deleted)
    /// vs user-created (full lifecycle).
    #[serde(default)]
    pub builtin: bool,
}

impl PromptPreset {
    pub fn builtin(id: &str, name: &str, prompt: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            prompt: prompt.to_string(),
            builtin: true,
        }
    }
}

#[cfg(test)]
mod tests;
