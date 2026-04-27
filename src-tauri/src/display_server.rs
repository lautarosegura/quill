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

/// Linux-only environment fingerprint. Tells the wizard whether the user
/// is on a "zero-config" combo (GNOME 48+ Wayland, KDE Plasma 6 Wayland,
/// or any X11 session) or whether they need to opt into the `input` group
/// for the rdev evdev-listen fallback to work.
///
/// Detection sources:
/// - `display_server`: reuses [`DisplayServer::detect`]
/// - `desktop`: read from `XDG_CURRENT_DESKTOP` env var (colon-separated,
///   we take the first segment)
/// - `gnome_version`: parsed from `gnome-shell --version` stdout when
///   the desktop is GNOME. Only the major version (`48` from `"48.0"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxEnvironment {
    pub display_server: DisplayServer,
    pub desktop: String,
    pub gnome_version: Option<u32>,
}

#[cfg(target_os = "linux")]
pub fn detect_linux_environment() -> LinuxEnvironment {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .and_then(|s| s.split(':').next().map(|seg| seg.to_string()))
        .unwrap_or_else(|| "Unknown".to_string());
    let gnome_version = if desktop.eq_ignore_ascii_case("GNOME") {
        gnome_shell_major_version()
    } else {
        None
    };
    LinuxEnvironment {
        display_server: DisplayServer::detect(),
        desktop,
        gnome_version,
    }
}

#[cfg(target_os = "linux")]
fn gnome_shell_major_version() -> Option<u32> {
    let output = std::process::Command::new("gnome-shell")
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    // stdout format: "GNOME Shell 48.0" / "GNOME Shell 46.5"
    let s = String::from_utf8_lossy(&output.stdout);
    let last_token = s.split_whitespace().last()?;
    last_token.split('.').next()?.parse::<u32>().ok()
}
