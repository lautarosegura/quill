use crate::error::{QuillError, Result};

const SERVICE: &str = "quill";
const GROQ_ACCOUNT: &str = "groq_api_key";

pub struct SecretStore;

impl SecretStore {
    pub fn get_groq_key() -> Result<Option<String>> {
        let entry = keyring::Entry::new(SERVICE, GROQ_ACCOUNT)
            .map_err(|e| QuillError::Keyring(e.to_string()))?;
        match entry.get_password() {
            Ok(pw) => Ok(Some(pw)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(QuillError::Keyring(e.to_string())),
        }
    }

    pub fn set_groq_key(key: &str) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE, GROQ_ACCOUNT)
            .map_err(|e| QuillError::Keyring(e.to_string()))?;
        entry
            .set_password(key)
            .map_err(|e| QuillError::Keyring(e.to_string()))
    }

    pub fn delete_groq_key() -> Result<()> {
        let entry = keyring::Entry::new(SERVICE, GROQ_ACCOUNT)
            .map_err(|e| QuillError::Keyring(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(QuillError::Keyring(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests;
