//! AppOrchestrator — the central state machine that drives every dictation.
//!
//! Supports two modes:
//! - **Push-to-talk**: hold hotkey → record, release → finalize.
//! - **Double-tap lock**: quick tap-tap of the hotkey starts a locked session
//!   that records hands-free until the user taps the hotkey once more.
//!
//! The two modes share the same `HotkeyEvent` stream; the choice is inferred
//! from `held_ms` + the gap between releases (see [`DOUBLE_TAP_GAP`]).

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, Mutex, RwLock};

/// Max time between the first release and the second press for a pair of taps
/// to count as a double-tap. 300 ms is tight enough to not collide with
/// "oops, tapped twice" but loose enough to be reachable for most users.
const DOUBLE_TAP_GAP: Duration = Duration::from_millis(300);

use crate::audio::AudioRecorder;
use crate::config::Config;
use crate::cost_alert;
use crate::engines::{TranscriptionEngine, TranscriptionRequest};
use crate::events::{TranscriptionState, HISTORY_CHANGED, TRANSCRIPTION_STATE_CHANGED};
use crate::failed_wav;
use crate::history::{HistoryStore, NewHistoryEntry};
use crate::hotkey::HotkeyEvent;
use crate::injector::{InjectOutcome, TextInjector};
use crate::post_process;
use crate::types::Engine;

/// Decouples the orchestrator from tauri::AppHandle so the flow can be unit-tested
/// with a mock emitter.
pub trait EventEmitter: Send + Sync {
    fn emit_state(&self, state: &TranscriptionState);
    /// Signal to the frontend that a new history row was just persisted so
    /// the Historial view can refresh live instead of polling.
    fn emit_history_changed(&self);
}

impl EventEmitter for tauri::AppHandle {
    fn emit_state(&self, state: &TranscriptionState) {
        use tauri::Emitter;
        let _ = self.emit(TRANSCRIPTION_STATE_CHANGED, state);
    }
    fn emit_history_changed(&self) {
        use tauri::Emitter;
        let _ = self.emit(HISTORY_CHANGED, ());
    }
}

/// Fires desktop notifications for cost alerts and similar user-facing events.
/// Kept separate from [`EventEmitter`] because tests want to exercise the state
/// machine without touching the notification plugin.
pub trait Notifier: Send + Sync {
    fn notify(&self, title: &str, body: &str);
}

impl Notifier for tauri::AppHandle {
    fn notify(&self, title: &str, body: &str) {
        use tauri_plugin_notification::NotificationExt;
        if let Err(e) = self.notification().builder().title(title).body(body).show() {
            log::warn!("notification show failed: {e}");
        }
    }
}

/// Runtime state of the dictation session. One field for each discrete phase
/// plus timestamps for double-tap detection.
#[derive(Default)]
struct SessionState {
    /// The mic is actively capturing audio.
    recording: bool,
    /// We're in a locked hands-free session — recording persists after release.
    locked: bool,
    /// User tapped the hotkey to exit a locked session; finalize on the release.
    exiting_lock: bool,
    /// The current press is the second tap of a potential double-tap. Turns
    /// into `locked = true` on the matching short release.
    awaiting_lock_arm: bool,
    /// When the most recent short-tap release happened. Used to detect
    /// double-taps within [`DOUBLE_TAP_GAP`].
    last_short_tap_at: Option<Instant>,
    /// Foreground-window title captured at Press time. Propagated to the
    /// history row on finalize so Historial can show "dictated into: …".
    source_app: Option<String>,
}

impl SessionState {
    /// Clears everything except the double-tap memory. Called after a
    /// dictation finalizes (success or error) so the next press starts fresh.
    fn reset(&mut self) {
        self.recording = false;
        self.locked = false;
        self.exiting_lock = false;
        self.awaiting_lock_arm = false;
        self.source_app = None;
        // Keep `last_short_tap_at` — a short tap that just happened should
        // still be able to arm a double-tap even if handle_release ran in
        // between (rare race, but harmless to preserve).
    }
}

