//! Tauri commands for the Modelos page.

use tauri::{AppHandle, Emitter};

use crate::error::SerializableError;
use crate::events::{
    ModelDownloadComplete, ModelDownloadError, ModelDownloadProgress,
    MODEL_DOWNLOAD_COMPLETE, MODEL_DOWNLOAD_ERROR, MODEL_DOWNLOAD_PROGRESS,
};
use crate::models::{ModelEntry, ModelManager};

#[tauri::command]
pub async fn list_known_models() -> Result<Vec<ModelEntry>, SerializableError> {
    Ok(ModelManager::new().list())
}

#[tauri::command]
pub async fn delete_model(name: String) -> Result<(), SerializableError> {
    ModelManager::new()
        .delete(&name)
        .map_err(SerializableError::from)
}

#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    name: String,
) -> Result<(), SerializableError> {
    let mgr = ModelManager::new();
    let name_clone = name.clone();
    let app_for_progress = app.clone();

    let result = mgr
        .download(&name, move |downloaded, total| {
            let _ = app_for_progress.emit(
                MODEL_DOWNLOAD_PROGRESS,
                ModelDownloadProgress {
                    name: name_clone.clone(),
                    downloaded,
                    total,
                },
            );
        })
        .await;

    match result {
        Ok(_) => {
            let _ = app.emit(MODEL_DOWNLOAD_COMPLETE, ModelDownloadComplete { name });
            Ok(())
        }
        Err(e) => {
            let message = e.to_string();
            let _ = app.emit(
                MODEL_DOWNLOAD_ERROR,
                ModelDownloadError {
                    name,
                    message: message.clone(),
                },
            );
            Err(SerializableError::from(e))
        }
    }
}
