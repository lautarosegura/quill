//! ModelManager — lists, downloads, and deletes Whisper models.
//! Downloads are streamed from Hugging Face with optional SHA-256 verification.

use std::path::PathBuf;

use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::error::QuillError;
use crate::paths;

#[derive(Debug, Clone, Copy)]
pub struct ModelInfo {
    pub name: &'static str,
    pub display_name: &'static str,
    pub url: &'static str,
    pub size_bytes: u64,
    /// Optional SHA-256 of the file. `None` means we download without
    /// verification (with a warning log). Phase 6 pins the correct hashes.
    pub sha256: Option<&'static str>,
    pub description: &'static str,
}

/// Known Whisper models, in order of size. Download URLs and approx sizes
/// from https://huggingface.co/ggerganov/whisper.cpp.
pub const KNOWN_MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "ggml-tiny",
        display_name: "Tiny",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin",
        size_bytes: 77_700_000,
        sha256: Some("be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21"),
        description: "Multilingüe básico · ~75 MB · muy rápido en cualquier CPU",
    },
    ModelInfo {
        name: "ggml-base",
        display_name: "Base",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
        size_bytes: 147_900_000,
        sha256: Some("60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe"),
        description: "Balanceado · ~140 MB · sweet spot para push-to-talk en CPU",
    },
    ModelInfo {
        name: "ggml-small",
        display_name: "Small",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
        size_bytes: 487_600_000,
        sha256: Some("1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b"),
        description: "Alta precisión · ~465 MB · recomendado con GPU o Apple Silicon",
    },
    ModelInfo {
        name: "ggml-medium",
        display_name: "Medium",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
        size_bytes: 1_533_800_000,
        sha256: Some("6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208"),
        description: "Muy alta precisión · ~1.5 GB · pesado en CPU-only",
    },
    ModelInfo {
        name: "ggml-large-v3",
        display_name: "Large v3",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin",
        size_bytes: 3_094_400_000,
        sha256: Some("64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2"),
        description: "Máxima precisión · ~3 GB · sólo con GPU robusta",
    },
];

#[derive(Debug, Clone, Serialize)]
pub struct ModelEntry {
    pub name: String,
    pub display_name: String,
    pub size_bytes: u64,
    pub description: String,
    pub installed: bool,
    pub installed_size_bytes: Option<u64>,
}

pub struct ModelManager {
    client: reqwest::Client,
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            models_dir: paths::models_dir(),
        }
    }

    #[cfg(test)]
    pub fn new_with_dir(models_dir: PathBuf) -> Self {
        Self {
            client: reqwest::Client::new(),
            models_dir,
        }
    }

    pub fn list(&self) -> Vec<ModelEntry> {
        KNOWN_MODELS
            .iter()
            .map(|m| {
                let path = self.models_dir.join(format!("{}.bin", m.name));
                let (installed, size) = match std::fs::metadata(&path) {
                    Ok(meta) => (true, Some(meta.len())),
                    Err(_) => (false, None),
                };
                ModelEntry {
                    name: m.name.to_string(),
                    display_name: m.display_name.to_string(),
                    size_bytes: m.size_bytes,
                    description: m.description.to_string(),
                    installed,
                    installed_size_bytes: size,
                }
            })
            .collect()
    }

    pub fn delete(&self, name: &str) -> Result<(), QuillError> {
        let path = self.models_dir.join(format!("{name}.bin"));
        if !path.exists() {
            return Err(QuillError::NotFound(format!("{name}.bin")));
        }
        std::fs::remove_file(&path)?;
        Ok(())
    }

    /// Streams the model file. `on_progress(downloaded, total)` fires roughly
    /// every ~512 KB. Writes to a `.partial` file and renames on success, so a
    /// crashed download never leaves a half-complete `.bin` behind.
    pub async fn download<F>(
        &self,
        name: &str,
        on_progress: F,
    ) -> Result<PathBuf, QuillError>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        let info = KNOWN_MODELS
            .iter()
            .find(|m| m.name == name)
            .copied()
            .ok_or_else(|| QuillError::NotFound(format!("model {name}")))?;

        std::fs::create_dir_all(&self.models_dir)?;
        let final_path = self.models_dir.join(format!("{}.bin", info.name));
        let partial_path = self.models_dir.join(format!("{}.bin.partial", info.name));

        // Wipe any stale partial from a previous failed attempt.
        if partial_path.exists() {
            let _ = std::fs::remove_file(&partial_path);
        }

        let response = self
            .client
            .get(info.url)
            .send()
            .await
            .map_err(|e| QuillError::Other(format!("download request: {e}")))?;

        if !response.status().is_success() {
            return Err(QuillError::Other(format!(
                "download HTTP {}",
                response.status()
            )));
        }

        let total = response.content_length().unwrap_or(info.size_bytes);

        let mut file = tokio::fs::File::create(&partial_path).await?;
        let mut hasher = Sha256::new();
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_reported: u64 = 0;
        const PROGRESS_INTERVAL: u64 = 512 * 1024;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| QuillError::Other(format!("download chunk: {e}")))?;
            file.write_all(&chunk).await?;
            hasher.update(&chunk);
            downloaded += chunk.len() as u64;
            if downloaded - last_reported >= PROGRESS_INTERVAL {
                on_progress(downloaded, total);
                last_reported = downloaded;
            }
        }
        file.flush().await?;
        drop(file);

        // Final progress.
        on_progress(downloaded, total);

        // SHA-256 verification (only if a hash is pinned for this model).
        if let Some(expected) = info.sha256 {
            let got = hex::encode(hasher.finalize());
            if got.eq_ignore_ascii_case(expected) {
                log::info!("SHA-256 verified for {name}");
            } else {
                let _ = std::fs::remove_file(&partial_path);
                return Err(QuillError::Other(format!(
                    "SHA-256 mismatch for {name}: expected {expected}, got {got}"
                )));
            }
        } else {
            log::warn!(
                "No pinned SHA-256 for {name} — download completed without verification"
            );
        }

        std::fs::rename(&partial_path, &final_path)?;
        Ok(final_path)
    }
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
