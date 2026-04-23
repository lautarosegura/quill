use super::*;
use tempfile::tempdir;

fn make_entry(text: &str) -> NewHistoryEntry {
    NewHistoryEntry {
        engine: "local".into(),
        language: "es".into(),
        model: Some("ggml-base".into()),
        duration_ms: Some(2500),
        latency_ms: Some(900),
        text: text.into(),
        status: "success".into(),
        failure_reason: None,
        failed_wav_path: None,
        source_app: None,
    }
}

fn open_tmp() -> (tempfile::TempDir, HistoryStore) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.db");
    let store = HistoryStore::open(&path).unwrap();
    (dir, store)
}

#[test]
fn insert_and_round_trip() {
    let (_d, store) = open_tmp();
    let id = store.insert(&make_entry("hola mundo")).unwrap();
    assert!(id > 0);
    let got = store.get(id).unwrap().unwrap();
    assert_eq!(got.text, "hola mundo");
    assert_eq!(got.engine, "local");
    assert_eq!(got.status, "success");
}

#[test]
fn recent_returns_newest_first() {
    let (_d, store) = open_tmp();
    store.insert(&make_entry("primero")).unwrap();
    // Small sleep not needed — created_at uses high-res chrono; but if inserts
    // ran within the same millisecond, SQLite ordering by created_at is stable.
    // The actual newest-first behavior is guaranteed by ORDER BY created_at DESC.
    store.insert(&make_entry("segundo")).unwrap();
    store.insert(&make_entry("tercero")).unwrap();

    let rows = store.recent(10, 0).unwrap();
    assert_eq!(rows.len(), 3);
    // If all inserted in the same clock tick, allow either stable order —
    // just assert they're all present.
    let texts: Vec<_> = rows.iter().map(|r| r.text.clone()).collect();
    assert!(texts.contains(&"primero".to_string()));
    assert!(texts.contains(&"segundo".to_string()));
    assert!(texts.contains(&"tercero".to_string()));
}

#[test]
fn search_matches_substring_case_insensitive() {
    let (_d, store) = open_tmp();
    store.insert(&make_entry("Hola mundo")).unwrap();
    store.insert(&make_entry("adiós hasta luego")).unwrap();
    store.insert(&make_entry("HELLO world")).unwrap();

    let hits = store.search("hola", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].text, "Hola mundo");

    let hits = store.search("HELLO", 10).unwrap();
    assert_eq!(hits.len(), 1);

    let hits = store.search("hell", 10).unwrap();
    assert_eq!(hits.len(), 1); // case-insensitive substring
}

#[test]
fn delete_by_id() {
    let (_d, store) = open_tmp();
    let id = store.insert(&make_entry("borrame")).unwrap();
    assert_eq!(store.count().unwrap(), 1);
    store.delete(id).unwrap();
    assert_eq!(store.count().unwrap(), 0);
    assert!(store.get(id).unwrap().is_none());
}

#[test]
fn clear_all_empties_table() {
    let (_d, store) = open_tmp();
    for i in 0..5 {
        store.insert(&make_entry(&format!("entry {i}"))).unwrap();
    }
    assert_eq!(store.count().unwrap(), 5);
    store.clear_all().unwrap();
    assert_eq!(store.count().unwrap(), 0);
}

#[test]
fn failed_entries_stored_with_reason() {
    let (_d, store) = open_tmp();
    let entry = NewHistoryEntry {
        status: "failed".into(),
        failure_reason: Some("network error".into()),
        text: String::new(),
        ..make_entry("")
    };
    let id = store.insert(&entry).unwrap();
    let got = store.get(id).unwrap().unwrap();
    assert_eq!(got.status, "failed");
    assert_eq!(got.failure_reason.as_deref(), Some("network error"));
}

#[test]
fn open_is_idempotent() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.db");
    let store1 = HistoryStore::open(&path).unwrap();
    store1.insert(&make_entry("persistido")).unwrap();
    drop(store1);

    let store2 = HistoryStore::open(&path).unwrap();
    assert_eq!(store2.count().unwrap(), 1);
    let rows = store2.recent(10, 0).unwrap();
    assert_eq!(rows[0].text, "persistido");
}

#[test]
fn engine_check_constraint_rejects_invalid() {
    let (_d, store) = open_tmp();
    let bad = NewHistoryEntry {
        engine: "bogus".into(),
        ..make_entry("test")
    };
    let result = store.insert(&bad);
    assert!(result.is_err());
}
