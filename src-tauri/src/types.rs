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

#[cfg(test)]
mod tests;