pub struct AppOrchestrator {
    pub emitter: Arc<dyn EventEmitter>,
    pub engine: Arc<dyn TranscriptionEngine>,
    pub recorder: Arc<AudioRecorder>,
    /// Shared state — language, vocabulary, min_duration_ms all read from here
    /// at event time so runtime changes apply without restart.
    pub config: Arc<RwLock<Config>>,
    /// Optional to keep tests that run without disk happy.
    pub history: Option<Arc<HistoryStore>>,
    /// Optional to keep tests decoupled from the Tauri notification plugin.
    pub notifier: Option<Arc<dyn Notifier>>,
    session: Arc<Mutex<SessionState>>,
}

impl AppOrchestrator {
    pub fn new(
        emitter: Arc<dyn EventEmitter>,
        engine: Arc<dyn TranscriptionEngine>,
        recorder: Arc<AudioRecorder>,
        config: Arc<RwLock<Config>>,
    ) -> Self {
        Self {
            emitter,
            engine,
            recorder,
            config,
            history: None,
            notifier: None,
            session: Arc::new(Mutex::new(SessionState::default())),
        }
    }

    pub fn with_history(mut self, history: Arc<HistoryStore>) -> Self {
        self.history = Some(history);
        self
    }

    pub fn with_notifier(mut self, notifier: Arc<dyn Notifier>) -> Self {
        self.notifier = Some(notifier);
        self
    }

    fn emit(&self, state: TranscriptionState) {
        self.emitter.emit_state(&state);
    }

    pub async fn run(self: Arc<Self>, mut hotkey_rx: mpsc::UnboundedReceiver<HotkeyEvent>) {
        while let Some(event) = hotkey_rx.recv().await {
            match event {
                HotkeyEvent::Pressed { source_app } => self.on_pressed(source_app).await,
                HotkeyEvent::Released { held_ms } => {
                    let min_dur = self.config.read().await.min_duration_ms as u64;
                    let orch = Arc::clone(&self);
                    orch.on_released(held_ms, min_dur).await;
                }
                HotkeyEvent::CancelRequested => {
                    let orch = Arc::clone(&self);
                    orch.on_cancel().await;
                }
                HotkeyEvent::ReinjectLast => {
                    let orch = Arc::clone(&self);
                    tokio::spawn(async move {
                        orch.on_reinject_last().await;
                    });
                }
            }
        }
    }

