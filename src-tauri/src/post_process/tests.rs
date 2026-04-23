use super::clean;

#[test]
fn trims_whitespace_and_adds_period() {
    assert_eq!(clean("  hola mundo  "), "Hola mundo.");
}

#[test]
fn capitalizes_first_letter() {
    assert_eq!(clean("hola"), "Hola");
}

#[test]
fn preserves_existing_final_punctuation_period() {
    assert_eq!(clean("ya está."), "Ya está.");
}

#[test]
fn preserves_existing_final_punctuation_question() {
    // Capitalizes first alphabetic char (skipping leading ¿) and keeps ?.
    assert_eq!(clean("¿qué tal?"), "¿Qué tal?");
}

#[test]
fn preserves_existing_final_punctuation_exclamation() {
    assert_eq!(clean("increíble!"), "Increíble!");
}

#[test]
fn preserves_ellipsis() {
    assert_eq!(clean("bueno…"), "Bueno…");
}

#[test]
fn single_word_no_period_added() {
    assert_eq!(clean("hola"), "Hola");
}

#[test]
fn empty_string_returns_empty() {
    assert_eq!(clean(""), "");
}

#[test]
fn whitespace_only_returns_empty() {
    assert_eq!(clean("   \t\n  "), "");
}

#[test]
fn unicode_first_letter_capitalizes() {
    assert_eq!(clean("ñandú corre rápido"), "Ñandú corre rápido.");
}

#[test]
fn leading_punctuation_preserved() {
    // Whisper sometimes prefixes with ¿ — first *letter* should capitalize.
    assert_eq!(clean("¿por qué es así"), "¿Por qué es así.");
}

#[test]
fn already_capitalized_is_kept() {
    assert_eq!(clean("Hola mundo"), "Hola mundo.");
}

#[test]
fn does_not_double_period() {
    assert_eq!(clean("final."), "Final.");
}
