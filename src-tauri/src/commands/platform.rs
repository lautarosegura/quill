//! Platform metadata exposed to the frontend. Used by Svelte to render
//! display-server-specific UX (e.g. hide the hotkey key picker on Wayland,
//! show a "Wayland no expone la ventana activa" hint in Historial).

use crate::display_server::DisplayServer;

#[tauri::command]
pub fn get_display_server() -> DisplayServer {
    DisplayServer::detect()
}
