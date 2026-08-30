//! The encoding every reply surface shares — the bound on a list, the
//! quoting rule, and the ready rows — so no two surfaces spell the same
//! value differently.

use crate::date;
use crate::reply::ReadyTask;
use std::fmt::Write as _;

/// How many rows a reply carries before it names the rest as a count.
///
/// What a command derives from the notebook is answered at a fixed size,
/// whatever the notebook holds; a listing lifts that with `--all`, and a
/// consequence named in passing has no lift. A record's own bytes are not
/// derived, so `view` shows them as they stand — until a body outgrows a
/// reply on its own, when it too is answered at a fixed size and `--all`
/// lifts it.
///
/// Its other half, for every block a reply heads with `label[n]`: `n` is
/// the whole set, and the rows under it are as many as the reply affords,
/// with the shortfall named by the block's own hint. A header that counted
/// its rows would say what the reader can already see and hide what it
/// cannot.
pub const ROW_BOUND: usize = 20;

/// The ids named inline in a reply, comma-separated: the first `bound` of
/// them, then how many were left out.
#[must_use]
pub fn id_list(ids: &[String], bound: usize) -> String {
    bounded_join(ids, ", ", bound)
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

/// A table value carrying the row delimiter, a quote, or a character a
/// terminal acts on is JSON-quoted; everything else stays bare.
///
/// A record's text is whatever a hand wrote, and a terminal obeys what is
/// in it: an unescaped `\r` or `ESC[2K` rewrites the row above and forges a
/// line about another record, and a bidi override reverses what follows it.
/// Quoted, such a character cannot reach past the cell it sits in.
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
            other if acted_on_by_a_terminal(other) => {
                let _ = write!(out, "\\u{:04x}", other as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// The ready table — header and the first `shown` rows, ages derived from
/// `today_day`. The caller owns its own truncation hint.
#[must_use]
pub fn ready_table(rows: &[ReadyTask], shown: usize, today_day: i64) -> String {
    let mut out = format!("ready[{}]{{id,priority,age,title}}:\n", rows.len());
    for row in &rows[..shown] {
        let priority = row
            .priority
            .map_or_else(|| "-".to_owned(), |priority| priority.to_string());
        let _ = writeln!(
            out,
            "  {},{priority},{}d,{}",
            row.id,
            age_days(&row.created, today_day),
            quoted_if_delimited(&row.title)
        );
    }
    out
}

/// Whole days from `created` to `today_day`, floored at zero; an unreadable
/// date counts as today.
fn age_days(created: &str, today_day: i64) -> i64 {
    date::day_number(created).map_or(0, |day| (today_day - day).max(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, priority: Option<u8>, created: &str, title: &str) -> ReadyTask {
        ReadyTask {
            id: id.to_owned(),
            priority,
            created: created.to_owned(),
            title: title.to_owned(),
        }
    }

    fn ids(count: usize) -> Vec<String> {
        (0..count).map(|n| format!("task.t{n:02}")).collect()
    }

    #[test]
    fn a_list_filling_the_bound_names_every_id_and_marks_nothing() {
        assert_eq!(id_list(&ids(3), 3), "task.t00, task.t01, task.t02");
    }

    #[test]
    fn a_list_past_the_bound_names_the_first_and_counts_the_rest() {
        assert_eq!(
            id_list(&ids(6), 3),
            "task.t00, task.t01, task.t02, … 3 more"
        );
    }

    #[test]
    fn a_chain_walks_its_ids_with_arrows() {
        assert_eq!(id_chain(&ids(2)), "task.t00 → task.t01");
    }

    #[test]
    fn a_row_derives_its_age_and_quotes_a_delimited_title() {
        let rows = [row(
            "task.demo",
            None,
            "2026-08-24",
            "Degrades sections, keeps Budget",
        )];
        let today_day = date::day_number("2026-08-28").unwrap();
        assert_eq!(
            ready_table(&rows, 1, today_day),
            "ready[1]{id,priority,age,title}:\n  task.demo,-,4d,\"Degrades sections, keeps Budget\"\n"
        );
    }

    #[test]
    fn an_unreadable_created_date_counts_as_today() {
        let rows = [row("task.demo", Some(1), "", "A demo record")];
        assert_eq!(
            ready_table(&rows, 1, 20_000),
            "ready[1]{id,priority,age,title}:\n  task.demo,1,0d,A demo record\n"
        );
    }

    /// A clock behind the notebook's own dates would otherwise age a record
    /// backwards; nothing is younger than new.
    #[test]
    fn a_record_created_after_today_is_no_age_at_all() {
        let rows = [row("task.demo", None, "2026-08-30", "A demo record")];
        let today_day = date::day_number("2026-08-28").unwrap();
        assert_eq!(
            ready_table(&rows, 1, today_day),
            "ready[1]{id,priority,age,title}:\n  task.demo,-,0d,A demo record\n"
        );
    }

    /// A terminal obeys the escape sequences in a title, so a row must not
    /// carry one: `\r` alone reprints the line as another record's row.
    #[test]
    fn a_control_character_in_a_title_is_escaped_into_its_own_cell() {
        let rows = [row(
            "task.demo",
            None,
            "2026-08-28",
            "Harmless\u{1b}[2K\rShipped",
        )];
        let today_day = date::day_number("2026-08-28").unwrap();
        assert_eq!(
            ready_table(&rows, 1, today_day),
            "ready[1]{id,priority,age,title}:\n  task.demo,-,0d,\"Harmless\\u001b[2K\\rShipped\"\n"
        );
    }
}
