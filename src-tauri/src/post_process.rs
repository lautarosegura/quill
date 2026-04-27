//! Deterministic minimal cleanup applied to every transcription.
//! Rules: trim, capitalize first letter, ensure trailing period for multi-word text.
//!
//! [`substitute`] is the second-pass user-defined correction step — it
//! runs AFTER [`clean`] in the orchestrator and catches errors that
//! Whisper's prompt-biasing (via `Config.vocabulary`) couldn't fix.

use crate::types::Substitution;

pub fn clean(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Capitalize the first alphabetic character, leaving leading punctuation
    // (¿, ¡, «, quotes) untouched.
    let mut out = String::with_capacity(trimmed.len() + 1);
    let mut capitalized = false;
    for c in trimmed.chars() {
        if !capitalized && c.is_alphabetic() {
            for up in c.to_uppercase() {
                out.push(up);
            }
            capitalized = true;
        } else {
            out.push(c);
        }
    }

    // Append trailing period for multi-word text if missing.
    let last = out.chars().last();
    let has_terminal = matches!(last, Some('.') | Some('?') | Some('!') | Some('…'));
    let is_single_word = !out.chars().any(|c| c.is_whitespace());
    if !has_terminal && !is_single_word {
        out.push('.');
    }

    out
}

/// Apply user-configured exact-match replacements to a transcription.
/// Each rule's `from` pattern gets a leading `\b` only if it STARTS with
/// a word character, and a trailing `\b` only if it ENDS with a word
/// character. That handles patterns like `"C++"` (where the trailing
/// `+` is non-word, so `\b` can never fire after it) and `".net"`
/// without breaking the basic case `"Mokia"` (still gets both
/// boundaries, doesn't touch "Mokian"). Special regex chars in `from`
/// are escaped — users type literal text, not regex.
///
/// Compilation failures (impossible after `regex::escape`, but
/// belt-and-suspenders) skip the rule with a log line. Empty `from`
/// rules are also skipped — they'd match every position otherwise.
pub fn substitute(input: &str, rules: &[Substitution]) -> String {
    let mut out = input.to_string();
    for rule in rules {
        if rule.from.is_empty() {
            continue;
        }
        let pattern = build_word_boundary_pattern(&rule.from);
        let re = if rule.case_sensitive {
            regex::Regex::new(&pattern)
        } else {
            regex::RegexBuilder::new(&pattern)
                .case_insensitive(true)
                .build()
        };
        match re {
            Ok(re) => {
                out = re.replace_all(&out, rule.to.as_str()).into_owned();
            }
            Err(e) => {
                log::warn!(
                    "post_process::substitute: skipping rule {:?}→{:?}: {e}",
                    rule.from,
                    rule.to
                );
            }
        }
    }
    out
}

fn build_word_boundary_pattern(from: &str) -> String {
    let starts_word = from.chars().next().is_some_and(is_word_char);
    let ends_word = from.chars().last().is_some_and(is_word_char);
    let left = if starts_word { r"\b" } else { "" };
    let right = if ends_word { r"\b" } else { "" };
    format!("{}{}{}", left, regex::escape(from), right)
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests;
