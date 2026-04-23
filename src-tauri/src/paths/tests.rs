use super::*;

#[test]
fn quill_home_is_under_user_home() {
    let home = quill_home();
    let user_home = dirs::home_dir().unwrap();
    assert!(home.starts_with(user_home));
    assert!(home.ends_with(".quill"));
}

#[test]
fn config_file_path_is_correct() {
    let p = config_file();
    assert!(p.ends_with("config.json"));
    assert!(p.starts_with(quill_home()));
}

#[test]
fn subpaths_are_under_quill_home() {
    let home = quill_home();
    for p in [
        history_db(),
        vocabulary_file(),
        models_dir(),
        failed_dir(),
        logs_dir(),
    ] {
        assert!(p.starts_with(&home), "{p:?} should be under {home:?}");
    }
}
