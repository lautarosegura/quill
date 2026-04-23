//! Wayland hotkey backend — uses the XDG Desktop Portal
//! `org.freedesktop.portal.GlobalShortcuts` via `ashpd`. The compositor
//! owns the shortcut-binding UX: on first bind GNOME/KDE show a system
//! dialog letting the user approve (or edit) the shortcut; Hyprland
//! requires the user to set the key in `hyprland.conf`; Sway/wlroots
//! don't implement the portal at all — the backend just logs and exits
//! cleanly there, and the user falls back to invoking Quill via tray.
//!
//! Push-to-talk is preserved because the portal emits `Activated` on
//! keydown and `Deactivated` on keyup — we translate those directly to
//! `HotkeyEvent::Pressed` / `Released`.

use std::time::Instant;

use futures_util::StreamExt;
use tokio::sync::mpsc;

use super::HotkeyEvent;

/// Stable IDs for the three shortcuts Quill registers. The portal lets the
/// user reassign the underlying keys via system settings; our IDs stay
/// constant so we can match on them in the Activated/Deactivated streams.
const PTT_ID: &str = "quill-ptt";
const REPASTE_ID: &str = "quill-repaste";
const CANCEL_ID: &str = "quill-cancel";

pub fn start(tx: mpsc::UnboundedSender<HotkeyEvent>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = run(tx).await {
            log::error!(
                "wayland portal hotkey backend exited with error: {e}. \
                 The app will still run but the global hotkey is inactive \
                 on this session. Compositors without GlobalShortcuts \
                 portal support (Sway/wlroots, older GNOME) land here."
            );
        }
    })
}

async fn run(tx: mpsc::UnboundedSender<HotkeyEvent>) -> ashpd::Result<()> {
    use ashpd::desktop::global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut};
    use ashpd::desktop::CreateSessionOptions;

    let proxy = GlobalShortcuts::new().await?;
    let session = proxy
        .create_session(CreateSessionOptions::default())
        .await?;

    let shortcuts = [
        NewShortcut::new(PTT_ID, "Dictate with Quill (hold to record)"),
        NewShortcut::new(REPASTE_ID, "Re-paste last transcription"),
        NewShortcut::new(CANCEL_ID, "Cancel active dictation"),
    ];
    // `identifier: None` means no parent window — we register the shortcuts
    // at app scope rather than tying them to a specific toplevel. The
    // compositor will show its own system dialog for approval.
    proxy
        .bind_shortcuts(&session, &shortcuts, None, BindShortcutsOptions::default())
        .await?;

    let mut activated = proxy.receive_activated().await?;
    let mut deactivated = proxy.receive_deactivated().await?;

    let mut pressed_at: Option<Instant> = None;

    loop {
        tokio::select! {
            Some(activation) = activated.next() => {
                let id = activation.shortcut_id().to_string();
                match id.as_str() {
                    PTT_ID => {
                        pressed_at = Some(Instant::now());
                        // source_app is always None on Wayland — the
                        // compositor intentionally doesn't expose which app
                        // is focused to third-party clients.
                        let _ = tx.send(HotkeyEvent::Pressed { source_app: None });
                    }
                    REPASTE_ID => {
                        let _ = tx.send(HotkeyEvent::ReinjectLast);
                    }
                    CANCEL_ID => {
                        let _ = tx.send(HotkeyEvent::CancelRequested);
                    }
                    other => {
                        log::debug!("ignoring unknown activated shortcut id: {other}");
                    }
                }
            }
            Some(deactivation) = deactivated.next() => {
                let id = deactivation.shortcut_id().to_string();
                if id == PTT_ID {
                    let held_ms = pressed_at
                        .take()
                        .map(|t| t.elapsed().as_millis() as u64)
                        .unwrap_or(0);
                    let _ = tx.send(HotkeyEvent::Released { held_ms });
                }
            }
            else => break,
        }
    }

    Ok(())
}
