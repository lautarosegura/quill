use crate::error::SerializableError;
use crate::hardware::{self, HardwareProfile};

#[tauri::command]
pub async fn detect_hardware() -> Result<HardwareProfile, SerializableError> {
    Ok(hardware::detect())
}
