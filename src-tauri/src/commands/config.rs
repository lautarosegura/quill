use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::config::{Config, ConfigStore};
use crate::error::SerializableError;
use crate::paths;
use crate::types::Keybind;
use crate::AppState;

#[tauri::command]
pub async fn get_config(
    state: tauri::State<'_, AppState>,
) -> Result<Config, SerializableError> {
    // Read from the live, shared config — runtime mutations by other
    // commands are visible to the frontend.
    let cfg = state.config.read().await.clone();
    Ok(cfg)
}

#[tauri::command]
pub async fn save_config(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    config: Config,
) -> Result<(), SerializableError> {
    paths::ensure_quill_home()
        .map_err(crate::error::QuillError::Io)
        .map_err(SerializableError::from)?;

    // Update shared state FIRST so every module sees the new values on the
    // next tick. Then persist to disk.
    *state.config.write().await = config.clone();
    ConfigStore::save(&config).map_err(SerializableError::from)?;

    // Keep the OS autostart entry in sync with the toggle. Errors here are
    // non-fatal — the rest of the config is already persisted.
    sync_autostart(&app, config.start_on_boot);

    // Re-apply the overlay position in case it changed. Cheap no-op if the
    // user didn't touch that setting (set_position is idempotent with same
    // coordinates). Avoids needing to diff the old vs new config.
    crate::reposition_overlay(&app, config.overlay_position);
    Ok(())
}

/// Idempotent: sets the OS-level autostart state to match `enabled`. Swallows
/// plugin errors (unsupported platform, missing permission) with a warning.
pub fn sync_autostart(app: &AppHandle, enabled: bool) {
    let manager = app.autolaunch();
    let is_on = manager.is_enabled().unwrap_or(false);
    if enabled && !is_on {
        if let Err(e) = manager.enable() {
            log::warn!("autostart enable failed: {e}");
        }
    } else if !enabled && is_on {
        if let Err(e) = manager.disable() {
            log::warn!("autostart disable failed: {e}");
        }
    }
}

#[tauri::command]
pub async fn is_first_run() -> Result<bool, SerializableError> {
    let cfg = ConfigStore::load().map_err(SerializableError::from)?;
    // First run if there's no config OR the wizard was never completed.
    // Mirrors the same check in lib.rs::setup so the frontend can ask
    // "should I go to the wizard" without re-deriving the rule.
    Ok(match cfg {
        Some(c) => c.wizard_version < 1,
        None => true,
    })
}

/// Returns the platform-specific default hotkey. Exposed for the Settings UI
/// so users can one-click "Restaurar default" — useful on Windows where the
/// webview can't capture Ctrl+Win reliably (the OS eats the Meta keyup).
#[tauri::command]
pub async fn get_default_hotkey() -> Result<Keybind, SerializableError> {
    Ok(Keybind::default_push_to_talk())
}
