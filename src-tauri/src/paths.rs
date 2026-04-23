use std::path::PathBuf;

/// Returns `~/.quill/` — the base directory for all app data.
pub fn quill_home() -> PathBuf {
    dirs::home_dir()
        .expect("home dir must exist")
        .join(".quill")
}

pub fn config_file() -> PathBuf {
    quill_home().join("config.json")
}

pub fn history_db() -> PathBuf {
    quill_home().join("history.db")
}

pub fn vocabulary_file() -> PathBuf {
    quill_home().join("vocabulary.txt")
}

pub fn models_dir() -> PathBuf {
    quill_home().join("models")
}

pub fn failed_dir() -> PathBuf {
    quill_home().join("failed")
}

pub fn logs_dir() -> PathBuf {
    quill_home().join("logs")
}

/// Creates the `~/.quill/` tree (models, failed, logs subdirs) if missing.
pub fn ensure_quill_home() -> std::io::Result<()> {
    let home = quill_home();
    std::fs::create_dir_all(&home)?;
    std::fs::create_dir_all(home.join("models"))?;
    std::fs::create_dir_all(home.join("failed"))?;
    std::fs::create_dir_all(home.join("logs"))?;
    Ok(())
}

#[cfg(test)]
mod tests;
