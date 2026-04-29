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
    assert_eq!(c.min_duration_ms, 400);
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

#[test]
fn default_config_seeds_four_builtin_presets() {
    let c = Config::default();
    assert_eq!(c.presets.len(), 4);
    let ids: Vec<&str> = c.presets.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(ids, vec!["general", "code", "email", "casual"]);
    assert!(c.presets.iter().all(|p| p.builtin));
    assert_eq!(c.active_preset_id, None);
}

#[test]
fn active_prompt_empty_when_no_preset_no_vocab() {
    let c = Config::default(); // active_preset_id=None, vocabulary=""
    assert_eq!(c.active_prompt(), "");
}

#[test]
fn active_prompt_uses_vocab_only_when_no_preset_active() {
    let c = Config {
        vocabulary: "Quill, Tauri".to_string(),
        ..Config::default()
    };
    assert_eq!(c.active_prompt(), "Quill, Tauri");
}

#[test]
fn active_prompt_uses_preset_only_when_no_vocab() {
    let c = Config {
        active_preset_id: Some("code".to_string()),
        vocabulary: String::new(),
        ..Config::default()
    };
    let p = c.active_prompt();
    assert!(p.contains("snake_case")); // built-in code preset content
    assert!(!p.is_empty());
}

#[test]
fn active_prompt_concatenates_preset_then_vocab() {
    let c = Config {
        active_preset_id: Some("code".to_string()),
        vocabulary: "Quill, Tauri".to_string(),
        ..Config::default()
    };
    let p = c.active_prompt();
    assert!(p.starts_with("Estoy escribiendo código")); // preset starts here
    assert!(p.ends_with("Quill, Tauri")); // vocabulary appended at end
}

#[test]
fn active_prompt_truncates_to_880_chars() {
    let c = Config {
        active_preset_id: Some("code".to_string()),
        vocabulary: "X".repeat(2000), // wildly oversized
        ..Config::default()
    };
    let p = c.active_prompt();
    assert_eq!(p.chars().count(), 880);
    // Preset text is preserved (it's at the start, vocabulary suffix
    // gets truncated because we concat preset-then-vocab).
    assert!(p.starts_with("Estoy escribiendo código"));
}

#[test]
fn active_prompt_unknown_preset_id_falls_back_to_vocab_only() {
    let c = Config {
        active_preset_id: Some("nonexistent".to_string()),
        vocabulary: "Quill".to_string(),
        ..Config::default()
    };
    assert_eq!(c.active_prompt(), "Quill");
}

#[test]
fn config_with_unknown_fields_in_json_loads_via_defaults() {
    // Old configs (pre-presets) won't have `presets` / `active_preset_id`
    // fields. `#[serde(default = "default_presets")]` should hydrate them.
    let dir = tempdir().unwrap();
    let path = dir.path().join("legacy.json");
    let legacy = serde_json::json!({
        "language": "es",
        "engine": "local",
        "local_model_name": "ggml-base",
        "groq_model": "whisper-large-v3-turbo",
        "hotkey": { "modifiers": ["ctrl", "shift"], "key": "Space" },
        "language_cycle_hotkey": null,
        "mic_device": null,
        "overlay_position": "bottom-center",
        "max_duration_secs": 60,
        "min_duration_ms": 250,
        "start_on_boot": false,
        "sounds_enabled": false,
        "vocabulary": "",
        "monthly_cost_alert_usd": null,
        "wizard_version": 1
    });
    std::fs::write(&path, serde_json::to_string(&legacy).unwrap()).unwrap();

    let loaded = ConfigStore::load_from(&path).unwrap().unwrap();
    assert_eq!(loaded.presets.len(), 4); // hydrated from default
    assert_eq!(loaded.active_preset_id, None);
}
