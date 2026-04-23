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

        // Wayland path: `arboard` only holds the selection while its handle is
        // alive — as soon as our function returns, another Wayland client (or
        // nothing) takes ownership and the clipboard is empty. That would defeat
        // the whole "press Ctrl+V to paste" affordance. Instead we detach a
        // dedicated thread that blocks on `SetExtLinux::wait_until` for up to a
        // minute, which keeps the selection valid until either the user pastes
        // (another client claims ownership, wait returns) or the deadline hits.
        //
        // Check Wayland BEFORE constructing the sync Clipboard so we don't
        // create and immediately drop one — which on some compositors still
        // ships a transient selection we'd then immediately lose.
        if DisplayServer::detect().is_wayland() {
            let text_len = text.len();
            wayland_clipboard_hold(text.to_string());
            return Ok(InjectOutcome::ClipboardOnly { text_len });
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
