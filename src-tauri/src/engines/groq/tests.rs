use super::*;
use crate::types::Language;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fake_wav() -> Vec<u8> {
    // Minimal 44-byte RIFF/WAVE header so Groq wouldn't reject on structure.
    // Actual content doesn't matter for the mock server.
    let mut v = b"RIFF\x24\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00\x40\x1f\x00\x00\x80\x3e\x00\x00\x02\x00\x10\x00data\x00\x00\x00\x00".to_vec();
    v.extend_from_slice(&[0u8; 100]);
    v
}

fn req<'a>(wav: &'a [u8], language: Language, prompt: Option<&'a str>) -> TranscriptionRequest<'a> {
    TranscriptionRequest {
        audio_wav: wav,
        language,
        prompt,
    }
}

#[tokio::test]
async fn happy_path_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .and(header("authorization", "Bearer sk-test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "text": "hola mundo"
        })))
        .mount(&server)
        .await;

    let engine = GroqEngine::new_for_test("sk-test".into(), server.uri());
    let wav = fake_wav();
    let result = engine
        .transcribe(req(&wav, Language::Es, None))
        .await
        .unwrap();
    assert_eq!(result.text, "hola mundo");
    assert_eq!(result.model, DEFAULT_MODEL);
}

#[tokio::test]
async fn unauthorized_returns_unauthorized() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid key"))
        .mount(&server)
        .await;

    let engine = GroqEngine::new_for_test("sk-bad".into(), server.uri());
    let wav = fake_wav();
    let err = engine
        .transcribe(req(&wav, Language::Es, None))
        .await
        .unwrap_err();
    assert!(matches!(err, TranscriptionError::Unauthorized));
}

#[tokio::test]
async fn rate_limit_429_returns_rate_limited() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;

    let engine = GroqEngine::new_for_test("sk-test".into(), server.uri());
    let wav = fake_wav();
    let err = engine
        .transcribe(req(&wav, Language::Es, None))
        .await
        .unwrap_err();
    assert!(matches!(err, TranscriptionError::RateLimited));
}

#[tokio::test]
async fn payment_required_402_returns_rate_limited() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(402))
        .mount(&server)
        .await;

    let engine = GroqEngine::new_for_test("sk-test".into(), server.uri());
    let wav = fake_wav();
    let err = engine
        .transcribe(req(&wav, Language::Es, None))
        .await
        .unwrap_err();
    assert!(matches!(err, TranscriptionError::RateLimited));
}

#[tokio::test]
async fn server_error_5xx_returns_network() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503).set_body_string("upstream down"))
        .mount(&server)
        .await;

    let engine = GroqEngine::new_for_test("sk-test".into(), server.uri());
    let wav = fake_wav();
    let err = engine
        .transcribe(req(&wav, Language::Es, None))
        .await
        .unwrap_err();
    match err {
        TranscriptionError::Network(msg) => assert!(msg.contains("503")),
        other => panic!("expected Network error, got {other:?}"),
    }
}

#[tokio::test]
async fn bad_request_400_returns_audio_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad audio format"))
        .mount(&server)
        .await;

    let engine = GroqEngine::new_for_test("sk-test".into(), server.uri());
    let wav = fake_wav();
    let err = engine
        .transcribe(req(&wav, Language::Es, None))
        .await
        .unwrap_err();
    assert!(matches!(err, TranscriptionError::AudioRejected(_)));
}

#[tokio::test]
async fn slow_response_returns_timeout() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "text": "too late" }))
                .set_delay(Duration::from_secs(5)),
        )
        .mount(&server)
        .await;

    // new_for_test uses 1s client timeout — 5s delay will trip it.
    let engine = GroqEngine::new_for_test("sk-test".into(), server.uri());
    let wav = fake_wav();
    let err = engine
        .transcribe(req(&wav, Language::Es, None))
        .await
        .unwrap_err();
    match err {
        TranscriptionError::Timeout { .. } => {}
        TranscriptionError::Network(msg) if msg.contains("timed out") => {}
        other => panic!("expected timeout-style error, got {other:?}"),
    }
}

#[tokio::test]
async fn connection_refused_returns_network() {
    // Point at an unused localhost port. Depending on platform, reqwest may
    // surface this as Network (connect error) or Timeout — both are
    // acceptable; the point is we don't panic and we don't call it a success.
    let engine = GroqEngine::new_for_test("sk-test".into(), "http://127.0.0.1:1".into());
    let wav = fake_wav();
    let err = engine
        .transcribe(req(&wav, Language::Es, None))
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            TranscriptionError::Network(_) | TranscriptionError::Timeout { .. }
        ),
        "expected Network or Timeout, got {err:?}"
    );
}

#[tokio::test]
async fn omits_prompt_when_empty_or_none() {
    let server = MockServer::start().await;
    // Any POST responds 200 — we only care the request builds without the `prompt` field.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "text": "ok" })))
        .mount(&server)
        .await;

    let engine = GroqEngine::new_for_test("sk-test".into(), server.uri());
    let wav = fake_wav();
    let _ok_none = engine
        .transcribe(req(&wav, Language::Es, None))
        .await
        .unwrap();
    let _ok_empty = engine
        .transcribe(req(&wav, Language::Es, Some("")))
        .await
        .unwrap();
}

#[tokio::test]
async fn circuit_breaker_opens_after_repeated_failures() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let mut engine = GroqEngine::new_for_test("sk-test".into(), server.uri());
    engine.breaker = std::sync::Arc::new(crate::circuit_breaker::CircuitBreaker::new(3, 60, 30));
    let wav = fake_wav();

    for _ in 0..3 {
        let err = engine
            .transcribe(req(&wav, Language::Es, None))
            .await
            .unwrap_err();
        assert!(matches!(err, TranscriptionError::Network(_)));
    }

    let err = engine
        .transcribe(req(&wav, Language::Es, None))
        .await
        .unwrap_err();
    match err {
        TranscriptionError::Network(msg) => {
            assert!(
                msg.contains("circuit breaker") || msg.contains("Groq temporalmente"),
                "expected breaker message, got: {msg}"
            );
        }
        other => panic!("expected breaker-open Network error, got {other:?}"),
    }
}
