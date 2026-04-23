//! Commands for enumerating local resources: mic devices and downloaded models.

use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;
use tauri::AppHandle;

use crate::error::{QuillError, SerializableError};
use crate::paths;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct MicDevice {
    pub name: String,
    pub is_default: bool,
}

#[tauri::command]
pub async fn list_mic_devices() -> Result<Vec<MicDevice>, SerializableError> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    let devices = host
        .input_devices()
        .map_err(|e| SerializableError::from(QuillError::Audio(e.to_string())))?;

    let mut out = Vec::new();
    for d in devices {
        if let Ok(name) = d.name() {
            let is_default = name == default_name;
            out.push(MicDevice { name, is_default });
        }
    }
    Ok(out)
}

#[derive(Debug, Serialize)]
pub struct LocalModel {
    pub name: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub async fn list_local_models() -> Result<Vec<LocalModel>, SerializableError> {
    let dir = paths::models_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    let entries =
        std::fs::read_dir(&dir).map_err(|e| SerializableError::from(QuillError::Io(e)))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("bin") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(LocalModel { name, size_bytes });
    }

    // Stable order: alphabetical.
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Starts streaming live mic level as `mic_level` Tauri events (f32, 0..1).
/// Idempotent — calling while already running replaces the previous probe.
#[tauri::command]
pub async fn start_mic_test(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    device: Option<String>,
) -> Result<(), SerializableError> {
    state.mic_test.lock().unwrap().start(app, device);
    Ok(())
}

/// Stops the live mic level probe. Safe to call when nothing is running.
#[tauri::command]
pub async fn stop_mic_test(state: tauri::State<'_, AppState>) -> Result<(), SerializableError> {
    state.mic_test.lock().unwrap().stop();
    Ok(())
}
