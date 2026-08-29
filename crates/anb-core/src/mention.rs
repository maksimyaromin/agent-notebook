//! The Mention scan: id-shaped tokens in body text, found on demand.
//!
//! A scan, not a parse: the one piece of markdown it reads is the backtick
//! run — an id inside a code span or a fenced block is a quotation, not a
//! mention — nothing is stored, no body byte is ever written. A hit is a
//! hint for Debt and the view surfaces, never a validity judgment.

use crate::grammar;

const TYPE_PREFIXES: [&str; 4] = ["task.", "decision.", "note.", "question."];

/// The ids cited in `text`, first appearance first, each once.
///
/// A candidate must stand at an ASCII word boundary on both sides:
/// `subtask.x` and `task.fooBar` cite nothing, `(task.x)` and a
/// sentence-final `task.x.` cite `task.x`. Every hit satisfies the id
/// grammar. An id inside a code span is a quotation and never a hit.
pub(crate) fn mentions(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut found: Vec<&str> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'`' {
            at = if opens_fence(bytes, at) {
                after_fence(bytes, at)
            } else {
                after_span_run(bytes, at)
            };
            continue;
        }
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

/// A fence opens at a line-leading run of three or more backticks.
fn opens_fence(bytes: &[u8], at: usize) -> bool {
    (at == 0 || bytes[at - 1] == b'\n') && backtick_run_len(bytes, at) >= 3
}

/// Where the scan resumes after the fence opening at `at`: past the run of
/// the line-leading closer, which markdown only asks to be at least as
/// long as the opener — or the end of the text, since an unclosed fence
/// swallows everything after it.
fn after_fence(bytes: &[u8], at: usize) -> usize {
    let opener = backtick_run_len(bytes, at);
    let mut line_start = next_line_start(bytes, at + opener);
    while line_start < bytes.len() {
        if bytes[line_start] == b'`' {
            let closer = backtick_run_len(bytes, line_start);
            if closer >= opener {
                return line_start + closer;
            }
        }
        line_start = next_line_start(bytes, line_start);
    }
    bytes.len()
}

/// Where the scan resumes after the span run at `at`: past the code span
/// the run opens — closed by the next run of exactly its length on the
/// same line, since the bodies here hold one log entry per line and a pair
/// across entries would let one stray backtick unquote every later one —
/// or past the run alone when none closes it, so what follows an unpaired
/// run still scans as prose.
fn after_span_run(bytes: &[u8], at: usize) -> usize {
    let opener = backtick_run_len(bytes, at);
    let mut seek = at + opener;
    while seek < bytes.len() && bytes[seek] != b'\n' {
        if bytes[seek] != b'`' {
            seek += 1;
            continue;
        }
        let closer = backtick_run_len(bytes, seek);
        if closer == opener {
            return seek + closer;
        }
        seek += closer;
    }
    at + opener
}

fn next_line_start(bytes: &[u8], from: usize) -> usize {
    bytes[from..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |newline| from + newline + 1)
}

fn backtick_run_len(bytes: &[u8], at: usize) -> usize {
    bytes[at..]
        .iter()
        .position(|byte| *byte != b'`')
        .unwrap_or(bytes.len() - at)
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
