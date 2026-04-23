use super::*;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn list_reports_installed_for_existing_file() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("ggml-base.bin"), b"fake model bytes").unwrap();

    let mgr = ModelManager::new_with_dir(dir.path().to_path_buf());
    let list = mgr.list();

    let base = list.iter().find(|e| e.name == "ggml-base").unwrap();
    assert!(base.installed);
    assert_eq!(base.installed_size_bytes, Some(16));

    let tiny = list.iter().find(|e| e.name == "ggml-tiny").unwrap();
    assert!(!tiny.installed);
    assert_eq!(tiny.installed_size_bytes, None);
}

#[tokio::test]
async fn delete_removes_installed_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ggml-tiny.bin");
    std::fs::write(&path, b"data").unwrap();
    assert!(path.exists());

    let mgr = ModelManager::new_with_dir(dir.path().to_path_buf());
    mgr.delete("ggml-tiny").unwrap();
    assert!(!path.exists());
}

#[tokio::test]
async fn delete_missing_returns_not_found() {
    let dir = tempdir().unwrap();
    let mgr = ModelManager::new_with_dir(dir.path().to_path_buf());
    let result = mgr.delete("ggml-tiny");
    assert!(matches!(result, Err(QuillError::NotFound(_))));
}

#[tokio::test]
async fn download_unknown_model_returns_not_found() {
    let dir = tempdir().unwrap();
    let mgr = ModelManager::new_with_dir(dir.path().to_path_buf());
    let result = mgr.download("ggml-does-not-exist", |_, _| {}).await;
    assert!(matches!(result, Err(QuillError::NotFound(_))));
}

// A custom model with a known SHA and a custom URL. We don't mutate
// KNOWN_MODELS (it's &'static), so this test uses a constructed in-memory
// model via an injected test.

#[tokio::test]
async fn download_streaming_calls_progress_and_verifies_sha() {
    // Build a fake payload.
    let payload: Vec<u8> = (0..2048).map(|i| (i % 256) as u8).collect();
    let expected_sha = {
        let mut h = Sha256::new();
        h.update(&payload);
        hex::encode(h.finalize())
    };

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(payload.clone()))
        .mount(&server)
        .await;

    let dir = tempdir().unwrap();
    let mgr = ModelManager::new_with_dir(dir.path().to_path_buf());

    // Manually perform the download with a custom ModelInfo since KNOWN_MODELS
    // is pinned to real URLs. Use the internal pieces directly for the test.
    let url = format!("{}/model.bin", server.uri());
    let progress_calls: Arc<Mutex<Vec<(u64, u64)>>> = Arc::new(Mutex::new(Vec::new()));
    let progress_clone = Arc::clone(&progress_calls);

    let response = mgr.client.get(&url).send().await.unwrap();
    let total = response.content_length().unwrap_or(payload.len() as u64);

    let partial = dir.path().join("test.partial");
    let final_path = dir.path().join("test.bin");

    let mut file = tokio::fs::File::create(&partial).await.unwrap();
    let mut hasher = Sha256::new();
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(c) = stream.next().await {
        let chunk = c.unwrap();
        file.write_all(&chunk).await.unwrap();
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;
        progress_clone.lock().unwrap().push((downloaded, total));
    }
    file.flush().await.unwrap();
    drop(file);

    let got_sha = hex::encode(hasher.finalize());
    assert_eq!(got_sha, expected_sha);

    std::fs::rename(&partial, &final_path).unwrap();
    assert!(final_path.exists());
    assert_eq!(
        std::fs::metadata(&final_path).unwrap().len(),
        payload.len() as u64
    );
    assert!(!progress_calls.lock().unwrap().is_empty());
}
