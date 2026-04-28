use crate::error::{QuillError, Result};
use crate::types::LlmProvider;

const SERVICE: &str = "quill";
const GROQ_ACCOUNT: &str = "groq_api_key";

pub struct SecretStore;

impl SecretStore {
    pub fn get_groq_key() -> Result<Option<String>> {
        Self::get_at(GROQ_ACCOUNT)
    }

    pub fn set_groq_key(key: &str) -> Result<()> {
        Self::set_at(GROQ_ACCOUNT, key)
    }

    pub fn delete_groq_key() -> Result<()> {
        Self::delete_at(GROQ_ACCOUNT)
    }

    /// Get the API key for an LLM polish provider. Each provider has its
    /// own keychain slot — kept separate from the transcription Groq key
    /// so they can be revoked independently.
    pub fn get_llm_key(provider: LlmProvider) -> Result<Option<String>> {
        Self::get_at(provider.key_id())
    }

    pub fn set_llm_key(provider: LlmProvider, key: &str) -> Result<()> {
        Self::set_at(provider.key_id(), key)
    }

    pub fn delete_llm_key(provider: LlmProvider) -> Result<()> {
        Self::delete_at(provider.key_id())
    }

    fn get_at(account: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(SERVICE, account)
            .map_err(|e| QuillError::Keyring(e.to_string()))?;
        match entry.get_password() {
            Ok(pw) => Ok(Some(pw)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(QuillError::Keyring(e.to_string())),
        }
    }

    fn set_at(account: &str, key: &str) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE, account)
            .map_err(|e| QuillError::Keyring(e.to_string()))?;
        entry
            .set_password(key)
            .map_err(|e| QuillError::Keyring(e.to_string()))
    }

    fn delete_at(account: &str) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE, account)
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
