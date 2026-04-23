//! HotkeyManager — owns the backend that listens for the global push-to-talk
//! chord. The backend choice is runtime-dynamic because Wayland is not a
//! source-code-level platform (it shares `target_os = "linux"` with X11) —
//! the same binary has to do the right thing on both session types.
//!
//! - **Windows / macOS / Linux-X11** → [`rdev_backend`] uses `rdev::grab`
//!   so we can fully suppress modifiers (needed to prevent Start-menu
//!   popping on Windows) and see precise keydown / keyup timing.
//! - **Linux-Wayland** → [`wayland_backend`] talks to the XDG Desktop
//!   Portal `GlobalShortcuts` service via `ashpd`. The compositor owns
//!   the key-binding UX; we just subscribe to Activated / Deactivated
//!   events and translate them into the same [`HotkeyEvent`] stream.
//!
//! Regardless of backend, consumers read a single
//! `mpsc::UnboundedReceiver<HotkeyEvent>` — the orchestrator is
//! platform-agnostic.

use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::config::Config;
#[cfg(target_os = "linux")]
use crate::display_server::DisplayServer;

mod rdev_backend;
#[cfg(target_os = "linux")]
mod wayland_backend;

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    /// User started the chord. `source_app` is the title of the foreground
    /// window at that moment (captured on the hook thread, cheap). Always
    /// `None` on Wayland — the compositor intentionally does not expose it.
    Pressed {
        source_app: Option<String>,
    },
    Released {
        held_ms: u64,
    },
    /// User pressed Escape — the orchestrator cancels an active (locked)
    /// session if there is one, otherwise ignores. Only fires on X11/Win/mac
    /// (the rdev backend sees raw Escape). On Wayland Escape reaches the
    /// focused app normally, as it should.
    CancelRequested,
    /// User pressed Alt+Shift+Z (rdev) or invoked the repaste portal
    /// shortcut (Wayland) — re-paste the last successful transcription.
    ReinjectLast,
}

/// Keeps the listener alive for the app lifetime. rdev's native thread has
/// no clean stop; the Wayland task stops when the D-Bus proxy is dropped.
/// Holding the handle prevents premature cleanup.
pub struct HotkeyManager {
    _inner: Inner,
}

#[allow(dead_code)] // variants are cfg-dependent; the unused one drops out per target
enum Inner {
    Rdev(std::thread::JoinHandle<()>),
    #[cfg(target_os = "linux")]
    WaylandPortal(tokio::task::JoinHandle<()>),
}

impl HotkeyManager {
    pub fn start(config: Arc<RwLock<Config>>) -> (Self, mpsc::UnboundedReceiver<HotkeyEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();

        #[cfg(target_os = "linux")]
        {
            if DisplayServer::detect().is_wayland() {
                log::info!("hotkey: using Wayland XDG GlobalShortcuts backend");
                let task = wayland_backend::start(tx);
                return (
                    Self {
                        _inner: Inner::WaylandPortal(task),
                    },
                    rx,
                );
            }
        }

        log::info!("hotkey: using rdev grab backend");
        let thread = rdev_backend::start(config, tx);
        (
            Self {
                _inner: Inner::Rdev(thread),
            },
            rx,
        )
    }
}
