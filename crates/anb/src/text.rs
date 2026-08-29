//! The plain-text rendering: shape-matched output under the contract —
//! a leading `ok:` line with the transition and its computed consequences,
//! header+rows tables for flat lists, labeled `key: value` for one record,
//! and every refusal as a recovery payload with literal next commands.

use crate::cli::Subject;
use crate::json;
use crate::reply::{Recovery, Reply, shown};
use anb_core::encode::quoted_if_delimited;
use anb_core::{
    FileFinding, ListedRecord, NotebookError, Overview, ReadyTask, View, counts_phrase, encode,
    grammar,
};
use std::fmt::Write as _;

#[must_use]
pub fn render(reply: &Reply, today: &str) -> String {
    match reply {
        Reply::Created { command, created } => created_lines(command, created),
        Reply::Moved {
            command,
            transition,
        } => transition_line(command, transition),
        Reply::Routed { transition, to } => {
            let mut out = transition_line("answer", transition);
            let _ = writeln!(out, "routed-to: {to}");
            out
        }
        Reply::Dropped(dropped) => {
            let mut out = transition_line("answer", &dropped.transition);
            dangling_mention_line(&mut out, &dropped.dangling_mentions);
            out
        }
        Reply::Closed(closed) => closed_lines(closed),
        Reply::Held { held, until } => {
            let outcome = match until {
                Some(until) => format!("held until {until}"),
                None => "held".to_owned(),
            };
            format!(
                "ok: hold {} — {outcome}{}\n",
                held.id,
                already_mark(held.already)
            )
        }
        Reply::Unheld(held) => format!(
            "ok: unhold {} — unheld{}\n",
            held.id,
            already_mark(held.already)
        ),
        Reply::Blocked(edge) => format!(
            "ok: block {} — waits on {}{}\n",
            edge.id,
            edge.on,
            already_mark(edge.already)
        ),
        Reply::Unblocked(edge) => format!(
            "ok: unblock {} — edge on {} erased{}\n",
            edge.id,
            edge.on,
            already_mark(edge.already)
        ),
        Reply::Commented(commented) => commented_lines(commented),
        Reply::Ready { rows, all } => ready_table(rows, shown(rows.len(), *all), today),
        Reply::Listing { rows, all } => {
            listing_table(rows, shown(rows.len(), *all), "records", "anb list --all")
        }
        Reply::Viewed(view) => single_record(view),
        Reply::Checked { findings, all } => findings_table(findings, shown(findings.len(), *all)),
        Reply::Archived(moved) => {
            if moved.already {
                format!("ok: archive {} — archived (already)\n", moved.id)
            } else {
                format!(
                    "ok: archive {} — {}\u{2192}{}\n",
                    moved.id, moved.from, moved.to
                )
            }
        }
        Reply::Expunged(gone) => format!(
            "ok: expunge {} — {} removed\n",
            gone.id,
            gone.paths.join(", ")
        ),
        Reply::Edited(edited) => {
            let mut out = if edited.changed.is_empty() {
                format!("ok: edit {} — unchanged (already)\n", edited.id)
            } else {
                format!("ok: edit {} — {}\n", edited.id, edited.changed.join(", "))
            };
            dangling_mention_line(&mut out, &edited.dangling_mentions);
            out
        }
        Reply::Searched { query, rows, all } => listing_table(
            rows,
            shown(rows.len(), *all),
            "matches",
            &format!("anb search {} --all", shell_quoted(query)),
        ),
        Reply::Overviewed { overview, all } => overview_page(overview, *all),
        Reply::Status { status, hook } => {
            if *hook {
                json::hook_payload(status)
            } else {
                status.text.clone()
            }
        }
        Reply::Silence => String::new(),
    }
}

/// The refusal as text: the stable code, the message, the detail lines a
/// repair needs, and the literal next commands.
#[must_use]
pub fn render_error(error: &NotebookError, subject: &Subject) -> String {
    render_recovery(&Recovery::new(error, subject))
}

#[must_use]
pub fn render_recovery(recovery: &Recovery) -> String {
    let mut out = format!("error[{}]: {}\n", recovery.code, recovery.message);
    for detail in &recovery.details {
        let _ = writeln!(out, "  {detail}");
    }
    for suggestion in &recovery.tries {
        let _ = writeln!(out, "try: {suggestion}");
    }
    out
}

fn already_mark(already: bool) -> &'static str {
    if already { " (already)" } else { "" }
}

fn created_lines(command: &str, created: &anb_core::Created) -> String {
    let mut out = format!("ok: {command} {} — {}\n", created.id, created.path);
    if let Some(victim) = &created.superseded {
        let _ = writeln!(out, "superseded: {victim}");
    }
    if !created.may_conflict.is_empty() {
        let named: Vec<String> = created
            .may_conflict
            .iter()
            .map(|cited| format!("{} ({})", cited.id, cited.author()))
            .collect();
        let _ = writeln!(
            out,
            "may-conflict[{}]: {}",
            created.may_conflict.len(),
            named.join(", ")
        );
    }
    dangling_mention_line(&mut out, &created.dangling_mentions);
    out
}

fn commented_lines(commented: &anb_core::Commented) -> String {
    let mut out = format!(
        "ok: comment {} — logged{}\n",
        commented.id,
        already_mark(commented.already)
    );
    dangling_mention_line(&mut out, &commented.dangling_mentions);
    out
}

