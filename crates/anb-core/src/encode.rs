//! The encoding every reply surface shares — the bound on a list, the
//! quoting rule, and the ready rows — so no two surfaces spell the same
//! value differently.

use crate::grammar;
use crate::reply::ReadyTask;
use std::fmt::Write as _;

/// How many rows a reply carries before it names the rest as a count.
///
/// What a command derives from the notebook is answered at a fixed size,
/// whatever the notebook holds; a listing lifts that with `--all`, and a
/// consequence named in passing has no lift. A record's own bytes are not
/// derived and are shown whole.
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
