use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::Result;
use crate::types::{Engine, Keybind, Language, OverlayPosition, PromptPreset, Substitution};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub language: Language,
    pub engine: Engine,
    pub local_model_name: String,
    pub groq_model: String,
    pub hotkey: Keybind,
    pub language_cycle_hotkey: Option<Keybind>,
    pub mic_device: Option<String>,
    pub overlay_position: OverlayPosition,
    pub max_duration_secs: u32,
    pub min_duration_ms: u32,
    pub start_on_boot: bool,
    pub sounds_enabled: bool,
    pub vocabulary: String,
    pub monthly_cost_alert_usd: Option<f64>,
    pub wizard_version: u32,
    /// XDG RemoteDesktop portal restore token (Linux Wayland only). Issued
    /// after the first successful libei paste; presenting it on subsequent
    /// runs lets the compositor skip the consent dialog. `#[serde(default)]`
    /// keeps old configs (which don't have this field) loading cleanly.
    #[serde(default)]
    pub wayland_remotedesktop_token: Option<String>,
    /// Enables Silero VAD pre-processing in `whisper-cli`. Eliminates the
    /// "you" / "thanks for watching" hallucinations Whisper produces from
    /// trailing-silence audio. Default true; toggle exposed in Settings →
    /// Advanced for the rare user who needs to disable it.
    #[serde(default = "default_vad_enabled")]
    pub vad_enabled: bool,
    /// Post-transcription exact-match replacements. Applied AFTER
    /// `post_process::clean` so they catch errors that the Whisper
    /// prompt-biasing in `vocabulary` couldn't fix.
    #[serde(default)]
    pub substitutions: Vec<Substitution>,
    /// Available prompt presets (built-in + user-created). The active one
    /// has its `prompt` concatenated with `vocabulary` and passed to
    /// Whisper at transcribe time. `#[serde(default = ...)]` seeds the
    /// 4 built-ins on first run / on existing configs that pre-date this
    /// field.
    #[serde(default = "default_presets")]
    pub presets: Vec<PromptPreset>,
    /// Currently selected preset ID. `None` means "no preset, use only
    /// the global vocabulary as the Whisper prompt" — preserves the
    /// pre-presets behavior for users who don't opt in.
    #[serde(default)]
    pub active_preset_id: Option<String>,
}

fn default_vad_enabled() -> bool {
    true
}

/// Built-in prompt presets shipped with every fresh install. Order
/// matches what the user sees in the tray submenu / Settings list.
pub fn default_presets() -> Vec<PromptPreset> {
    vec![
        PromptPreset::builtin(
            "general",
            "General",
            "Hola, buen día. Vamos a ver qué tal funciona esto.",
        ),
        PromptPreset::builtin(
            "code",
            "Código",
            "Estoy escribiendo código en Python o Rust. Identificadores en \
             snake_case y camelCase. Términos comunes: async, await, fn, \
             struct, return, import, function, const, let, if, else.",
        ),
        PromptPreset::builtin(
            "email",
            "Email",
            "Estimado equipo, les escribo para hacer seguimiento sobre el \
             tema que conversamos. Por otro lado, quería comentarles. \
             Quedamos en contacto. Saludos cordiales,",
        ),
        PromptPreset::builtin(
            "casual",
            "Casual",
            "Che, mirá, lo que pasa es que. La verdad, no sé bien. Bueno, \
             te cuento. Dale, hablamos después. Posta. Ojalá que sí.",
        ),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Config {
            language: Language::Es,
            engine: Engine::Local,
            local_model_name: "ggml-base".to_string(),
            groq_model: "whisper-large-v3-turbo".to_string(),
            hotkey: Keybind::default_push_to_talk(),
            language_cycle_hotkey: None,
            mic_device: None,
            overlay_position: OverlayPosition::BottomCenter,
            max_duration_secs: 60,
            min_duration_ms: 250,
            start_on_boot: false,
            sounds_enabled: false,
            vocabulary: String::new(),
            monthly_cost_alert_usd: None,
            wizard_version: 0,
            wayland_remotedesktop_token: None,
            vad_enabled: default_vad_enabled(),
            substitutions: Vec::new(),
            presets: default_presets(),
            active_preset_id: None,
        }
    }
}

impl Config {
    /// Compose the final Whisper prompt to pass to the engine: the active
    /// preset's text (if any) plus the global `vocabulary`, truncated
    /// together to a safe character budget.
    ///
    /// Whisper's decoder accepts ~224 tokens of prompt context. We
    /// approximate at the char level (4 chars/token rule of thumb → 880
    /// chars). When both preset and vocabulary are present and the
    /// concatenation overflows, the suffix (vocabulary) is what gets
    /// truncated — preserves the preset's tone/style framing.
    pub fn active_prompt(&self) -> String {
        const MAX_CHARS: usize = 880;
        let preset_prompt: Option<&str> = self.active_preset_id.as_deref().and_then(|id| {
            self.presets
                .iter()
                .find(|p| p.id == id)
                .map(|p| p.prompt.as_str())
        });
        let raw = match (preset_prompt, self.vocabulary.is_empty()) {
            (None, true) => return String::new(),
            (None, false) => self.vocabulary.clone(),
            (Some(p), true) => p.to_string(),
            (Some(p), false) => format!("{} {}", p, self.vocabulary),
        };
        if raw.chars().count() <= MAX_CHARS {
            raw
        } else {
            raw.chars().take(MAX_CHARS).collect()
        }
    }
}

pub struct ConfigStore;

impl ConfigStore {
    pub fn load_from(path: &Path) -> Result<Option<Config>> {
        if !path.exists() {
            return Ok(None);
        }
        let s = std::fs::read_to_string(path)?;
        let cfg: Config = serde_json::from_str(&s)?;
        Ok(Some(cfg))
    }

    pub fn save_to(path: &Path, cfg: &Config) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = serde_json::to_string_pretty(cfg)?;
        std::fs::write(path, s)?;
        Ok(())
    }

    pub fn load() -> Result<Option<Config>> {
        Self::load_from(&crate::paths::config_file())
    }

    pub fn save(cfg: &Config) -> Result<()> {
        Self::save_to(&crate::paths::config_file(), cfg)
    }
}

#[cfg(test)]
mod tests;