fn closed_lines(closed: &anb_core::Closed) -> String {
    let mut out = transition_line("close", &closed.transition);
    if let Some(note) = &closed.report_note {
        let _ = writeln!(out, "report: {note}");
    }
    dangling_mention_line(&mut out, &closed.dangling_mentions);
    if !closed.unblocked.is_empty() {
        let _ = writeln!(
            out,
            "unblocked[{}]: {}",
            closed.unblocked.len(),
            closed.unblocked.join(", ")
        );
    }
    if !closed.open_questions.is_empty() {
        let _ = writeln!(
            out,
            "open-questions[{}]: {}",
            closed.open_questions.len(),
            closed.open_questions.join(", ")
        );
    }
    out
}

/// The quotation rule's write-time nudge.
fn dangling_mention_line(out: &mut String, ids: &[String]) {
    if ids.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "dangling-mention[{}]: {} — backtick to quote, or create the record",
        ids.len(),
        ids.join(", ")
    );
}

/// A move as `from→to`; a replay names the standing state instead of a
/// self-loop.
fn transition_line(command: &str, transition: &anb_core::Transitioned) -> String {
    if transition.already {
        format!(
            "ok: {command} {} — {} (already)\n",
            transition.id, transition.to
        )
    } else {
        format!(
            "ok: {command} {} — {}\u{2192}{}\n",
            transition.id, transition.from, transition.to
        )
    }
}

fn ready_table(rows: &[ReadyTask], shown: usize, today: &str) -> String {
    let mut out = format!("count: {}\n", rows.len());
    if rows.is_empty() {
        return out;
    }
    let today_day = grammar::day_number(today).unwrap_or(0);
    out.push_str(&encode::ready_table(rows, shown, today_day));
    truncation_hint(&mut out, rows.len(), shown, "anb ready --all");
    out
}

fn listing_table(rows: &[ListedRecord], shown: usize, label: &str, restore: &str) -> String {
    let mut out = format!("count: {}\n", rows.len());
    if rows.is_empty() {
        return out;
    }
    out.push_str(&record_rows(rows, shown, label));
    truncation_hint(&mut out, rows.len(), shown, restore);
    out
}

/// The one shape of a record-listing block: header, then comma rows.
fn record_rows(rows: &[ListedRecord], shown: usize, label: &str) -> String {
    let mut out = format!("{label}[{shown}]{{id,state,priority,title}}:\n");
    for row in &rows[..shown] {
        let priority = row
            .priority
            .map_or_else(|| "-".to_owned(), |priority| priority.to_string());
        let title = row
            .title
            .as_deref()
            .map_or_else(|| "-".to_owned(), quoted_if_delimited);
        let _ = writeln!(out, "  {},{},{priority},{title}", row.id, row.state);
    }
    out
}

fn findings_table(findings: &[FileFinding], shown: usize) -> String {
    let mut out = format!("count: {}\n", findings.len());
    if findings.is_empty() {
        return out;
    }
    let _ = writeln!(out, "findings[{shown}]{{file,line,severity,code,message}}:");
    for located in &findings[..shown] {
        let line = located
            .finding
            .line
            .map_or_else(|| "-".to_owned(), |line| line.to_string());
        let _ = writeln!(
            out,
            "  {},{line},{},{},{}",
            located.path,
            located.finding.code.severity().as_str(),
            located.finding.code.as_str(),
            quoted_if_delimited(&located.finding.message)
        );
    }
    truncation_hint(&mut out, findings.len(), shown, "anb check --all");
    out
}

fn overview_page(overview: &Overview, all: bool) -> String {
    let mut out = format!("notebook: {}\n", counts_phrase(&overview.live));
    if !overview.epics.is_empty() {
        let _ = writeln!(out, "epics[{}]:", overview.epics.len());
        for epic in &overview.epics {
            let _ = writeln!(out, "  {}", anb_core::epic_line(epic));
        }
    }
    for section in &overview.sections {
        if section.rows.is_empty() {
            continue;
        }
        let shown = shown(section.rows.len(), all);
        out.push_str(&record_rows(
            &section.rows,
            shown,
            section.record_type.directory(),
        ));
        truncation_hint(&mut out, section.rows.len(), shown, "anb overview --all");
    }
    let archived = &overview.archived;
    if archived.tasks + archived.decisions + archived.notes + archived.questions > 0 {
        let _ = writeln!(out, "archive: {}", counts_phrase(archived));
    }
    out
}

/// The query as one shell word, single-quoted so every character stays
/// literal and the truncation hint stays executable.
fn shell_quoted(query: &str) -> String {
    let bare = query
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if bare && !query.is_empty() {
        query.to_owned()
    } else {
        format!("'{}'", query.replace('\'', "'\\''"))
    }
}

fn truncation_hint(out: &mut String, total: usize, shown: usize, restore: &str) {
    if total > shown {
        let _ = writeln!(out, "  \u{2026} {} more: {restore}", total - shown);
    }
}

fn single_record(view: &View) -> String {
    let mut out = String::new();
    for (key, value) in &view.fields {
        if value.is_empty() {
            let _ = writeln!(out, "{key}:");
        } else {
            let _ = writeln!(out, "{key}: {value}");
        }
    }
    if view.archived {
        out.push_str("archived: true\n");
    }
    if !view.body.is_empty() {
        out.push_str("body: |\n");
        for line in view.body.lines() {
            if line.is_empty() {
                out.push('\n');
            } else {
                let _ = writeln!(out, "  {line}");
            }
        }
    }
    for (label, ids) in [
        ("mentions", &view.mentions),
        ("mentioned-by", &view.mentioned_by),
    ] {
        if !ids.is_empty() {
            let _ = writeln!(out, "{label}[{}]: {}", ids.len(), ids.join(", "));
        }
    }
    out
}
