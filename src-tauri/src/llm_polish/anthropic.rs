//! Anthropic cloud LLM polisher. Uses the `/v1/messages` endpoint —
//! distinct from the OpenAI/Groq chat shape in four ways:
//!   1. Endpoint path: `/v1/messages` (not `/chat/completions`).
//!   2. The system prompt is a top-level `system` field, NOT a message
//!      with `role: "system"` inside the `messages` array.
//!   3. Auth uses `x-api-key` header (not `Authorization: Bearer`).
//!   4. An `anthropic-version` header is required.
//!   5. The response shape: `content` is an array of blocks (each with
//!      `type` + `text`), and usage uses `input_tokens`/`output_tokens`
//!      (not `prompt_tokens`/`completion_tokens`).
//!   6. `max_tokens` is required in the request body.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::Deserialize;

use super::{LlmPolishError, LlmPolisher, PolishRequest, PolishResult};
use crate::types::LlmProvider;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const REQUEST_TIMEOUT_SECS: u64 = 30;
/// Cap output at 2× the input chars worth of tokens — polish should
/// shorten or stay the same length, not expand.
const MAX_OUTPUT_TOKENS: u32 = 4096;

pub struct AnthropicLlmPolisher {
    pub api_key: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

impl AnthropicLlmPolisher {
    pub fn new(api_key: String) -> Result<Self, LlmPolishError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|e| LlmPolishError::Other(format!("http client init: {e}")))?;
        Ok(Self {
            api_key,
            base_url: DEFAULT_BASE_URL.to_string(),
            client,
        })
    }

    #[cfg(test)]
    pub fn new_for_test(api_key: String, base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .unwrap();
        Self {
            api_key,
            base_url,
            client,
        }
    }
}

#[async_trait]
impl LlmPolisher for AnthropicLlmPolisher {
    async fn polish(&self, req: PolishRequest<'_>) -> Result<PolishResult, LlmPolishError> {
        let url = format!("{}/messages", self.base_url);
        let body = serde_json::json!({
            "model": req.model,
            "system": req.system_prompt,
            "messages": [
                { "role": "user", "content": req.text },
            ],
            "max_tokens": MAX_OUTPUT_TOKENS,
            "temperature": 0.2,
            "stream": false,
        });

        let started = Instant::now();
        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmPolishError::Timeout {
                        secs: REQUEST_TIMEOUT_SECS,
                    }
                } else if e.is_connect() {
                    LlmPolishError::Network(format!("connection error: {e}"))
                } else {
                    LlmPolishError::Network(e.to_string())
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => LlmPolishError::Unauthorized,
                429 => LlmPolishError::RateLimited,
                400 => LlmPolishError::BadRequest(body_text),
                500..=599 => LlmPolishError::Network(format!("Anthropic {status}: {body_text}")),
                _ => LlmPolishError::Network(format!("HTTP {status}: {body_text}")),
            });
        }

        let payload: MessagesResponse = response
            .json()
            .await
            .map_err(|e| LlmPolishError::Other(format!("response parse: {e}")))?;

        // The `content` field is an array of blocks, each with `type` and
        // `text`. For polish we expect a single text block; we concatenate
        // any text-typed blocks defensively.
        let text = payload
            .content
            .iter()
            .filter(|b| b.block_type == "text")
            .map(|b| b.text.as_str())
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        Ok(PolishResult {
            text,
            latency_ms: started.elapsed().as_millis() as u64,
            model: req.model.to_string(),
            input_tokens: payload.usage.as_ref().map(|u| u.input_tokens),
            output_tokens: payload.usage.as_ref().map(|u| u.output_tokens),
        })
    }

    fn provider(&self) -> LlmProvider {
        LlmProvider::Anthropic
    }
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    usage: Option<MessagesUsage>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: String,
}

#[derive(Deserialize)]
struct MessagesUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn req<'a>() -> PolishRequest<'a> {
        PolishRequest {
            text: "eh hola este como va",
            system_prompt: "Limpiá la transcripción.",
            model: "claude-haiku-4-5-20251001",
        }
    }

    #[tokio::test]
    async fn happy_path_returns_polished_text() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/messages"))
            .and(header("x-api-key", "sk-ant-test"))
            .and(header("anthropic-version", ANTHROPIC_VERSION))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "msg_01",
                "type": "message",
                "role": "assistant",
                "model": "claude-haiku-4-5-20251001",
                "content": [
                    { "type": "text", "text": "Hola, ¿cómo va?" }
                ],
                "stop_reason": "end_turn",
                "usage": { "input_tokens": 12, "output_tokens": 6 }
            })))
            .mount(&server)
            .await;

        let p = AnthropicLlmPolisher::new_for_test("sk-ant-test".into(), server.uri());
        let r = p.polish(req()).await.unwrap();
        assert_eq!(r.text, "Hola, ¿cómo va?");
        assert_eq!(r.input_tokens, Some(12));
        assert_eq!(r.output_tokens, Some(6));
    }

    #[tokio::test]
    async fn multiple_text_blocks_are_concatenated() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "content": [
                    { "type": "text", "text": "Hola" },
                    { "type": "text", "text": ", ¿cómo va?" }
                ]
            })))
            .mount(&server)
            .await;
        let p = AnthropicLlmPolisher::new_for_test("sk-test".into(), server.uri());
        let r = p.polish(req()).await.unwrap();
        assert_eq!(r.text, "Hola, ¿cómo va?");
    }

    #[tokio::test]
    async fn unauthorized_401() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;
        let p = AnthropicLlmPolisher::new_for_test("sk-bad".into(), server.uri());
        assert!(matches!(
            p.polish(req()).await.unwrap_err(),
            LlmPolishError::Unauthorized
        ));
    }

    #[tokio::test]
    async fn rate_limit_429() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&server)
            .await;
        let p = AnthropicLlmPolisher::new_for_test("sk-test".into(), server.uri());
        assert!(matches!(
            p.polish(req()).await.unwrap_err(),
            LlmPolishError::RateLimited
        ));
    }

    #[tokio::test]
    async fn server_error_500() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("oops"))
            .mount(&server)
            .await;
        let p = AnthropicLlmPolisher::new_for_test("sk-test".into(), server.uri());
        match p.polish(req()).await.unwrap_err() {
            LlmPolishError::Network(msg) => assert!(msg.contains("500")),
            other => panic!("got {other:?}"),
        }
    }

    #[tokio::test]
    async fn connection_refused_returns_network() {
        let p = AnthropicLlmPolisher::new_for_test("sk-test".into(), "http://127.0.0.1:1".into());
        let err = p.polish(req()).await.unwrap_err();
        assert!(matches!(
            err,
            LlmPolishError::Network(_) | LlmPolishError::Timeout { .. }
        ));
    }
}
