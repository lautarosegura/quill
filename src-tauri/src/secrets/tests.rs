use super::*;

#[test]
#[ignore = "touches OS keychain — run manually with `cargo test secrets -- --ignored`"]
fn set_and_get_round_trips() {
    let test_key = "test-key-12345";
    SecretStore::set_groq_key(test_key).unwrap();
    let retrieved = SecretStore::get_groq_key().unwrap();
    assert_eq!(retrieved.as_deref(), Some(test_key));

    SecretStore::delete_groq_key().unwrap();
    assert_eq!(SecretStore::get_groq_key().unwrap(), None);
}

#[test]
#[ignore = "touches OS keychain"]
fn delete_missing_is_ok() {
    let _ = SecretStore::delete_groq_key();
    // Second delete should not error (NoEntry is treated as success).
    SecretStore::delete_groq_key().unwrap();
}
