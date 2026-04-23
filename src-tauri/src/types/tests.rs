use super::*;

#[test]
fn language_serializes_as_lowercase_string() {
    assert_eq!(serde_json::to_string(&Language::Es).unwrap(), r#""es""#);
    assert_eq!(serde_json::to_string(&Language::En).unwrap(), r#""en""#);
}

#[test]
fn language_round_trips() {
    let original = Language::Es;
    let j = serde_json::to_string(&original).unwrap();
    let restored: Language = serde_json::from_str(&j).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn language_code_and_label() {
    assert_eq!(Language::Es.code(), "es");
    assert_eq!(Language::En.code(), "en");
    assert_eq!(Language::Es.label(), "Español");
    assert_eq!(Language::En.label(), "English");
}

#[test]
fn engine_serializes_as_lowercase() {
    assert_eq!(serde_json::to_string(&Engine::Local).unwrap(), r#""local""#);
    assert_eq!(serde_json::to_string(&Engine::Groq).unwrap(), r#""groq""#);
}

#[test]
fn overlay_position_serializes_kebab_case() {
    let j = serde_json::to_string(&OverlayPosition::BottomCenter).unwrap();
    assert_eq!(j, r#""bottom-center""#);
}

#[test]
#[cfg(not(target_os = "windows"))]
fn default_push_to_talk_unix_is_ctrl_shift_space() {
    let k = Keybind::default_push_to_talk();
    assert_eq!(k.modifiers, vec![Modifier::Ctrl, Modifier::Shift]);
    assert_eq!(k.key.as_deref(), Some("Space"));
}

#[test]
#[cfg(target_os = "windows")]
fn default_push_to_talk_windows_is_modifier_only_ctrl_meta() {
    let k = Keybind::default_push_to_talk();
    assert_eq!(k.modifiers, vec![Modifier::Ctrl, Modifier::Meta]);
    assert_eq!(k.key, None);
}

#[test]
fn keybind_round_trips() {
    let original = Keybind::default_push_to_talk();
    let j = serde_json::to_string(&original).unwrap();
    let restored: Keybind = serde_json::from_str(&j).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn keybind_with_trigger_round_trips() {
    let original = Keybind {
        modifiers: vec![Modifier::Ctrl, Modifier::Shift],
        key: Some("Space".into()),
    };
    let j = serde_json::to_string(&original).unwrap();
    let restored: Keybind = serde_json::from_str(&j).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn keybind_modifier_only_round_trips() {
    let original = Keybind {
        modifiers: vec![Modifier::Ctrl, Modifier::Meta],
        key: None,
    };
    let j = serde_json::to_string(&original).unwrap();
    let restored: Keybind = serde_json::from_str(&j).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn legacy_config_with_plain_string_key_still_parses() {
    // Users upgrading from pre-Phase-7 have a config where "key" is a plain
    // string ("Space"). The Option<String> field must still accept that.
    let j = r#"{"modifiers":["ctrl","shift"],"key":"Space"}"#;
    let k: Keybind = serde_json::from_str(j).expect("legacy shape should parse");
    assert_eq!(k.modifiers, vec![Modifier::Ctrl, Modifier::Shift]);
    assert_eq!(k.key.as_deref(), Some("Space"));
}

#[test]
fn modifier_only_config_without_key_field_parses_as_none() {
    // Serde treats a missing Option field as None by default.
    let j = r#"{"modifiers":["ctrl","meta"]}"#;
    let k: Keybind = serde_json::from_str(j).expect("omitted key should be None");
    assert_eq!(k.key, None);
}
