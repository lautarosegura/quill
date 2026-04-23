//! Deterministic minimal cleanup applied to every transcription.
//! Rules: trim, capitalize first letter, ensure trailing period for multi-word text.

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

#[cfg(test)]
mod tests;
