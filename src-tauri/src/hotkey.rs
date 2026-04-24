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

/// Keeps all listener handles alive for the app lifetime. rdev's native
/// thread has no clean stop; the Wayland task stops when the D-Bus proxy
/// is dropped. Holding the handles prevents premature cleanup.
///
/// On Wayland we intentionally hold BOTH the portal task and an rdev
/// evdev-listen thread: whichever of them actually picks up events feeds
/// the shared channel. The orchestrator's own "if s.recording return"
/// guard deduplicates the rare case where both sources fire for the
/// same key sequence.
pub struct HotkeyManager {
    _handles: Vec<Handle>,
}

#[allow(dead_code)] // variants are cfg-dependent
enum Handle {
    Thread(std::thread::JoinHandle<()>),
    #[cfg(target_os = "linux")]
    Task(tokio::task::JoinHandle<()>),
}

impl HotkeyManager {
    pub fn start(config: Arc<RwLock<Config>>) -> (Self, mpsc::UnboundedReceiver<HotkeyEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();

        #[cfg(target_os = "linux")]
        {
            if DisplayServer::detect().is_wayland() {
                log::info!(
                    "hotkey: Wayland session — starting dual backend (XDG \
                     GlobalShortcuts portal + rdev evdev listen fallback)"
                );
                let portal = wayland_backend::start(tx.clone());
                let evdev = rdev_backend::start_listen(Arc::clone(&config), tx);
                return (
                    Self {
                        _handles: vec![Handle::Task(portal), Handle::Thread(evdev)],
                    },
                    rx,
                );
            }
        }

        log::info!("hotkey: using rdev grab backend");
        let thread = rdev_backend::start(config, tx);
        (
            Self {
                _handles: vec![Handle::Thread(thread)],
            },
            rx,
        )
    }
}
