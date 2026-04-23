//! Detects which display server / platform backend Quill is running under.
//!
//! On Linux the app has to choose between two completely different stacks:
//! - **X11** — use `rdev::grab` for the hotkey and `enigo` Ctrl+V for paste.
//! - **Wayland** — use the XDG Desktop Portal (`ashpd::GlobalShortcuts`) for
//!   the hotkey and `ashpd::RemoteDesktop` + libei for paste, because
//!   Wayland's security model forbids regular apps from capturing global
//!   input or synthesizing keys into other windows.
//!
//! Detection is a cheap env-var read; safe to call repeatedly.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayServer {
    Windows,
    #[serde(rename = "macos")]
    MacOS,
    X11,
    Wayland,
}

impl DisplayServer {
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            DisplayServer::Windows
        }
        #[cfg(target_os = "macos")]
        {
            DisplayServer::MacOS
        }
        #[cfg(target_os = "linux")]
        {
            // XDG_SESSION_TYPE is the canonical value set by systemd-logind /
            // the display manager. WAYLAND_DISPLAY is a secondary signal used
            // when the env is misconfigured (e.g. a tmux shell that didn't
            // inherit session vars).
            match std::env::var("XDG_SESSION_TYPE").ok().as_deref() {
                Some("wayland") => DisplayServer::Wayland,
                Some("x11") => DisplayServer::X11,
                _ => {
                    if std::env::var("WAYLAND_DISPLAY").is_ok() {
                        DisplayServer::Wayland
                    } else {
                        DisplayServer::X11
                    }
                }
            }
        }
    }

    pub fn is_wayland(self) -> bool {
        matches!(self, DisplayServer::Wayland)
    }
}
