//! OpenAI cloud LLM polisher. Uses the standard `/v1/chat/completions`
//! endpoint. Schema is identical to Groq's (Groq is OpenAI-compatible),
//! the only differences are the base URL and the model identifiers.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::Deserialize;

use super::{LlmPolishError, LlmPolisher, PolishRequest, PolishResult};
use crate::types::LlmProvider;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct OpenaiLlmPolisher {
    pub api_key: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

impl OpenaiLlmPolisher {
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
impl LlmPolisher for OpenaiLlmPolisher {
    async fn polish(&self, req: PolishRequest<'_>) -> Result<PolishResult, LlmPolishError> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = serde_json::json!({
            "model": req.model,
            "messages": [
                { "role": "system", "content": req.system_prompt },
                { "role": "user",   "content": req.text },
            ],
            "temperature": 0.2,
            "stream": false,
        });

        let started = Instant::now();
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
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
                500..=599 => LlmPolishError::Network(format!("OpenAI {status}: {body_text}")),
                _ => LlmPolishError::Network(format!("HTTP {status}: {body_text}")),
            });
        }

        let payload: ChatResponse = response
            .json()
            .await
            .map_err(|e| LlmPolishError::Other(format!("response parse: {e}")))?;

        let text = payload
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default()
            .trim()
            .to_string();

        Ok(PolishResult {
            text,
            latency_ms: started.elapsed().as_millis() as u64,
            model: req.model.to_string(),
            input_tokens: payload.usage.as_ref().map(|u| u.prompt_tokens),
            output_tokens: payload.usage.as_ref().map(|u| u.completion_tokens),
        })
    }

    fn provider(&self) -> LlmProvider {
        LlmProvider::Openai
    }
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: Option<ChatUsage>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

#[derive(Deserialize)]
struct ChatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn req<'a>() -> PolishRequest<'a> {
        PolishRequest {
            text: "uh ok so this is a test",
            system_prompt: "Clean up the transcription.",
            model: "gpt-4o-mini",
        }
    }

    #[tokio::test]
    async fn happy_path_returns_polished_text() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer sk-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": { "role": "assistant", "content": "OK, so this is a test." }
                }],
                "usage": { "prompt_tokens": 9, "completion_tokens": 7 }
            })))
            .mount(&server)
            .await;

        let p = OpenaiLlmPolisher::new_for_test("sk-test".into(), server.uri());
        let r = p.polish(req()).await.unwrap();
        assert_eq!(r.text, "OK, so this is a test.");
        assert_eq!(r.input_tokens, Some(9));
        assert_eq!(r.output_tokens, Some(7));
    }

    #[tokio::test]
    async fn unauthorized_401() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;
        let p = OpenaiLlmPolisher::new_for_test("sk-bad".into(), server.uri());
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
        let p = OpenaiLlmPolisher::new_for_test("sk-test".into(), server.uri());
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
        let p = OpenaiLlmPolisher::new_for_test("sk-test".into(), server.uri());
        match p.polish(req()).await.unwrap_err() {
            LlmPolishError::Network(msg) => assert!(msg.contains("500")),
            other => panic!("got {other:?}"),
        }
    }

    #[tokio::test]
    async fn connection_refused_returns_network() {
        let p = OpenaiLlmPolisher::new_for_test("sk-test".into(), "http://127.0.0.1:1".into());
        let err = p.polish(req()).await.unwrap_err();
        assert!(matches!(
            err,
            LlmPolishError::Network(_) | LlmPolishError::Timeout { .. }
        ));
    }
}
