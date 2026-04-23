//! Commands for the first-run wizard lifecycle.

use tauri::{AppHandle, Manager};

use crate::config::ConfigStore;
use crate::error::SerializableError;
use crate::AppState;

/// Called from the wizard's final "Terminar" button. Hides the wizard window,
/// shows the main window, AND persists `wizard_version = 1` to disk as a
/// belt-and-suspenders guarantee.
///
/// The frontend's `applyDraftToRuntime({ markComplete: true })` also writes
/// `wizard_version = 1` via `save_config`, but if that path fails silently —
/// a stale `config.value` store, a thrown `setGroqKey` halfway through, a
/// window close race, etc. — we'd end up re-showing the wizard on every
/// launch. Mirroring the write here is cheap, idempotent, and removes the
/// whole class of "frontend partially persisted" bugs.
#[tauri::command]
pub async fn finish_wizard(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), SerializableError> {
    {
        let mut cfg = state.config.write().await;
        if cfg.wizard_version < 1 {
            cfg.wizard_version = 1;
            if let Err(e) = ConfigStore::save(&cfg) {
                log::error!("finish_wizard: failed to persist wizard_version=1: {e}");
                // Continue — the window transitions are still worth doing so
                // the user isn't stranded. Next launch will re-show the
                // wizard, which is annoying but not destructive.
            } else {
                log::info!("finish_wizard: wizard_version=1 persisted");
            }
        }
    }

    if let Some(wizard) = app.get_webview_window("wizard") {
        let _ = wizard.hide();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
    Ok(())
}
