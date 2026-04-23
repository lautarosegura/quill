//! TextInjector — inserts text into the currently-focused app via clipboard
//! paste. Saves and restores the user's previous clipboard.

use std::time::Duration;

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

use crate::error::QuillError;

const POST_PASTE_SLEEP_MS: u64 = 80;
const MAX_CLIPBOARD_BYTES: usize = 3 * 1024 * 1024;

pub struct TextInjector;

impl TextInjector {
    pub async fn inject(text: &str) -> Result<(), QuillError> {
        if text.is_empty() {
            return Ok(());
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

        result
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
