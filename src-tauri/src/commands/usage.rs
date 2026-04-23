use crate::error::SerializableError;
use crate::usage::{self, UsageStats};
use crate::AppState;

#[tauri::command]
pub async fn get_usage_stats(
    state: tauri::State<'_, AppState>,
) -> Result<UsageStats, SerializableError> {
    usage::compute_for_current_month(&state.history).map_err(SerializableError::from)
}
