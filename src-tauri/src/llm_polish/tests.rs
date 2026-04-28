//! Unit tests for `LlmPolishDispatcher` itself. Provider-specific HTTP
//! tests will live in each provider module once Phase B implements the
//! real network calls.

use std::sync::Arc;

use tokio::sync::RwLock;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::{groq::GroqLlmPolisher, LlmPolishDispatcher, LlmPolishError};
use crate::config::Config;
use crate::types::LlmProvider;

fn cfg() -> Arc<RwLock<Config>> {
    Arc::new(RwLock::new(Config::default()))
}

#[tokio::test]
async fn dispatcher_returns_input_when_disabled() {
    // Default config has llm_polish_enabled=false — the dispatcher must
    // skip the provider call entirely and return the input verbatim.
    let cfg = cfg();
    let d = LlmPolishDispatcher::new(cfg);
    let out = d.polish_active("hola").await.unwrap();
    assert_eq!(out, "hola");
}

#[tokio::test]
async fn dispatcher_returns_not_configured_when_enabled_but_no_provider() {
    let cfg = cfg();
    {
        let mut c = cfg.write().await;
        c.llm_polish_enabled = true;
    }
    let d = LlmPolishDispatcher::new(cfg);
    let err = d.polish_active("hola").await.unwrap_err();
    assert!(matches!(
        err,
        LlmPolishError::NotConfigured(LlmProvider::Groq)
    ));
}

#[tokio::test]
async fn dispatcher_uses_active_provider() {
    // Stand up a mock Groq server so the dispatch path actually round-trips
    // the polish call rather than hitting the real api.groq.com.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{
                "message": { "role": "assistant", "content": "Hola." }
            }]
        })))
        .mount(&server)
        .await;

    let cfg = cfg();
    {
        let mut c = cfg.write().await;
        c.llm_polish_enabled = true;
    }
    let d = LlmPolishDispatcher::new(cfg);
    d.set_groq(Some(Arc::new(GroqLlmPolisher::new_for_test(
        "sk-test".to_string(),
        server.uri(),
    ))))
    .await;
    let out = d.polish_active("hola").await.unwrap();
    assert_eq!(out, "Hola.");
}

#[tokio::test]
async fn dispatcher_rejects_oversized_input() {
    let cfg = cfg();
    {
        let mut c = cfg.write().await;
        c.llm_polish_enabled = true;
        c.llm_polish_max_input_chars = 10;
    }
    let d = LlmPolishDispatcher::new(cfg);
    // Doesn't matter that the provider would 404 — the dispatcher should
    // reject before even calling it.
    d.set_groq(Some(Arc::new(GroqLlmPolisher::new_for_test(
        "sk-test".to_string(),
        "http://127.0.0.1:1".to_string(),
    ))))
    .await;
    let err = d
        .polish_active("this text is way longer than 10 chars")
        .await
        .unwrap_err();
    assert!(matches!(err, LlmPolishError::BadRequest(_)));
}
