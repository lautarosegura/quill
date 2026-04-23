//! Short audio cues played when dictation starts and stops.
//!
//! Sine waves are generated programmatically — no asset files. Everything is
//! gated on `Config::sounds_enabled`, which the user toggles in Settings.
//!
//! ## Why a dedicated std thread?
//! `rodio::OutputStream` is `!Send` + `!Sync`, so it can't live in a tokio
//! task or be captured by the `Send`-bound Tauri listener closure. The thread
//! below owns the stream for the process lifetime and plays tones on demand
//! via an `mpsc` channel.
//!
//! ## Why `try_read` on the tokio RwLock?
//! Turns out `app.listen` callbacks in Tauri 2 run on a *tokio runtime worker*
//! (not a plain OS thread), so `blocking_read` panics: you can't block a
//! runtime thread. `try_read` is non-blocking and safe from any context; if
//! it fails due to a concurrent write (rare — only happens during a settings
//! save), we fall back to "sounds off" and skip the beep.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::Source;
use tauri::{AppHandle, Listener};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::events::{self, TranscriptionState};

#[derive(Debug, Clone, Copy)]
enum BeepCmd {
    Start,
    Stop,
}

/// Listen for transcription-state events and play a short tone on the
/// transitions `* -> Recording` (start) and `Recording -> *` (stop), provided
/// the user has opted in via `Config::sounds_enabled`.
pub fn spawn_beep_listener(app: AppHandle, config: Arc<RwLock<Config>>) {
    let (tx, rx) = std::sync::mpsc::channel::<BeepCmd>();

    // Audio playback thread: owns the !Send OutputStream for the app lifetime.
    std::thread::spawn(move || {
        let (_stream, handle) = match rodio::OutputStream::try_default() {
            Ok(pair) => pair,
            Err(e) => {
                log::warn!("audio output unavailable; beeps disabled ({e})");
                return;
            }
        };
        while let Ok(cmd) = rx.recv() {
            let hz: f32 = match cmd {
                BeepCmd::Start => 880.0, // A5
                BeepCmd::Stop => 660.0,  // E5
            };
            let sink = match rodio::Sink::try_new(&handle) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("sink creation failed: {e}");
                    continue;
                }
            };
            sink.append(
                rodio::source::SineWave::new(hz)
                    .take_duration(Duration::from_millis(80))
                    .amplify(0.1),
            );
            sink.sleep_until_end();
        }
    });

    // Previous observed state — used to detect *transitions*, not just arrivals.
    let prev: Arc<Mutex<TranscriptionState>> = Arc::new(Mutex::new(TranscriptionState::Idle));

    app.listen(events::TRANSCRIPTION_STATE_CHANGED, move |event| {
        let Ok(next) = serde_json::from_str::<TranscriptionState>(event.payload()) else {
            return;
        };

        // Swap in the new state and grab the old one in a single critical section.
        let previous = {
            let mut guard = match prev.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            std::mem::replace(&mut *guard, next.clone())
        };

        // No-op if state didn't actually change.
        if previous == next {
            return;
        }

        let was_recording = matches!(previous, TranscriptionState::Recording);
        let is_recording = matches!(next, TranscriptionState::Recording);

        let cmd = if !was_recording && is_recording {
            BeepCmd::Start
        } else if was_recording && !is_recording {
            BeepCmd::Stop
        } else {
            return;
        };

        // Gate on the user's opt-in. `try_read` is non-blocking and safe from
        // the tokio runtime worker this closure runs on — if a write lock is
        // held right now, we just skip this beep rather than panic.
        let sounds_on = config.try_read().map(|c| c.sounds_enabled).unwrap_or(false);
        if !sounds_on {
            return;
        }

        if let Err(e) = tx.send(cmd) {
            log::warn!("beep channel closed: {e}");
        }
    });
}
