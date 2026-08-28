//! The plain-text encoding every table surface shares — the quoting rule
//! and the ready rows — so the Status and the flat lists cannot drift
//! apart on the same value.

use crate::grammar;
use crate::notebook::ReadyTask;
use std::fmt::Write as _;

/// A table value carrying the row delimiter or a quote is JSON-quoted;
/// everything else stays bare.
#[must_use]
pub fn quoted_if_delimited(value: &str) -> String {
    if value.contains(',') || value.contains('"') {
        json_quoted(value)
    } else {
        value.to_owned()
    }
}

/// Quotes the value, escaping the backslash and the quote.
#[must_use]
pub(crate) fn json_quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The ready table — header and the first `shown` rows, ages derived from
/// `today_day`. The caller owns the count line and its own truncation hint.
#[must_use]
pub fn ready_table(rows: &[ReadyTask], shown: usize, today_day: i64) -> String {
    let mut out = format!("ready[{shown}]{{id,priority,age,title}}:\n");
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
    grammar::day_number(created).map_or(0, |day| (today_day - day).max(0))
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

    #[test]
    fn a_row_derives_its_age_and_quotes_a_delimited_title() {
        let rows = [row(
            "task.demo",
            None,
            "2026-08-24",
            "Degrades sections, keeps Budget",
        )];
        let today_day = grammar::day_number("2026-08-28").unwrap();
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
}
