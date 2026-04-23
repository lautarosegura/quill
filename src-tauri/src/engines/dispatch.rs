//! DispatchingEngine — picks Local or Groq at runtime based on current Config.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{
    groq::GroqEngine, local::LocalEngine, TranscriptionEngine, TranscriptionRequest,
    TranscriptionResult,
};
use crate::config::Config;
use crate::error::TranscriptionError;
use crate::types::Engine;

pub struct DispatchingEngine {
    pub local: Arc<LocalEngine>,
    pub groq: RwLock<Option<Arc<GroqEngine>>>,
    pub config: Arc<RwLock<Config>>,
}

impl DispatchingEngine {
    pub fn new(local: Arc<LocalEngine>, config: Arc<RwLock<Config>>) -> Self {
        Self {
            local,
            groq: RwLock::new(None),
            config,
        }
    }

    /// Replace (or clear) the Groq engine. Called by commands when the user
    /// sets/deletes their API key.
    pub async fn set_groq(&self, engine: Option<Arc<GroqEngine>>) {
        *self.groq.write().await = engine;
    }
}

#[async_trait]
impl TranscriptionEngine for DispatchingEngine {
    async fn transcribe(
        &self,
        req: TranscriptionRequest<'_>,
    ) -> Result<TranscriptionResult, TranscriptionError> {
        let engine_choice = self.config.read().await.engine;
        match engine_choice {
            Engine::Local => self.local.transcribe(req).await,
            Engine::Groq => {
                let groq = self.groq.read().await.clone();
                match groq {
                    Some(e) => e.transcribe(req).await,
                    None => Err(TranscriptionError::NotConfigured(
                        "Groq API key no configurada. Abrí Ajustes → API Key de Groq.".into(),
                    )),
                }
            }
        }
    }
}
