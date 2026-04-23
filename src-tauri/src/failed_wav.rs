//! Preservation of WAV bytes when a transcription fails, so the user can retry.
//!
//! Files are written to `~/.quill/failed/{uuid}.wav` and swept on app start
//! if older than [`RETENTION`].

use std::path::PathBuf;
use std::time::Duration;

use crate::paths;

/// How long to keep a failed WAV around before the next startup sweep removes it.
pub const RETENTION: Duration = Duration::from_secs(24 * 60 * 60);

/// Writes `bytes` to a new file in `~/.quill/failed/` and returns the path.
/// The caller should persist the returned path on the history row so a retry
/// can find it later.
pub fn save(bytes: &[u8]) -> std::io::Result<PathBuf> {
    let dir = paths::failed_dir();
    std::fs::create_dir_all(&dir)?;
    let name = format!("{}.wav", uuid::Uuid::new_v4());
    let path = dir.join(name);
    std::fs::write(&path, bytes)?;
    Ok(path)
}

/// Deletes every file in `~/.quill/failed/` whose mtime is older than `max_age`.
/// Errors on individual files are logged but do not abort the sweep.
pub fn cleanup_older_than(max_age: Duration) {
    let dir = paths::failed_dir();
    let entries = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(e) => {
            log::debug!("failed-dir sweep skipped: {e}");
            return;
        }
    };
    let now = std::time::SystemTime::now();
    let mut removed = 0usize;
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let Ok(modified) = meta.modified() else { continue };
        let Ok(age) = now.duration_since(modified) else {
            continue;
        };
        if age > max_age {
            match std::fs::remove_file(entry.path()) {
                Ok(_) => removed += 1,
                Err(e) => log::warn!("failed to delete stale wav {:?}: {e}", entry.path()),
            }
        }
    }
    if removed > 0 {
        log::info!("failed-dir sweep removed {removed} stale WAV(s)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::{Duration, SystemTime};

    #[test]
    fn save_then_read_roundtrips() {
        let bytes = b"RIFF....WAVE....fake".to_vec();
        let path = save(&bytes).expect("save");
        let read_back = fs::read(&path).expect("read");
        assert_eq!(read_back, bytes);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn cleanup_removes_only_old_files() {
        let dir = paths::failed_dir();
        fs::create_dir_all(&dir).unwrap();
        let old = dir.join(format!("old-{}.wav", uuid::Uuid::new_v4()));
        let fresh = dir.join(format!("fresh-{}.wav", uuid::Uuid::new_v4()));
        fs::write(&old, b"x").unwrap();
        fs::write(&fresh, b"y").unwrap();

        // Backdate `old` to before the retention window. `File::set_modified`
        // stabilized in 1.75 and quill's MSRV is 1.77.
        let past = SystemTime::now() - Duration::from_secs(48 * 3600);
        let Ok(f) = File::options().write(true).open(&old) else {
            let _ = fs::remove_file(&old);
            let _ = fs::remove_file(&fresh);
            return;
        };
        if f.set_modified(past).is_err() {
            let _ = fs::remove_file(&old);
            let _ = fs::remove_file(&fresh);
            return;
        }
        drop(f);

        cleanup_older_than(Duration::from_secs(24 * 3600));
        assert!(!old.exists(), "stale wav should be removed");
        assert!(fresh.exists(), "fresh wav should stay");
        let _ = fs::remove_file(&fresh);
    }
}
