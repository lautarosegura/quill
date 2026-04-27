pub mod audio;
pub mod beeps;
pub mod circuit_breaker;
pub mod commands;
pub mod config;
pub mod cost_alert;
pub mod display_server;
pub mod engines;
pub mod error;
pub mod events;
pub mod failed_wav;
pub mod focused_window;
pub mod hardware;
pub mod history;
pub mod hotkey;
pub mod injector;
pub mod mic_test;
pub mod models;
pub mod orchestrator;
pub mod paths;
pub mod post_process;
pub mod secrets;
pub mod tray;
pub mod types;
pub mod usage;
#[cfg(target_os = "linux")]
pub mod wayland_paste;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::Manager;
use tokio::sync::RwLock;

use std::sync::Mutex as StdMutex;

use crate::audio::AudioRecorder;
use crate::config::{Config, ConfigStore};
use crate::engines::dispatch::DispatchingEngine;
use crate::engines::groq::GroqEngine;
use crate::engines::local::LocalEngine;
use crate::engines::TranscriptionEngine;
use crate::history::HistoryStore;
use crate::hotkey::HotkeyManager;
use crate::mic_test::MicTestController;
use crate::orchestrator::{AppOrchestrator, EventEmitter, Notifier};
use crate::secrets::SecretStore;

/// Shared Tauri state accessible from all commands. Contains everything
/// commands need to mutate: the live config and the dispatching engine.
pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub dispatch: Arc<DispatchingEngine>,
    pub history: Arc<HistoryStore>,
    /// Live-mic probe for the Settings VU meter. Commands start/stop it;
    /// the controller owns a std thread + cpal input stream internally.
    pub mic_test: Arc<StdMutex<MicTestController>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::save_config,
            commands::config::is_first_run,
            commands::config::get_default_hotkey,
            commands::dictation::trigger_test_dictation,
            commands::groq::get_groq_key_masked,
            commands::groq::set_groq_key,
            commands::groq::delete_groq_key,
            commands::groq::test_groq_key,
            commands::groq::list_groq_models,
            commands::groq::refresh_groq_engine,
            commands::groq::set_groq_model,
            commands::devices::list_mic_devices,
            commands::devices::list_local_models,
            commands::devices::start_mic_test,
            commands::devices::stop_mic_test,
            commands::history::list_history,
            commands::history::search_history,
            commands::history::delete_history_entry,
            commands::history::clear_all_history,
            commands::history::count_history,
            commands::history::reinject_history_entry,
            commands::history::retry_history_entry,
            commands::hardware::detect_hardware,
            commands::platform::get_display_server,
            commands::platform::get_linux_environment,
            commands::models::list_known_models,
            commands::models::download_model,
            commands::models::delete_model,
            commands::usage::get_usage_stats,
            commands::wizard::finish_wizard,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            paths::ensure_quill_home()?;

            // Sweep stale failed WAVs in the background (best-effort). Files
            // older than 24h are deleted so the directory can't grow forever
            // if the user never clicks "Reintentar".
            tauri::async_runtime::spawn(async {
                failed_wav::cleanup_older_than(failed_wav::RETENTION);
            });

            // Reconcile OS autostart with the persisted `start_on_boot` toggle.
            // Runs once on startup so toggling the setting while the app was
            // closed still takes effect on the next launch.
            {
                let app_handle = app.handle().clone();
                let boot_wanted = ConfigStore::load()
                    .ok()
                    .flatten()
                    .map(|c| c.start_on_boot)
                    .unwrap_or(false);
                commands::config::sync_autostart(&app_handle, boot_wanted);
            }

            tray::build(app.handle())?;
            tray::spawn_state_listener(app.handle());

            // First-run detection: show the wizard if either the config
            // doesn't exist OR the user hasn't completed the wizard yet
            // (wizard_version == 0). The latter matters because we now persist
            // the draft config at Step 4 → Step 5 transition so the live
            // dictation test works — we don't want that partial save to
            // suppress the wizard on the next launch.
            let is_first_run = match ConfigStore::load() {
                Ok(Some(cfg)) => cfg.wizard_version < 1,
                _ => true,
            };

            if let Some(main) = app.get_webview_window("main") {
                // Always wire close-to-tray. Only show if NOT first run.
                let main_clone = main.clone();
                main.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = main_clone.hide();
                    }
                });
                if !is_first_run {
                    main.show()?;
                }
            }

            if is_first_run {
                if let Some(wizard) = app.get_webview_window("wizard") {
                    wizard.show()?;
                    wizard.set_focus()?;
                }
            }

            // Position the overlay according to the user's saved preference.
            // Re-applied from `save_config` whenever `overlay_position` changes.
            {
                let pos = ConfigStore::load()
                    .ok()
                    .flatten()
                    .map(|c| c.overlay_position)
                    .unwrap_or(crate::types::OverlayPosition::BottomCenter);
                reposition_overlay(app.handle(), pos);
            }

            // Bootstrap: the engine pipeline (no mic) is registered synchronously
            // so AppState is available to commands the moment the frontend calls.
            // Only the AudioRecorder + orchestrator start in the background spawn.
            let sidecar_dir = resolve_sidecar_dir(app);
            let loaded = ConfigStore::load().unwrap_or(None).unwrap_or_default();
            let config: Arc<RwLock<Config>> = Arc::new(RwLock::new(loaded));
            // Pass the shared config to LocalEngine so it re-reads
            // local_model_name on every transcribe — switching the model
            // from Settings/Modelos takes effect without a restart.
            let local_engine = Arc::new(LocalEngine::from_dir(sidecar_dir, Arc::clone(&config)));
            let dispatch = Arc::new(DispatchingEngine::new(local_engine, Arc::clone(&config)));

            // Pre-populate Groq engine if a key is in the keychain.
            {
                let dispatch = Arc::clone(&dispatch);
                let groq_model = {
                    let cfg_guard =
                        tauri::async_runtime::block_on(async { config.read().await.clone() });
                    cfg_guard.groq_model
                };
                tauri::async_runtime::block_on(async move {
                    if let Ok(Some(key)) = SecretStore::get_groq_key() {
                        if let Ok(g) = GroqEngine::new_with_model(key, groq_model) {
                            dispatch.set_groq(Some(Arc::new(g))).await;
                        }
                    }
                });
            }

            // Open the history DB. If this fails we log but don't abort — the
            // user can still dictate, just without persistence.
            let history = match HistoryStore::open(&paths::history_db()) {
                Ok(h) => Arc::new(h),
                Err(e) => {
                    log::error!("failed to open history db: {e}");
                    // Fallback: an in-memory store so other code paths still work.
                    Arc::new(
                        HistoryStore::open(std::path::Path::new(":memory:"))
                            .expect("in-memory sqlite must work"),
                    )
                }
            };

            app.manage(AppState {
                config: Arc::clone(&config),
                dispatch: Arc::clone(&dispatch),
                history: Arc::clone(&history),
                mic_test: Arc::new(StdMutex::new(MicTestController::new())),
            });

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = bootstrap_dictation(app_handle, dispatch, config, history).await {
                    log::error!("dictation bootstrap failed: {e}");
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Finds the directory that actually contains the whisper-cli sidecar binary
/// (not just any `binaries/` folder). Tauri's `resources` configuration copies
/// DLLs into `target/<profile>/binaries/` during dev, so the mere existence of
/// that directory isn't enough — we probe for the target-triple-suffixed exe.
fn resolve_sidecar_dir(app: &tauri::App) -> PathBuf {
    let exe_name = sidecar_exe_name();

    // Candidates, in priority order.
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("binaries"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("src-tauri").join("binaries"));
        candidates.push(cwd.join("binaries"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("binaries"));
            // `tauri dev` runs exe from target/<profile>/; sidecar source stays in src-tauri/binaries.
            if let Some(grandparent) = parent.parent() {
                if let Some(great) = grandparent.parent() {
                    candidates.push(great.join("src-tauri").join("binaries"));
                    candidates.push(great.join("binaries"));
                }
            }
        }
    }

    for c in &candidates {
        if c.join(&exe_name).exists() {
            log::info!("sidecar found at {}", c.display());
            return c.clone();
        }
    }

    log::warn!(
        "sidecar exe '{}' not found in any candidate: {:?}",
        exe_name,
        candidates
    );
    candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| PathBuf::from("src-tauri/binaries"))
}

