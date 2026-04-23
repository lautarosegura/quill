use super::*;
use crate::error::QuillError;
use tempfile::tempdir;

#[test]
fn default_config_has_expected_values() {
    let c = Config::default();
    assert_eq!(c.language, Language::Es);
    assert_eq!(c.engine, Engine::Local);
    assert_eq!(c.local_model_name, "ggml-base");
    assert_eq!(c.groq_model, "whisper-large-v3-turbo");
    assert_eq!(c.hotkey, Keybind::default_push_to_talk());
    assert_eq!(c.overlay_position, OverlayPosition::BottomCenter);
    assert_eq!(c.max_duration_secs, 60);
    assert_eq!(c.min_duration_ms, 250);
    assert!(!c.start_on_boot);
    assert!(!c.sounds_enabled);
    assert_eq!(c.vocabulary, "");
    assert_eq!(c.monthly_cost_alert_usd, None);
    assert_eq!(c.wizard_version, 0);
    assert_eq!(c.mic_device, None);
    assert_eq!(c.language_cycle_hotkey, None);
}

#[test]
fn config_round_trips_through_json() {
    let original = Config::default();
    let j = serde_json::to_string_pretty(&original).unwrap();
    let restored: Config = serde_json::from_str(&j).unwrap();
    assert_eq!(original.language, restored.language);
    assert_eq!(original.engine, restored.engine);
    assert_eq!(original.hotkey, restored.hotkey);
}

#[test]
fn load_missing_file_returns_none() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nope.json");
    let result = ConfigStore::load_from(&path).unwrap();
    assert!(result.is_none());
}

#[test]
fn save_and_load_round_trip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.json");

    let c = Config {
        language: Language::En,
        vocabulary: "Tauri, Svelte, Rust".to_string(),
        ..Config::default()
    };

    ConfigStore::save_to(&path, &c).unwrap();
    let loaded = ConfigStore::load_from(&path).unwrap().unwrap();

    assert_eq!(loaded.language, Language::En);
    assert_eq!(loaded.vocabulary, "Tauri, Svelte, Rust");
}

#[test]
fn load_corrupt_file_returns_json_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, "{{{not json").unwrap();

    let result = ConfigStore::load_from(&path);
    assert!(matches!(result, Err(QuillError::Json(_))));
}

#[test]
fn save_creates_parent_dir_if_missing() {
    let dir = tempdir().unwrap();
    let nested = dir.path().join("a/b/c/config.json");
    let c = Config::default();
    ConfigStore::save_to(&nested, &c).unwrap();
    assert!(nested.exists());
}
