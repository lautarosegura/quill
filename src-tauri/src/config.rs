use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::Result;
use crate::types::{Engine, Keybind, Language, OverlayPosition, Substitution};

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
}

fn default_vad_enabled() -> bool {
    true
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
