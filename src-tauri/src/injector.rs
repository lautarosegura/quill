//! TextInjector — inserts text into the currently-focused app via clipboard
//! paste. On Windows/macOS/Linux-X11 we simulate Ctrl+V with `enigo` and
//! restore the user's previous clipboard. On Linux-Wayland the security
//! model forbids programmatic input injection, so we leave the text on the
//! clipboard and let the user press Ctrl+V manually — the orchestrator
//! emits a `ClipboardOnly` state the UI can surface as a toast.

use std::time::Duration;

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

#[allow(unused_imports)]
use crate::display_server::DisplayServer;
use crate::error::QuillError;

const POST_PASTE_SLEEP_MS: u64 = 80;
const MAX_CLIPBOARD_BYTES: usize = 3 * 1024 * 1024;

/// What happened after `inject` returned Ok. Distinguishes the normal
/// "we simulated Ctrl+V" path from the Wayland "we left it on clipboard"
/// fallback so the orchestrator can emit the right UI state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectOutcome {
    Pasted,
    ClipboardOnly { text_len: usize },
}

pub struct TextInjector;

impl TextInjector {
    pub async fn inject(text: &str) -> Result<InjectOutcome, QuillError> {
        if text.is_empty() {
            return Ok(InjectOutcome::Pasted);
        }

        // Wayland path. Two-stage:
        //   1. Put the text on the clipboard via wl-clipboard-rs (talks
        //      directly to the compositor's data_device_manager — arboard
        //      can't be trusted here because on GNOME Wayland `$DISPLAY` is
        //      also set and arboard auto-picks its X11 backend).
        //   2. Synthesize Ctrl+V via the XDG RemoteDesktop portal + libei
        //      so the focused app pastes without the user lifting a finger.
        // If step 2 fails (compositor without portal support, user denies
        // the consent dialog, libei timeout, etc.) we fall back to the
        // clipboard-only UX — the selection is still live so the user can
        // press Ctrl+V manually.
        if DisplayServer::detect().is_wayland() {
            let text_len = text.len();
            wayland_clipboard_hold(text.to_string());
            // Give the compositor a moment to register the clipboard
            // selection before we ask it to emit a paste keystroke.
            tokio::time::sleep(Duration::from_millis(120)).await;

            #[cfg(target_os = "linux")]
            {
                match crate::wayland_paste::paste_ctrl_v().await {
                    Ok(()) => {
                        log::info!("wayland: auto-paste via libei succeeded");
                        return Ok(InjectOutcome::Pasted);
                    }
                    Err(e) => {
                        log::warn!(
                            "wayland auto-paste failed, falling back to clipboard-only: {e}. \
                             User will need to press Ctrl+V manually."
                        );
                        return Ok(InjectOutcome::ClipboardOnly { text_len });
                    }
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                return Ok(InjectOutcome::ClipboardOnly { text_len });
            }
        }

        let mut clipboard =
            Clipboard::new().map_err(|e| QuillError::Injection(format!("clipboard init: {e}")))?;

        // Save existing clipboard text (only preserve text; other types are
        // not restored). Skip if it's absurdly large.
        let original = clipboard.get_text().ok().and_then(|t| {
            if t.len() <= MAX_CLIPBOARD_BYTES {
                Some(t)
            } else {
                None
            }
        });

        clipboard
            .set_text(text.to_string())
            .map_err(|e| QuillError::Injection(format!("clipboard set: {e}")))?;

        let result = simulate_paste();

        tokio::time::sleep(Duration::from_millis(POST_PASTE_SLEEP_MS)).await;

        if let Some(prev) = original {
            let _ = clipboard.set_text(prev);
        } else {
            let _ = clipboard.clear();
        }

        result.map(|_| InjectOutcome::Pasted)
    }
}

#[cfg(target_os = "macos")]
const PASTE_MODIFIER: Key = Key::Meta;

#[cfg(not(target_os = "macos"))]
const PASTE_MODIFIER: Key = Key::Control;

fn simulate_paste() -> Result<(), QuillError> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| QuillError::Injection(format!("enigo init: {e}")))?;

    // Defensive: clear any modifiers the user might still be physically
    // holding before we send our synthetic Ctrl+V. Two separate problems
    // this solves:
    //
    // 1. **Alt+Shift+Z repaste**: the user literally just pressed Z with
    //    Alt+Shift held. If we paste right then, the focused app sees
    //    `Alt+Shift+Ctrl+V` (not `Ctrl+V`) and nothing pastes.
    // 2. **Windows 11 Sound-output picker**: `Win+Ctrl+V` opens the picker.
    //    If our hotkey hook ever leaves Meta "stuck down" in Windows' state,
    //    our paste would trigger the picker instead.
    //
    // Releasing a key that the OS already considers up is a no-op.
    #[cfg(target_os = "windows")]
    {
        let _ = enigo.key(Key::Meta, Direction::Release);
        let _ = enigo.key(Key::Shift, Direction::Release);
        let _ = enigo.key(Key::Alt, Direction::Release);
    }
    #[cfg(target_os = "macos")]
    {
        let _ = enigo.key(Key::Shift, Direction::Release);
        let _ = enigo.key(Key::Option, Direction::Release);
    }

    enigo
        .key(PASTE_MODIFIER, Direction::Press)
        .map_err(|e| QuillError::Injection(format!("press modifier: {e}")))?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| QuillError::Injection(format!("click v: {e}")))?;
    enigo
        .key(PASTE_MODIFIER, Direction::Release)
        .map_err(|e| QuillError::Injection(format!("release modifier: {e}")))?;
    Ok(())
}

/// Wayland helper — talks directly to the wl_data_device_manager protocol
/// via `wl-clipboard-rs`. We can't use `arboard` here because on GNOME
/// Wayland (the most common Wayland setup) both `$DISPLAY` and
/// `$WAYLAND_DISPLAY` are set — arboard picks its X11 backend on sight of
/// `$DISPLAY` and writes to the XWayland pseudo-selection, which is
/// invisible to native Wayland apps when they Ctrl+V.
///
/// The thread blocks in `wl-clipboard-rs::copy` with `foreground=true` +
/// `ServeRequests::Unlimited`. The function returns when another client
/// takes selection ownership (the user pastes into the target, or copies
/// something else from any app), which tears the thread down cleanly.
#[cfg(target_os = "linux")]
fn wayland_clipboard_hold(text: String) {
    use wl_clipboard_rs::copy::{MimeType, Options, ServeRequests, Source};

    std::thread::spawn(move || {
        let mut opts = Options::new();
        // foreground(true) keeps the wayland connection alive in THIS
        // thread — no fork(), which would interact badly with Tauri's
        // multi-threaded process.
        opts.foreground(true);
        opts.serve_requests(ServeRequests::Unlimited);
        let source = Source::Bytes(text.into_bytes().into_boxed_slice());
        match opts.copy(source, MimeType::Text) {
            Ok(()) => log::info!("wayland clipboard: selection served until replaced"),
            Err(e) => log::warn!("wayland clipboard copy failed: {e}"),
        }
    });
}

#[cfg(not(target_os = "linux"))]
fn wayland_clipboard_hold(_text: String) {
    // Unreachable — `DisplayServer::detect().is_wayland()` is hard-coded
    // false on non-Linux platforms. This stub exists so the call site in
    // `inject` doesn't need a `cfg` guard.
}
