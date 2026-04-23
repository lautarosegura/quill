//! System tray icon — visible while the MainWindow is closed or minimized.
//! Phase 3: basic menu (Open/Quit). Phase 6: state-driven icon colors that
//! reflect the current TranscriptionState (idle/recording/error).

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Listener, Manager,
};

use crate::events;

// Embed raw RGBA buffers for the three tray states. 32x32 * 4 bytes = 4096.
// We use raw RGBA (not PNG) so we don't need the `image-png` cargo feature
// on `tauri`. The tools/make_icons.ps1 script produces these alongside the
// PNG variants used for bundling.
const TRAY_IDLE_RGBA: &[u8] = include_bytes!("../icons/tray-idle.rgba");
const TRAY_RECORDING_RGBA: &[u8] = include_bytes!("../icons/tray-recording.rgba");
const TRAY_ERROR_RGBA: &[u8] = include_bytes!("../icons/tray-error.rgba");
const TRAY_ICON_SIZE: u32 = 32;

/// Stable ID for the tray icon. Used again by `spawn_state_listener` to
/// look up the live icon and swap its image when state changes.
const TRAY_ID: &str = "quill-tray";

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir Quill", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    // Start with the Idle icon (muted gray).
    let initial_icon = Image::new(TRAY_IDLE_RGBA, TRAY_ICON_SIZE, TRAY_ICON_SIZE);

    TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Quill")
        .icon(initial_icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// Subscribe to `TRANSCRIPTION_STATE_CHANGED` and repaint the tray icon so the
/// user can see at a glance whether Quill is idle, recording, working, or in
/// an error state — even when the main window is hidden.
///
/// Call once, after `build`, from the Tauri setup closure.
pub fn spawn_state_listener(app: &AppHandle) {
    let handle = app.clone();
    app.listen(events::TRANSCRIPTION_STATE_CHANGED, move |event| {
        let Ok(state) = serde_json::from_str::<events::TranscriptionState>(event.payload()) else {
            return;
        };

        // Map state -> RGBA buffer.
        // Transcribing/Injecting keep the "recording" red so the tray reads as
        // "busy" until the whole round-trip finishes (matches the overlay).
        let rgba: &[u8] = match state {
            // Idle + Cancelled both read as "nothing happening" in the tray.
            events::TranscriptionState::Idle
            | events::TranscriptionState::Cancelled => TRAY_IDLE_RGBA,
            events::TranscriptionState::Recording
            | events::TranscriptionState::Transcribing
            | events::TranscriptionState::Injecting => TRAY_RECORDING_RGBA,
            events::TranscriptionState::Error { .. } => TRAY_ERROR_RGBA,
        };

        let Some(tray) = handle.tray_by_id(TRAY_ID) else {
            log::warn!("tray icon '{TRAY_ID}' not found when updating state");
            return;
        };

        let img = Image::new(rgba, TRAY_ICON_SIZE, TRAY_ICON_SIZE);
        if let Err(e) = tray.set_icon(Some(img)) {
            log::warn!("failed to swap tray icon: {e}");
        }
    });
}

fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
