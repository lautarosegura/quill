//! Integration test for the orchestrator.
//!
//! Requires a working microphone (for AudioRecorder::new). `#[ignore]`d by
//! default so CI doesn't fail on headless runners. Run locally:
//!
//!     cargo test --test orchestrator_flow -- --ignored

use std::sync::Arc;

use async_trait::async_trait;
use quill_lib::audio::AudioRecorder;
use quill_lib::config::Config;
use quill_lib::engines::{TranscriptionEngine, TranscriptionRequest, TranscriptionResult};
use quill_lib::error::TranscriptionError;
use quill_lib::events::TranscriptionState;
use quill_lib::hotkey::HotkeyEvent;
use quill_lib::orchestrator::{AppOrchestrator, EventEmitter};
use quill_lib::types::Language;
use std::sync::Mutex;
use tokio::sync::RwLock;

struct MockEngine {
    response: String,
}

#[async_trait]
impl TranscriptionEngine for MockEngine {
    async fn transcribe(
        &self,
        _req: TranscriptionRequest<'_>,
    ) -> Result<TranscriptionResult, TranscriptionError> {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(TranscriptionResult {
            text: self.response.clone(),
            latency_ms: 10,
            model: "mock".into(),
        })
    }
}

struct RecordingEmitter {
    states: Mutex<Vec<TranscriptionState>>,
}

impl EventEmitter for RecordingEmitter {
    fn emit_state(&self, state: &TranscriptionState) {
        self.states.lock().unwrap().push(state.clone());
    }
    fn emit_history_changed(&self) {
        // No-op in tests — the history side effect is observed by reading the
        // store directly, not by consuming events.
    }
}

fn make_config(language: Language, min_duration_ms: u32) -> Arc<RwLock<Config>> {
    Arc::new(RwLock::new(Config {
        language,
        min_duration_ms,
        ..Config::default()
    }))
}

#[tokio::test]
#[ignore = "AudioRecorder requires a microphone — run locally with --ignored"]
async fn orchestrator_happy_path_emits_expected_states() {
    let recorder = Arc::new(AudioRecorder::new().expect("mic required"));
    let engine = Arc::new(MockEngine {
        response: "hola mundo".into(),
    });
    let emitter = Arc::new(RecordingEmitter {
        states: Mutex::new(Vec::new()),
    });
    let config = make_config(Language::Es, 250);

    let orch = Arc::new(AppOrchestrator::new(
        emitter.clone() as Arc<dyn EventEmitter>,
        engine,
        recorder,
        config,
    ));

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let run_handle = tokio::spawn({
        let orch = Arc::clone(&orch);
        async move {
            orch.run(rx).await;
        }
    });

    tx.send(HotkeyEvent::Pressed { source_app: None }).unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    tx.send(HotkeyEvent::Released { held_ms: 400 }).unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    drop(tx);
    let _ = tokio::time::timeout(std::time::Duration::from_secs(1), run_handle).await;

    let states = emitter.states.lock().unwrap();
    assert!(states.contains(&TranscriptionState::Recording));
    assert!(states.contains(&TranscriptionState::Transcribing));
    let terminal_ok = states.iter().any(|s| {
        matches!(
            s,
            TranscriptionState::Idle | TranscriptionState::Error { .. }
        )
    });
    assert!(terminal_ok, "expected Idle or Error in {states:?}");
}

#[tokio::test]
#[ignore = "AudioRecorder requires a microphone — run locally with --ignored"]
async fn orchestrator_short_tap_ignored() {
    let recorder = Arc::new(AudioRecorder::new().expect("mic required"));
    let engine = Arc::new(MockEngine {
        response: "never gets called".into(),
    });
    let emitter = Arc::new(RecordingEmitter {
        states: Mutex::new(Vec::new()),
    });
    let config = make_config(Language::Es, 250);

    let orch = Arc::new(AppOrchestrator::new(
        emitter.clone() as Arc<dyn EventEmitter>,
        engine,
        recorder,
        config,
    ));

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let run_handle = tokio::spawn({
        let orch = Arc::clone(&orch);
        async move {
            orch.run(rx).await;
        }
    });

    tx.send(HotkeyEvent::Pressed { source_app: None }).unwrap();
    tx.send(HotkeyEvent::Released { held_ms: 100 }).unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    drop(tx);
    let _ = tokio::time::timeout(std::time::Duration::from_secs(1), run_handle).await;

    let states = emitter.states.lock().unwrap();
    assert!(states.contains(&TranscriptionState::Recording));
    assert!(states.contains(&TranscriptionState::Idle));
    assert!(!states.contains(&TranscriptionState::Transcribing));
    assert!(!states.contains(&TranscriptionState::Injecting));
}