fn sidecar_exe_name() -> String {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "whisper-cli-x86_64-pc-windows-msvc.exe".to_string()
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "whisper-cli-aarch64-apple-darwin".to_string()
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "whisper-cli-x86_64-apple-darwin".to_string()
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "whisper-cli-x86_64-unknown-linux-gnu".to_string()
    } else {
        "whisper-cli".to_string()
    }
}

async fn bootstrap_dictation(
    app: tauri::AppHandle,
    dispatch: Arc<DispatchingEngine>,
    config: Arc<RwLock<Config>>,
    history: Arc<HistoryStore>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let recorder = Arc::new(AudioRecorder::new()?);

    let emitter: Arc<dyn EventEmitter> = Arc::new(app.clone());
    let notifier: Arc<dyn Notifier> = Arc::new(app.clone());
    let engine: Arc<dyn TranscriptionEngine> = dispatch;
    let orchestrator = Arc::new(
        AppOrchestrator::new(emitter, engine, recorder, Arc::clone(&config))
            .with_history(history)
            .with_notifier(notifier),
    );

    let (hotkey_mgr, hotkey_rx) = HotkeyManager::start(Arc::clone(&config));
    // rdev's listener thread has no clean stop — hold the manager for the
    // lifetime of the app.
    std::mem::forget(hotkey_mgr);

    spawn_overlay_visibility(app.clone());
    #[cfg(target_os = "linux")]
    spawn_wayland_state_notifier(app.clone());
    beeps::spawn_beep_listener(app.clone(), Arc::clone(&config));

    tauri::async_runtime::spawn(async move {
        orchestrator.run(hotkey_rx).await;
    });

    Ok(())
}

