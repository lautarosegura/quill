//! Platform metadata exposed to the frontend. Used by Svelte to render
//! display-server-specific UX (e.g. hide the hotkey key picker on Wayland,
//! show a "Wayland no expone la ventana activa" hint in Historial).

use crate::display_server::{DisplayServer, LinuxEnvironment};

#[tauri::command]
pub fn get_display_server() -> DisplayServer {
    DisplayServer::detect()
}

/// Returns the user's Linux compositor + version fingerprint, or `None`
/// when not running on Linux. Used by the wizard to decide whether to
/// surface the "you need to be in the input group" info card.
#[tauri::command]
pub fn get_linux_environment() -> Option<LinuxEnvironment> {
    #[cfg(target_os = "linux")]
    {
        Some(crate::display_server::detect_linux_environment())
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
