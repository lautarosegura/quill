use super::{clean, substitute};
use crate::types::Substitution;

fn sub(from: &str, to: &str, case_sensitive: bool) -> Substitution {
    Substitution {
        from: from.to_string(),
        to: to.to_string(),
        case_sensitive,
    }
}

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

// ----- substitute() tests -----

#[test]
fn substitute_basic_word_replace() {
    let rules = vec![sub("Mokia", "Nokia", false)];
    assert_eq!(substitute("Compré un Mokia.", &rules), "Compré un Nokia.");
}

#[test]
fn substitute_respects_word_boundary() {
    // "Mokia" rule should NOT replace inside "Mokian".
    let rules = vec![sub("Mokia", "Nokia", false)];
    assert_eq!(
        substitute("Mokian es otra palabra.", &rules),
        "Mokian es otra palabra."
    );
}

#[test]
fn substitute_case_insensitive_by_default() {
    let rules = vec![sub("mokia", "Nokia", false)];
    // Both casings replaced with "Nokia" (the rule's `to` is used verbatim,
    // not case-preserved — this is intentional for fixing brand names).
    assert_eq!(substitute("Mokia y mokia.", &rules), "Nokia y Nokia.");
}

#[test]
fn substitute_case_sensitive_when_flagged() {
    let rules = vec![sub("Quill", "Quill™", true)];
    // Lowercase shouldn't match.
    assert_eq!(substitute("quill y Quill", &rules), "quill y Quill™");
}

#[test]
fn substitute_escapes_regex_specials() {
    // User typing "C++" must match "C++" literally, not "C" followed by "+".
    let rules = vec![sub("C++", "Cpp", false)];
    assert_eq!(
        substitute("Programa en C++ rápido.", &rules),
        "Programa en Cpp rápido."
    );
}

#[test]
fn substitute_multiple_rules_apply_in_order() {
    let rules = vec![sub("foo", "bar", false), sub("bar", "baz", false)];
    // First rule: "foo" → "bar". Second rule: "bar" → "baz" (matches both
    // the original "bar" and the new "bar" from rule 1). Order matters.
    assert_eq!(substitute("foo y bar", &rules), "baz y baz");
}

#[test]
fn substitute_skips_empty_from_rule() {
    // An empty pattern would match every position. Skip silently.
    let rules = vec![sub("", "X", false)];
    assert_eq!(substitute("hola mundo", &rules), "hola mundo");
}

#[test]
fn substitute_no_rules_returns_input_unchanged() {
    assert_eq!(substitute("hola mundo", &[]), "hola mundo");
}

#[test]
fn substitute_handles_punctuation_around_match() {
    let rules = vec![sub("mokia", "Nokia", false)];
    // Word boundaries treat "." and "," and "?" as boundaries, so these match.
    assert_eq!(
        substitute("¿Mokia? ¡mokia, claro!", &rules),
        "¿Nokia? ¡Nokia, claro!"
    );
}
