//! Text bounds, log signatures, shell arguments and record string quoting.
//! Reply document encoding belongs to the host.

use std::fmt::Write as _;

/// The default row bound for hosts; total and omitted counts are separate.
pub const ROW_BOUND: usize = 20;

/// How many characters of a record's own text a derived reply carries.
///
/// [`ROW_BOUND`] answers how many records a reply names; this answers how
/// much of one. Without it a reply is bounded in one dimension only, and a
/// single title or log line written long enough carries the whole reply
/// past its budget.
pub const TEXT_BOUND: usize = 200;

/// A record's text cut to [`TEXT_BOUND`] characters — its two ends, with
/// the elision that counts what fell out between them — or the text itself
/// when it already fits.
///
/// Both ends are kept because such a value is read from both: the opening
/// words say which record or which field, and a finding's reason stands at
/// the end. Whoever needs the rest reads the record.
#[must_use]
pub fn bounded_text(value: String) -> String {
    let length = value.chars().count();
    if length <= TEXT_BOUND {
        return value;
    }
    // The elision counts what it dropped, and no drop is wider than the
    // text it came from, so the ends are cut to leave room for the widest
    // that count can be — whatever it turns out to be, the answer fits.
    let half = TEXT_BOUND.saturating_sub(elision(length).chars().count()) / 2;
    let head = char_boundary(&value, half);
    let tail = char_boundary(&value, length - half);
    format!(
        "{}{}{}",
        &value[..head],
        elision(length - 2 * half),
        &value[tail..]
    )
}

fn elision(dropped: usize) -> String {
    format!(" \u{2026} {dropped} more characters \u{2026} ")
}

/// The byte index the `chars`th character starts at, or the end of the
/// value when it holds no such character.
fn char_boundary(value: &str, chars: usize) -> usize {
    value
        .char_indices()
        .nth(chars)
        .map_or(value.len(), |(index, _)| index)
}

/// An edge walk named inline in a message — a dependency cycle, a lineage.
/// A message has one surface and so one bound.
#[must_use]
pub(crate) fn id_chain(ids: &[String]) -> String {
    bounded_join(ids, " \u{2192} ", ROW_BOUND)
}

fn bounded_join(ids: &[String], separator: &str, bound: usize) -> String {
    let shown = ids.len().min(bound);
    let mut out = ids[..shown].join(separator);
    if ids.len() > shown {
        let _ = write!(out, "{separator}\u{2026} {} more", ids.len() - shown);
    }
    out
}

/// A body cut to its ends: the first and last [`ROW_BOUND`] lines as they
/// stand, and how many fell out between them. `None` when the body is
/// short enough that an elision would save nothing.
///
/// A record is read from both ends — its terms are written at the top and
/// its log grows at the bottom — so the middle is what a long record can
/// spare. The two ends are slices of the body, so each keeps its own line
/// terminator and a CRLF file still reads as one.
#[must_use]
pub fn body_ends(body: &str) -> Option<(&str, usize, &str)> {
    let lines: Vec<&str> = body.split_inclusive('\n').collect();
    if lines.len() <= 2 * ROW_BOUND + 1 {
        return None;
    }
    let head: usize = lines[..ROW_BOUND].iter().map(|line| line.len()).sum();
    let tail: usize = lines[lines.len() - ROW_BOUND..]
        .iter()
        .map(|line| line.len())
        .sum();
    Some((
        &body[..head],
        lines.len() - 2 * ROW_BOUND,
        &body[body.len() - tail..],
    ))
}

/// The one spelling of who stands behind a text: `by`, plus `/via` when an
/// agent hand wrote it; no identity at all prints as `-`, so a reader
/// judging two sides never meets a blank one. A log entry is signed with
/// it, and a cited record is attributed with it.
#[must_use]
pub fn author(by: Option<&str>, via: Option<&str>) -> String {
    match (by, via) {
        (Some(by), Some(via)) => format!("{by}/{via}"),
        (Some(by), None) => by.to_owned(),
        (None, Some(via)) => format!("-/{via}"),
        (None, None) => "-".to_owned(),
    }
}

/// A value as one shell word, so a command a reply names stays typeable:
/// single-quoted unless every character is one a shell reads as itself.
#[must_use]
pub fn shell_word(value: &str) -> String {
    let bare = value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if bare && !value.is_empty() {
        value.to_owned()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

/// Quote inline diagnostic values containing commas, quotes or terminal controls.
#[must_use]
pub fn quoted_if_delimited(value: &str) -> String {
    if value.contains([',', '"']) || value.contains(acted_on_by_a_terminal) {
        json_quoted(value)
    } else {
        value.to_owned()
    }
}

/// A character a terminal treats as an instruction rather than as text:
/// the control characters, and the bidirectional and line/paragraph
/// separators, which `char::is_control` does not count.
fn acted_on_by_a_terminal(character: char) -> bool {
    character.is_control()
        || matches!(character,
            '\u{200e}' | '\u{200f}' | '\u{2028}' | '\u{2029}'
            | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// Quotes the value, escaping the backslash, the quote, and every
/// character a terminal would act on.
#[must_use]
pub(crate) fn json_quoted(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if acted_on_by_a_terminal(other) || matches!(other, '\u{fffe}' | '\u{ffff}') => {
                let _ = write!(out, "\\u{:04x}", other as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(count: usize) -> Vec<String> {
        (0..count).map(|n| format!("task.t{n:02}")).collect()
    }

    #[test]
    fn a_chain_walks_its_ids_with_arrows() {
        assert_eq!(id_chain(&ids(2)), "task.t00 \u{2192} task.t01");
    }

    #[test]
    fn text_within_the_bound_stands_as_it_was_written() {
        let title = "Bounded replies keep a dashboard readable";
        assert_eq!(bounded_text(title.to_owned()), title);
    }

    /// The two ends are what a reader needs: which record the text is
    /// about, and the reason a finding ends with. Nothing is invented and
    /// nothing goes missing — the count between the ends is exactly what
    /// they leave out — and the whole answer fits the bound, elision
    /// included, so a caller can spend against it.
    #[test]
    fn text_past_the_bound_keeps_both_ends_and_counts_the_rest() {
        let written = format!("state: `{}` is not a state", "x".repeat(500));
        let bounded = bounded_text(written.clone());

        assert!(bounded.chars().count() <= TEXT_BOUND, "{bounded}");
        let (head, rest) = bounded.split_once(" \u{2026} ").unwrap();
        let (dropped, tail) = rest.split_once(" more characters \u{2026} ").unwrap();
        assert!(
            written.starts_with(head) && written.ends_with(tail),
            "{bounded}"
        );
        assert_eq!(
            head.chars().count() + dropped.parse::<usize>().unwrap() + tail.chars().count(),
            written.chars().count()
        );
    }

    /// The bound counts characters, and a cut lands between two of them:
    /// slicing a multi-byte character in half would abort the process.
    #[test]
    fn a_multi_byte_text_is_cut_by_characters_not_bytes() {
        let bounded = bounded_text("\u{4e16}".repeat(300));
        assert!(bounded.chars().count() <= TEXT_BOUND, "{bounded}");
        assert!(bounded.starts_with('\u{4e16}') && bounded.ends_with('\u{4e16}'));
    }
}
