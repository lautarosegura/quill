//! Commands for the first-run wizard lifecycle.

use tauri::{AppHandle, Manager};

use crate::error::SerializableError;

/// Called from the wizard's final "Terminar" button. Hides the wizard window
/// and shows the main window. Assumes the frontend already persisted config
/// + API key via their own commands (save_config, set_groq_key) before
/// invoking this.
#[tauri::command]
pub async fn finish_wizard(app: AppHandle) -> Result<(), SerializableError> {
    if let Some(wizard) = app.get_webview_window("wizard") {
        let _ = wizard.hide();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
    Ok(())
}
