//! LocalEngine — spawns whisper-cli (bundled as a Tauri sidecar) as a
//! subprocess. Stdin/stdout are not used; communication happens via temp files.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::timeout;

use super::{TranscriptionEngine, TranscriptionRequest, TranscriptionResult};
use crate::config::Config;
use crate::error::TranscriptionError;
use crate::paths;

/// RAII guard that best-effort deletes a list of files on drop. We use it
/// instead of scattering `tokio::fs::remove_file` calls before every return.
struct TempFiles {
    paths: Vec<PathBuf>,
}

impl Drop for TempFiles {
    fn drop(&mut self) {
        for p in &self.paths {
            let _ = std::fs::remove_file(p);
        }
    }
}

pub struct LocalEngine {
    pub sidecar_dir: PathBuf,
    pub sidecar_exe: PathBuf,
    /// Shared config — the local model to use is read from here on every
    /// transcribe call so runtime changes (Settings / Modelos page) take
    /// effect without restarting the app. Before this was a baked-in
    /// `model_path: PathBuf` resolved at construction, which meant switching
    /// models silently kept using the startup choice.
    pub config: Arc<RwLock<Config>>,
    pub timeout_secs: u64,
}

impl LocalEngine {
    /// Builds the engine by locating the target-triple-suffixed sidecar
    /// executable inside `sidecar_dir`.
    pub fn new(sidecar_dir: PathBuf, config: Arc<RwLock<Config>>, timeout_secs: u64) -> Self {
        let exe_name = resolve_sidecar_name();
        let sidecar_exe = sidecar_dir.join(&exe_name);
        Self {
            sidecar_dir,
            sidecar_exe,
            config,
            timeout_secs,
        }
    }

    /// Convenience constructor with default 60-second timeout.
    pub fn from_dir(sidecar_dir: PathBuf, config: Arc<RwLock<Config>>) -> Self {
        Self::new(sidecar_dir, config, 60)
    }

    /// Resolves the current model path from the shared config on every call.
    async fn current_model_path(&self) -> PathBuf {
        let name = self.config.read().await.local_model_name.clone();
        paths::models_dir().join(format!("{name}.bin"))
    }
}

fn resolve_sidecar_name() -> String {
    // Tauri's externalBin convention: <name>-<target-triple>[.exe]
    // We hard-code the target triples that match our `scripts/download-whisper-cli.*`.
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "whisper-cli-x86_64-pc-windows-msvc.exe".to_string()
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "whisper-cli-aarch64-apple-darwin".to_string()
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "whisper-cli-x86_64-apple-darwin".to_string()
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "whisper-cli-x86_64-unknown-linux-gnu".to_string()
    } else {
        // Fallback: no suffix; let the caller feel the error.
        "whisper-cli".to_string()
    }
}

#[async_trait]
impl TranscriptionEngine for LocalEngine {
    async fn transcribe(
        &self,
        req: TranscriptionRequest<'_>,
    ) -> Result<TranscriptionResult, TranscriptionError> {
        if !self.sidecar_exe.exists() {
            return Err(TranscriptionError::NotConfigured(format!(
                "whisper-cli not found at {:?}",
                self.sidecar_exe
            )));
        }
        let model_path = self.current_model_path().await;
        if !model_path.exists() {
            return Err(TranscriptionError::ModelNotFound(format!(
                "{:?}",
                model_path
            )));
        }

        let id = uuid::Uuid::new_v4();
        let temp_dir = std::env::temp_dir();
        let wav_path = temp_dir.join(format!("quill-{id}.wav"));
        // whisper-cli appends .txt to --output-file's basename, yielding
        // quill-<id>.txt (we pass the basename without .wav to avoid the
        // quill-<id>.wav.txt double extension).
        let output_prefix = temp_dir.join(format!("quill-{id}"));
        let txt_path = temp_dir.join(format!("quill-{id}.txt"));

        // Any path that returns from here on cleans up these files via Drop.
        let _cleanup = TempFiles {
            paths: vec![wav_path.clone(), txt_path.clone()],
        };

        tokio::fs::write(&wav_path, req.audio_wav).await?;

        log::info!(
            "whisper-cli transcribe: model={} lang={} prompt_len={}",
            model_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("?"),
            req.language.code(),
            req.prompt.map(|p| p.len()).unwrap_or(0)
        );

        let mut cmd = Command::new(&self.sidecar_exe);
        cmd.current_dir(&self.sidecar_dir)
            .arg("-m")
            .arg(&model_path)
            .arg("-l")
            .arg(req.language.code())
            .arg("-f")
            .arg(&wav_path)
            .arg("--output-txt")
            .arg("--output-file")
            .arg(&output_prefix);

        if let Some(prompt) = req.prompt {
            if !prompt.is_empty() {
                cmd.arg("--prompt").arg(prompt);
            }
        }

        // Suppress stdout — whisper-cli is noisy. `kill_on_drop` ensures the
        // subprocess is cleaned up if we abandon the future (e.g. on timeout).
        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let started = Instant::now();
        let child = cmd
            .spawn()
            .map_err(|e| TranscriptionError::SubprocessFailed {
                code: -1,
                stderr: format!("spawn failed: {e}"),
            })?;

        let output = match timeout(
            Duration::from_secs(self.timeout_secs),
            child.wait_with_output(),
        )
        .await
        {
            Ok(r) => r.map_err(|e| TranscriptionError::SubprocessFailed {
                code: -1,
                stderr: e.to_string(),
            })?,
            Err(_) => {
                // Timeout — the dropped future kills the child via kill_on_drop.
                return Err(TranscriptionError::Timeout {
                    seconds: self.timeout_secs,
                });
            }
        };

        if !output.status.success() {
            return Err(TranscriptionError::SubprocessFailed {
                code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let text = tokio::fs::read_to_string(&txt_path)
            .await
            .map_err(TranscriptionError::Io)?;

        Ok(TranscriptionResult {
            text: text.trim().to_string(),
            latency_ms: started.elapsed().as_millis() as u64,
            model: file_stem(&model_path),
        })
    }
}

fn file_stem(p: &Path) -> String {
    p.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}