/// Places the overlay window at the user's configured corner/edge of the
/// primary monitor. Called once at startup and again from `save_config`
/// whenever `overlay_position` changes. Silent no-op if the overlay window
/// or monitor info isn't available.
pub fn reposition_overlay(app: &tauri::AppHandle, position: crate::types::OverlayPosition) {
    use crate::types::OverlayPosition;

    let Some(overlay) = app.get_webview_window("overlay") else {
        return;
    };

    // Wayland compositors (especially GNOME) silently ignore `set_position`
    // for regular toplevel windows. Worse, on multi-monitor setups our
    // calculated position can land on a monitor the user isn't looking at,
    // making the overlay appear to "not work". Skip the call entirely on
    // Wayland and let the compositor place the overlay where it deems
    // appropriate (typically the focused monitor's center). The tray
    // tooltip is the reliable feedback channel on those sessions.
    #[cfg(target_os = "linux")]
    {
        if crate::display_server::DisplayServer::detect().is_wayland() {
            log::debug!("reposition_overlay: skipping on Wayland (compositor-managed placement)");
            return;
        }
    }

    let Ok(Some(monitor)) = overlay.current_monitor() else {
        return;
    };
    let mon_size = monitor.size();
    let mon_pos = monitor.position();
    const OVERLAY_W: i32 = 220;
    const OVERLAY_H: i32 = 48;
    const EDGE_MARGIN: i32 = 24;
    const BOTTOM_MARGIN: i32 = 80; // sits above the taskbar / dock

    let mid_x = mon_pos.x + ((mon_size.width as i32) - OVERLAY_W) / 2;
    let right_x = mon_pos.x + (mon_size.width as i32) - OVERLAY_W - EDGE_MARGIN;
    let left_x = mon_pos.x + EDGE_MARGIN;
    let top_y = mon_pos.y + EDGE_MARGIN;
    let bottom_y = mon_pos.y + (mon_size.height as i32) - OVERLAY_H - BOTTOM_MARGIN;

    let (x, y) = match position {
        OverlayPosition::TopLeft => (left_x, top_y),
        OverlayPosition::TopCenter => (mid_x, top_y),
        OverlayPosition::TopRight => (right_x, top_y),
        OverlayPosition::BottomLeft => (left_x, bottom_y),
        OverlayPosition::BottomCenter => (mid_x, bottom_y),
        OverlayPosition::BottomRight => (right_x, bottom_y),
    };
    let _ = overlay.set_position(tauri::PhysicalPosition::new(x, y));
}

fn spawn_overlay_visibility(app: tauri::AppHandle) {
    use tauri::Listener;

    let visibility_app = app.clone();
    app.listen(events::TRANSCRIPTION_STATE_CHANGED, move |event| {
        let Ok(state) = serde_json::from_str::<events::TranscriptionState>(event.payload()) else {
            return;
        };
        let Some(overlay) = visibility_app.get_webview_window("overlay") else {
            return;
        };
        match state {
            events::TranscriptionState::Idle => {
                let _ = overlay.hide();
            }
            _ => {
                // Window is configured with focus: false in tauri.conf.json,
                // so show() does not steal focus from the user's target app.
                let _ = overlay.show();
            }
        }
    });
}

/// Hybrid feedback layer for Wayland sessions: the pill overlay still shows,
/// but its position is compositor-managed and easy to miss. For the two
/// states where the user has to *act* — ClipboardOnly (press Ctrl+V) and
/// Error (something failed) — we also fire a native desktop notification
/// via the Tauri notification plugin. On Linux that goes through
/// `org.freedesktop.Notifications` D-Bus, so it lands in the user's
/// regular notification stack regardless of where the pill ended up on
/// screen.
///
/// No-op on X11 (where the pill is reliably positioned) and on non-Linux.
#[cfg(target_os = "linux")]
fn spawn_wayland_state_notifier(app: tauri::AppHandle) {
    use tauri::Listener;

    if !crate::display_server::DisplayServer::detect().is_wayland() {
        return;
    }

    let notifier_app = app.clone();
    app.listen(events::TRANSCRIPTION_STATE_CHANGED, move |event| {
        use tauri_plugin_notification::NotificationExt;

        let Ok(state) = serde_json::from_str::<events::TranscriptionState>(event.payload()) else {
            return;
        };

        let (title, body) = match state {
            events::TranscriptionState::ClipboardOnly { .. } => (
                "Quill",
                "Texto copiado al portapapeles. Apretá Ctrl+V para pegar.".to_string(),
            ),
            events::TranscriptionState::Error { message } => ("Quill — error", message),
            // Recording / Transcribing / Injecting / Cancelled / Idle would
            // be too noisy as native notifications — the pill (or tray
            // tooltip) is enough for those.
            _ => return,
        };

        if let Err(e) = notifier_app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
        {
            log::warn!("wayland state notification failed: {e}");
        }
    });
}
