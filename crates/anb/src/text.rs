//! The plain-text rendering: shape-matched output under the contract —
//! a leading `ok:` line with the transition and its computed consequences,
//! header+rows tables for flat lists, labeled `key: value` for one record,
//! and every refusal as a recovery payload with literal next commands.

use crate::cli::Subject;
use crate::json;
use crate::reply::{Reply, shown};
use anb_core::encode::quoted_if_delimited;
use anb_core::notebook::path_stem;
use anb_core::{Finding, ListedRecord, NotebookError, ReadyTask, View, encode, grammar};
use std::fmt::Write as _;

#[must_use]
pub fn render(reply: &Reply, today: &str) -> String {
    match reply {
        Reply::Created { command, created } => {
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
            out
        }
        Reply::Moved {
            command,
            transition,
        } => transition_line(command, transition),
        Reply::Answered { transition, to } => {
            let mut out = transition_line("answer", transition);
            if let Some(to) = to {
                let _ = writeln!(out, "routed-to: {to}");
            }
            out
        }
        Reply::Closed(closed) => {
            let mut out = transition_line("close", &closed.transition);
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
        Reply::Commented(commented) => format!(
            "ok: comment {} — logged{}\n",
            commented.id,
            already_mark(commented.already)
        ),
        Reply::Ready { rows, all } => ready_table(rows, shown(rows.len(), *all), today),
        Reply::Listing { rows, all } => listing_table(rows, shown(rows.len(), *all)),
        Reply::Viewed(view) => single_record(view),
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
    let recovery = Recovery::new(error, subject);
    let mut out = format!("error[{}]: {}\n", recovery.code, recovery.message);
    for detail in &recovery.details {
        let _ = writeln!(out, "  {detail}");
    }
    for suggestion in &recovery.tries {
        let _ = writeln!(out, "try: {suggestion}");
    }
    out
}

/// A refusal decomposed for either output format: the stable kebab-case
/// code, the one-line message, detail lines, and the next commands computed
/// from the refusal's own state.
pub struct Recovery {
    pub code: &'static str,
    pub message: String,
    pub details: Vec<String>,
    pub tries: Vec<String>,
}

impl Recovery {
    #[must_use]
    pub fn new(error: &NotebookError, subject: &Subject) -> Self {
        let mut recovery = Recovery {
            code: error_code(error),
            message: error.to_string(),
            details: Vec::new(),
            tries: Vec::new(),
        };
        match error {
            NotebookError::UnknownId { .. } => {
                recovery.tries.push("anb list".to_owned());
            }
            NotebookError::DanglingRef { target, .. } => {
                if target.starts_with("task.") {
                    recovery
                        .tries
                        .push(format!("anb add \"<title>\" --id {target}"));
                }
                recovery.tries.push("anb list".to_owned());
            }
            NotebookError::Archived { id } | NotebookError::WrongType { id, .. } => {
                recovery.tries.push(format!("anb view {id}"));
            }
            NotebookError::InvalidRecord { path, findings } => {
                recovery.details = findings.iter().map(finding_line).collect();
                recovery.tries.push(format!("anb view {}", path_stem(path)));
            }
            NotebookError::InvalidTransition { id, valid, .. } => {
                for action in valid {
                    recovery.tries.push(match *action {
                        "close" => format!("anb close {id} --report <path>"),
                        action => format!("anb {action} {id}"),
                    });
                }
            }
            NotebookError::DuplicateId { id, .. } => {
                recovery.tries.push(format!("anb view {id}"));
                recovery.tries.push("anb add \"<title>\"".to_owned());
            }
            NotebookError::InvalidArgument { .. } => {
                recovery.tries = argument_retries(subject);
            }
            NotebookError::WouldCycle { chain } => {
                // The chain's first pair is the refused edge; the rest
                // already stand, and erasing any one of them opens it.
                recovery.tries = chain
                    .windows(2)
                    .skip(1)
                    .map(|edge| format!("anb unblock {} {}", edge[0], edge[1]))
                    .collect();
            }
            NotebookError::CannotSupersede { .. } | NotebookError::Storage(_) => {}
        }
        recovery
    }
}

/// The retry a refused argument points at, keyed by the verb it refused;
/// the create verbs carry no id and retry as a command shape.
fn argument_retries(subject: &Subject) -> Vec<String> {
    match (subject.verb, &subject.id) {
        ("close", Some(id)) => vec![
            format!("anb close {id} --pr <url>"),
            format!("anb close {id} --no-proof"),
        ],
        ("hold", Some(id)) => vec![format!("anb hold {id} --reason \"<why>\"")],
        ("comment", Some(id)) => vec![format!("anb comment {id} \"<one line>\"")],
        ("answer", Some(id)) => vec![
            format!("anb answer {id} --to <id>"),
            format!("anb answer {id} --drop \"<why>\""),
        ],
        ("add", _) => vec!["anb add \"<title>\"".to_owned()],
        ("decide", _) => vec!["anb decide \"<title>\" --kind rule".to_owned()],
        ("note", _) => vec!["anb note \"<title>\" --kind fact".to_owned()],
        ("ask", _) => vec!["anb ask \"<title>\"".to_owned()],
        _ => Vec::new(),
    }
}

fn error_code(error: &NotebookError) -> &'static str {
    match error {
        NotebookError::UnknownId { .. } => "unknown-id",
        NotebookError::Archived { .. } => "archived",
        NotebookError::InvalidRecord { .. } => "invalid-record",
        NotebookError::WrongType { .. } => "wrong-type",
        NotebookError::InvalidTransition { .. } => "invalid-transition",
        NotebookError::InvalidArgument { .. } => "invalid-argument",
        NotebookError::DuplicateId { .. } => "duplicate-id",
        NotebookError::DanglingRef { .. } => "dangling-ref",
        NotebookError::CannotSupersede { .. } => "cannot-supersede",
        NotebookError::WouldCycle { .. } => "would-cycle",
        NotebookError::Storage(_) => "storage",
    }
}

fn finding_line(finding: &Finding) -> String {
    match finding.line {
        Some(line) => format!("line {line}: {} {}", finding.code.as_str(), finding.message),
        None => format!("{} {}", finding.code.as_str(), finding.message),
    }
}

fn already_mark(already: bool) -> &'static str {
    if already { " (already)" } else { "" }
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

fn listing_table(rows: &[ListedRecord], shown: usize) -> String {
    let mut out = format!("count: {}\n", rows.len());
    if rows.is_empty() {
        return out;
    }
    let _ = writeln!(out, "records[{shown}]{{id,state,priority,title}}:");
    for row in &rows[..shown] {
        let priority = row
            .priority
            .map_or_else(|| "-".to_owned(), |priority| priority.to_string());
        let _ = writeln!(
            out,
            "  {},{},{priority},{}",
            row.id,
            row.state,
            quoted_if_delimited(&row.title)
        );
    }
    truncation_hint(&mut out, rows.len(), shown, "anb list --all");
    out
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