    /// Cancel an in-flight session (typically a locked hands-free recording).
    /// Silent no-op if nothing is active — Escape is pressed frequently for
    /// other reasons and must not produce spurious state transitions.
    async fn on_cancel(self: Arc<Self>) {
        let mut s = self.session.lock().await;
        if !s.recording && !s.locked {
            return;
        }
        s.reset();
        drop(s);

        // Discard whatever the mic has buffered.
        let _ = self.recorder.stop_recording();

        self.emit(TranscriptionState::Cancelled);

        // Auto-transition to Idle after a short feedback window so the
        // overlay disappears. Check session state before emitting Idle so
        // a new dictation started during the delay isn't clobbered.
        let orch = Arc::clone(&self);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(1200)).await;
            let s = orch.session.lock().await;
            if !s.recording && !s.locked {
                drop(s);
                orch.emit(TranscriptionState::Idle);
            }
        });
    }

    /// Pastes the most recent successful transcription into the focused app.
    /// No-op if there's nothing paste-worthy in history.
    async fn on_reinject_last(self: Arc<Self>) {
        log::info!("on_reinject_last fired");
        let Some(history) = &self.history else {
            log::warn!("on_reinject_last: no history store");
            return;
        };
        let rows = match history.recent(10, 0) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("on_reinject_last: history.recent failed: {e}");
                return;
            }
        };
        log::info!("on_reinject_last: {} recent rows", rows.len());
        let Some(entry) = rows
            .into_iter()
            .find(|e| e.status == "success" && !e.text.is_empty())
        else {
            log::warn!("on_reinject_last: no success entry with non-empty text found");
            return;
        };
        log::info!(
            "on_reinject_last: injecting {} chars from entry {}",
            entry.text.len(),
            entry.id
        );
        match TextInjector::inject(&entry.text).await {
            Ok(InjectOutcome::Pasted) => {
                log::info!("on_reinject_last: inject succeeded");
            }
            Ok(InjectOutcome::ClipboardOnly { text_len }) => {
                log::info!("on_reinject_last: clipboard-only ({text_len} chars)");
                self.emit(TranscriptionState::ClipboardOnly { text_len });
                let orch = Arc::clone(&self);
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(2500)).await;
                    let s = orch.session.lock().await;
                    if !s.recording && !s.locked {
                        drop(s);
                        orch.emit(TranscriptionState::Idle);
                    }
                });
            }
            Err(e) => {
                log::warn!("reinject_last inject failed: {e}");
            }
        }
    }

    async fn on_pressed(self: &Arc<Self>, source_app: Option<String>) {
        let mut s = self.session.lock().await;

        // Locked session: this press signals "stop". Mic keeps running until
        // the matching Release, to catch any final syllable.
        if s.locked {
            s.exiting_lock = true;
            return;
        }

        // Safety: ignore stray presses if we're already recording.
        if s.recording {
            return;
        }

        // If the previous short tap was recent enough, arm this press to
        // transition into locked mode on its matching short release.
        let is_second_tap = s
            .last_short_tap_at
            .map(|t| t.elapsed() < DOUBLE_TAP_GAP)
            .unwrap_or(false);
        s.awaiting_lock_arm = is_second_tap;
        s.last_short_tap_at = None;
        s.recording = true;
        s.source_app = source_app;
        drop(s);

        self.recorder.start_recording();
        self.emit(TranscriptionState::Recording);
    }

    async fn on_released(self: Arc<Self>, held_ms: u64, min_duration_ms: u64) {
        let mut s = self.session.lock().await;

        // Exiting a locked session — finalize normally. We clear only the
        // lock-related flags; `recording` stays true until handle_release
        // finishes + resets, so a stray press during the transcription
        // window (which runs async) does NOT start a concurrent session.
        if s.exiting_lock {
            s.locked = false;
            s.exiting_lock = false;
            s.awaiting_lock_arm = false;
            drop(s);
            let orch = Arc::clone(&self);
            tokio::spawn(async move {
                orch.handle_release().await;
            });
            return;
        }

        let was_arm = s.awaiting_lock_arm;
        s.awaiting_lock_arm = false;

        // Second tap of a double-tap completed — enter locked mode. The mic
        // keeps running; the next Press will signal stop.
        if was_arm && held_ms < min_duration_ms {
            s.locked = true;
            drop(s);
            // Reuse the Recording state so the overlay stays visible. Users
            // know they're locked because they did the tap-tap gesture.
            return;
        }

        // Short tap that's not a lock-arm — discard audio, remember the
        // timestamp so a follow-up press can form a double-tap.
        if held_ms < min_duration_ms {
            s.recording = false;
            s.last_short_tap_at = Some(Instant::now());
            drop(s);
            let _ = self.recorder.stop_recording();
            self.emit(TranscriptionState::Idle);
            return;
        }

        // Normal push-to-talk release — finalize. Keep `recording = true`
        // for the same concurrency reason as the exiting_lock branch above;
        // handle_release's reset_session() clears it at the end.
        drop(s);
        let orch = Arc::clone(&self);
        tokio::spawn(async move {
            orch.handle_release().await;
        });
    }

    async fn handle_release(self: Arc<Self>) {
        self.emit(TranscriptionState::Transcribing);

        // Snapshot the captured source app once; it's stable for this session
        // and will be re-read at insert time via `self.session`.
        let source_app = self.session.lock().await.source_app.clone();

        let wav = match self.recorder.stop_recording() {
            Ok(bytes) => bytes,
            Err(e) => {
                self.record_failure("", &format!("audio error: {e}")).await;
                self.emit_error(format!("audio error: {e}")).await;
                return;
            }
        };

        let duration_ms = wav_duration_ms(&wav);

        // Snapshot the bits of config we need — keeps the RwLock guard short.
        let (language, vocabulary, engine_choice) = {
            let cfg = self.config.read().await;
            (
                cfg.language,
                if cfg.vocabulary.is_empty() {
                    None
                } else {
                    Some(cfg.vocabulary.clone())
                },
                cfg.engine,
            )
        };

        let req = TranscriptionRequest {
            audio_wav: &wav,
            language,
            prompt: vocabulary.as_deref(),
        };

        let result = match self.engine.transcribe(req).await {
            Ok(r) => r,
            Err(e) => {
                let msg = format!("transcription failed: {e}");
                // Preserve the captured audio so the user can retry later
                // without re-recording. Swept from disk after 24h on startup.
                let wav_path = match failed_wav::save(&wav) {
                    Ok(p) => Some(p.to_string_lossy().into_owned()),
                    Err(err) => {
                        log::warn!("could not preserve failed WAV: {err}");
                        None
                    }
                };
                self.record_failure_full(
                    engine_choice,
                    language,
                    duration_ms,
                    None,
                    None,
                    wav_path,
                    source_app.clone(),
                    &msg,
                )
                .await;
                self.emit_error(msg).await;
                return;
            }
        };

        // Snapshot substitutions once per call so a Settings change
        // mid-transcribe doesn't half-apply.
        let substitutions = self.config.read().await.substitutions.clone();
        let text = post_process::substitute(&post_process::clean(&result.text), &substitutions);

        if text.is_empty() {
            self.emit(TranscriptionState::Idle);
            self.reset_session().await;
            return;
        }

        self.emit(TranscriptionState::Injecting);
        match TextInjector::inject(&text).await {
            Ok(outcome) => {
                // Persist success regardless of whether we pasted directly
                // or left the text on the clipboard — the transcription
                // itself succeeded in both cases.
                self.persist(
                    engine_choice,
                    language,
                    duration_ms,
                    Some(result.latency_ms as i64),
                    Some(result.model),
                    &text,
                    "success",
                    None,
                    None,
                    source_app,
                )
                .await;

                // If the user just crossed their monthly Groq budget for the
                // first time this month, fire a system notification. Best-
                // effort; any error here must not affect the happy path.
                if matches!(engine_choice, Engine::Groq) {
                    self.maybe_fire_cost_alert().await;
                }

                match outcome {
                    InjectOutcome::Pasted => {
                        self.emit(TranscriptionState::Idle);
                    }
                    InjectOutcome::ClipboardOnly { text_len } => {
                        // Wayland fallback: text is on the clipboard. Surface a
                        // toast and schedule auto-transition to Idle so the
                        // overlay goes away after the user has had a chance to
                        // paste.
                        self.emit(TranscriptionState::ClipboardOnly { text_len });
                        let orch = Arc::clone(&self);
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_millis(2500)).await;
                            let s = orch.session.lock().await;
                            if !s.recording && !s.locked {
                                drop(s);
                                orch.emit(TranscriptionState::Idle);
                            }
                        });
                    }
                }
                self.reset_session().await;
            }
            Err(e) => {
                // Transcription succeeded but paste failed. The text is
                // already captured, so "Reintentar" here just means
                // re-inject via the existing reinject command — no need to
                // preserve the WAV.
                let msg = format!("inject failed: {e}");
                self.persist(
                    engine_choice,
                    language,
                    duration_ms,
                    Some(result.latency_ms as i64),
                    Some(result.model.clone()),
                    &text,
                    "failed",
                    Some(&msg),
                    None,
                    source_app.clone(),
                )
                .await;
                self.emit_error(msg).await;
            }
        }
    }

    async fn maybe_fire_cost_alert(&self) {
        let (Some(history), Some(notifier)) = (&self.history, &self.notifier) else {
            return;
        };
        let threshold = self.config.read().await.monthly_cost_alert_usd;
        let Some(threshold) = threshold else { return };
        match cost_alert::check(history, threshold) {
            Ok(Some(alert)) => {
                notifier.notify(
                    "Alerta de gasto — Quill",
                    &format!(
                        "Ya gastaste ${:.2} USD en Groq este mes (límite ${:.2}).",
                        alert.current_cost_usd, alert.threshold_usd
                    ),
                );
            }
            Ok(None) => {}
            Err(e) => log::warn!("cost alert check failed: {e}"),
        }
    }

    async fn record_failure(&self, text: &str, reason: &str) {
        let (language, engine_choice) = {
            let cfg = self.config.read().await;
            (cfg.language, cfg.engine)
        };
        let source_app = self.session.lock().await.source_app.clone();
        self.record_failure_full(
            engine_choice,
            language,
            None,
            None,
            None,
            None,
            source_app,
            reason,
        )
        .await;
        let _ = text;
    }

    #[allow(clippy::too_many_arguments)]
    async fn record_failure_full(
        &self,
        engine: Engine,
        language: crate::types::Language,
        duration_ms: Option<i64>,
        latency_ms: Option<i64>,
        model: Option<String>,
        failed_wav_path: Option<String>,
        source_app: Option<String>,
        reason: &str,
    ) {
        self.persist(
            engine,
            language,
            duration_ms,
            latency_ms,
            model,
            "",
            "failed",
            Some(reason),
            failed_wav_path,
            source_app,
        )
        .await;
    }

    #[allow(clippy::too_many_arguments)]
    async fn persist(
        &self,
        engine: Engine,
        language: crate::types::Language,
        duration_ms: Option<i64>,
        latency_ms: Option<i64>,
        model: Option<String>,
        text: &str,
        status: &str,
        failure_reason: Option<&str>,
        failed_wav_path: Option<String>,
        source_app: Option<String>,
    ) {
        let Some(history) = &self.history else {
            return;
        };
        let entry = NewHistoryEntry {
            engine: match engine {
                Engine::Local => "local".into(),
                Engine::Groq => "groq".into(),
            },
            language: language.code().to_string(),
            model,
            duration_ms,
            latency_ms,
            text: text.to_string(),
            status: status.to_string(),
            failure_reason: failure_reason.map(str::to_string),
            failed_wav_path,
            source_app,
        };
        match history.insert(&entry) {
            Ok(_) => self.emitter.emit_history_changed(),
            Err(e) => log::warn!("history insert failed: {e}"),
        }
    }

    async fn emit_error(&self, message: String) {
        self.emit(TranscriptionState::Error { message });
        self.reset_session().await;
    }

    async fn reset_session(&self) {
        self.session.lock().await.reset();
    }
}

/// Given WAV bytes in our canonical format (16 kHz mono f32), returns the
/// playback duration in milliseconds. Returns None if the byte count is too
/// small to contain even the RIFF header.
fn wav_duration_ms(wav: &[u8]) -> Option<i64> {
    const HEADER_SIZE: usize = 44;
    const BYTES_PER_SAMPLE: usize = 4; // f32
    const SAMPLE_RATE: u64 = 16_000;
    if wav.len() <= HEADER_SIZE {
        return None;
    }
    let audio_bytes = (wav.len() - HEADER_SIZE) as u64;
    let samples = audio_bytes / BYTES_PER_SAMPLE as u64;
    Some(((samples * 1000) / SAMPLE_RATE) as i64)
}
