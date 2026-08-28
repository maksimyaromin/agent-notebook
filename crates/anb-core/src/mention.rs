//! The Mention scan: id-shaped tokens in body text, found on demand.
//!
//! A scan, not a parse: no markdown structure is
//! interpreted, nothing is stored, no body byte is ever written. A hit is a
//! hint for Debt and the view surfaces, never a validity judgment.

use crate::grammar;

const TYPE_PREFIXES: [&str; 4] = ["task.", "decision.", "note.", "question."];

/// The ids cited in `text`, first appearance first, each once.
///
/// A candidate must stand at an ASCII word boundary on both sides:
/// `subtask.x` and `task.fooBar` cite nothing, `(task.x)` and a
/// sentence-final `task.x.` cite `task.x`. Every hit satisfies the id
/// grammar.
pub(crate) fn mentions(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut found: Vec<&str> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if !starts_word(bytes, at) {
            at += 1;
            continue;
        }
        match id_at(text, at) {
            Some(id) => {
                if !found.contains(&id) {
                    found.push(id);
                }
                at += id.len();
            }
            // Skip the whole word so its tail cannot fake a boundary.
            None => at += word_len(bytes, at),
        }
    }
    found
}

/// A word starts at an id-run byte not preceded by one; `.`, `-`, and `_`
/// block too, so a dotted or hyphenated compound is one word, not two.
fn starts_word(bytes: &[u8], at: usize) -> bool {
    let in_word = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_');
    in_word(bytes[at]) && (at == 0 || !in_word(bytes[at - 1]))
}

/// The run length from `at`, which sits on a word byte.
fn word_len(bytes: &[u8], at: usize) -> usize {
    bytes[at..]
        .iter()
        .position(|byte| !byte.is_ascii_alphanumeric() && !matches!(byte, b'.' | b'-' | b'_'))
        .unwrap_or(bytes.len() - at)
}

/// The id starting exactly at `at`, if one does: a type prefix, then the
/// longest `[a-z0-9-]` run trimmed of trailing hyphens — so `task.x.` and
/// `task.x-` cite `task.x` — rejected whole when the run continues into a
/// longer word or fails the id grammar.
fn id_at(text: &str, at: usize) -> Option<&str> {
    let rest = &text[at..];
    let prefix = TYPE_PREFIXES
        .into_iter()
        .find(|prefix| rest.starts_with(prefix))?;
    let run_len = rest[prefix.len()..]
        .bytes()
        .position(|byte| !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && byte != b'-')
        .unwrap_or(rest.len() - prefix.len());
    let after_run = rest[prefix.len() + run_len..].bytes().next();
    if after_run.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        return None;
    }
    let slug = rest[prefix.len()..prefix.len() + run_len].trim_end_matches('-');
    let id = &rest[..prefix.len() + slug.len()];
    grammar::id_error(id).is_none().then_some(id)
}
