//! A refusal decomposed for either output format, and what it points back
//! at. Both renderings read this one value, so a refusal reads like a
//! reply and is bounded like one.

use crate::cli::Command;
use anb_core::StorageError;
use anb_core::encode::ROW_BOUND;
use anb_core::path_stem;
use anb_core::{Finding, NotebookError, RecordType};
use clap::error::{ContextKind, ContextValue, ErrorKind};

/// What a refusal can point back at: the verb, the record it named, and
/// for `add` the type it was creating.
pub struct Subject {
    pub verb: &'static str,
    pub id: Option<String>,
    pub record_type: Option<RecordType>,
}

/// The command's subject, taken before dispatch consumes the command.
#[must_use]
pub fn subject(command: &Command) -> Subject {
    let (verb, id) = match command {
        Command::Add(args) => {
            return Subject {
                verb: "add",
                id: None,
                record_type: Some(args.record_type),
            };
        }
        Command::Start { id } => ("start", Some(id)),
        Command::Submit { id } => ("submit", Some(id)),
        Command::Close(args) => ("close", Some(&args.id)),
        Command::Reopen { id } => ("reopen", Some(id)),
        Command::Hold { id, .. } => ("hold", Some(id)),
        Command::Unhold { id } => ("unhold", Some(id)),
        Command::Block { id, .. } => ("block", Some(id)),
        Command::Unblock { id, .. } => ("unblock", Some(id)),
        Command::Comment { id, .. } => ("comment", Some(id)),
        Command::Retire { id } => ("retire", Some(id)),
        Command::Show { id, .. } => ("show", Some(id)),
        Command::Ready { .. } => ("ready", None),
        Command::List { .. } => ("list", None),
        Command::Status { .. } => ("status", None),
        Command::Check { .. } => ("check", None),
        Command::Archive { id } => ("archive", Some(id)),
        Command::Restore { id } => ("restore", Some(id)),
        Command::Delete { id } => ("delete", Some(id)),
        Command::Edit(args) => ("edit", Some(&args.id)),
        Command::Search { .. } => ("search", None),
        Command::Graph(_) => ("graph", None),
        Command::Overview { .. } => ("overview", None),
        Command::Setup { .. } => ("setup", None),
        Command::Skill { .. } => ("skill", None),
    };
    Subject {
        verb,
        id: id.cloned(),
        record_type: None,
    }
}

/// A refusal decomposed for either output format: the stable kebab-case
/// code, the one-line message, detail lines, and the next commands computed
/// from the refusal's own state.
///
/// A refusal reads like a reply and is bounded like one: its details and
/// its retries stop at [`ROW_BOUND`]. The retries need no marker — they
/// are alternatives, not an enumeration — but the details are the
/// notebook speaking, so a cut one says how much it cut.
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
            code: error.code(),
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
                        .push(format!("anb add task \"<title>\" --id {target}"));
                }
                recovery.tries.push("anb list".to_owned());
            }
            // `show` leads: it is right on every archived record, while
            // `restore` pulls settled history back into the working set —
            // right only when the reader means to.
            NotebookError::Archived { id } => {
                recovery.tries.push(format!("anb show {id}"));
                recovery.tries.push(format!("anb restore {id}"));
            }
            NotebookError::WrongType { id, .. } => {
                recovery.tries.push(format!("anb show {id}"));
            }
            // A hand-over is decided on purpose, so the name is left for the
            // caller to fill rather than filled with their own.
            NotebookError::Taken { id, .. } => {
                recovery
                    .tries
                    .push(format!("anb edit {id} --taken-by \"<name>\""));
                recovery.tries.push(format!("anb show {id}"));
            }
            NotebookError::InvalidRecord { path, findings } => {
                recovery.details = bounded(findings.iter().map(finding_line).collect());
                recovery.tries.push(format!("anb show {}", path_stem(path)));
            }
            NotebookError::InvalidTransition { id, valid, .. } => {
                // `close` and `close --reason` both offer the reason shape;
                // one line suffices however many moves reach it.
                for retry in valid
                    .iter()
                    .flat_map(|action| transition_retries(action, id))
                {
                    if !recovery.tries.contains(&retry) {
                        recovery.tries.push(retry);
                    }
                }
            }
            NotebookError::StillReferenced { blockers, .. } => {
                recovery.details = bounded(blockers.iter().map(ToString::to_string).collect());
                recovery.tries.extend(
                    anb_core::carriers_of(blockers)
                        .take(ROW_BOUND)
                        .map(|carrier| format!("anb show {carrier}")),
                );
            }
            NotebookError::DuplicateId { id, .. } => {
                recovery.tries.push(format!("anb show {id}"));
                recovery.tries.push("anb add task \"<title>\"".to_owned());
            }
            NotebookError::InvalidArgument { .. } => {
                recovery.tries = argument_retries(subject);
            }
            NotebookError::WouldCycle { chain } => {
                // The chain's first pair is the refused edge; the rest
                // already stand, and erasing any one of them opens it — so
                // a long cycle needs no more retries than a short one.
                recovery.tries = chain
                    .windows(2)
                    .skip(1)
                    .take(ROW_BOUND)
                    .map(|edge| format!("anb unblock {} {}", edge[0], edge[1]))
                    .collect();
            }
            NotebookError::Storage(StorageError::NotUtf8 { .. }) => {
                recovery.tries.push("anb check".to_owned());
            }
            NotebookError::CannotSupersede { .. } | NotebookError::Storage(_) => {}
        }
        recovery
    }
}

/// A detail list cut to [`ROW_BOUND`], the cut named as a final line. The
/// message above cannot say it: it counts the records at fault, and one
/// record can hold a reference through several lines at once.
fn bounded(details: Vec<String>) -> Vec<String> {
    let total = details.len();
    let mut lines: Vec<String> = details.into_iter().take(ROW_BOUND).collect();
    if total > ROW_BOUND {
        lines.push(format!("\u{2026} {} more", total - ROW_BOUND));
    }
    lines
}

/// The verbs that need a flag to run, as the command lines that supply
/// one. Both retry paths read this table, so a refusal and a retry never
/// offer a caller two different ways to do the same thing; `None` is a verb
/// this table has no shape for, and each caller falls back its own way.
///
/// The shapes are what an agent types next, so they are part of the
/// command-line contract and are read back by the test that keeps them
/// runnable.
#[must_use]
pub fn runnable(verb: &str, id: Option<&str>) -> Option<Vec<String>> {
    let shapes = match (verb, id) {
        ("close", Some(id)) if id.starts_with("question.") => vec![
            format!("anb close {id} --resolved-by <id>"),
            format!("anb close {id} --reason \"<why>\""),
        ],
        ("close", Some(id)) => vec![
            format!("anb close {id} --note <path>"),
            format!("anb close {id} --no-proof"),
            format!("anb close {id} --reason \"<why>\""),
        ],
        ("close --reason", Some(id)) => vec![format!("anb close {id} --reason \"<why>\"")],
        ("hold", Some(id)) => vec![format!("anb hold {id} --reason \"<why>\"")],
        ("comment", Some(id)) => vec![format!("anb comment {id} \"<one line>\"")],
        ("edit", Some(id)) => vec![format!("anb edit {id} --title \"<title>\"")],
        ("add", _) => vec!["anb add task \"<title>\"".to_owned()],
        ("search", _) => vec!["anb search \"<text>\"".to_owned()],
        ("setup", _) => crate::setup::agent_names()
            .into_iter()
            .map(|agent| format!("anb setup --agent {agent}"))
            .collect(),
        _ => return None,
    };
    Some(shapes)
}

/// A valid next state as the command that reaches it.
fn transition_retries(action: &str, id: &str) -> Vec<String> {
    runnable(action, Some(id)).unwrap_or_else(|| vec![format!("anb {action} {id}")])
}

/// The retry a refused argument points at: the same verb in a shape that
/// carries what it was missing. A verb whose bare form already runs has
/// nothing to offer — repeating what was just refused is no recovery.
fn argument_retries(subject: &Subject) -> Vec<String> {
    if let Some(record_type) = subject.record_type {
        return vec![format!("anb add {} \"<title>\"", record_type.word())];
    }
    runnable(subject.verb, subject.id.as_deref()).unwrap_or_default()
}

fn finding_line(finding: &Finding) -> String {
    match finding.line {
        Some(line) => format!("line {line}: {} {}", finding.code.as_str(), finding.message),
        None => format!("{} {}", finding.code.as_str(), finding.message),
    }
}

/// The recovery payload for a verb clap does not know — an agent typing an
/// unknown or not-yet-built command gets a next step, not raw usage. `None`
/// for everything else clap refuses (or serves, like `--help`), which keeps
/// clap's rendering.
#[must_use]
pub fn unknown_command_recovery(error: &clap::Error) -> Option<Recovery> {
    if error.kind() != ErrorKind::InvalidSubcommand {
        return None;
    }
    let verb = context_strings(error, ContextKind::InvalidSubcommand)
        .into_iter()
        .next()
        .unwrap_or_default();
    // `--help` keeps every suggestion runnable whatever arguments the
    // suggested verb requires.
    let mut tries: Vec<String> = context_strings(error, ContextKind::SuggestedSubcommand)
        .into_iter()
        .map(|nearest| format!("anb {nearest} --help"))
        .collect();
    tries.push("anb --help".to_owned());
    Some(Recovery {
        code: "unknown-command",
        message: format!("`{verb}` is not an anb command"),
        details: Vec::new(),
        tries,
    })
}

/// The strings clap recorded under `kind`, however it wrapped them.
fn context_strings(error: &clap::Error, kind: ContextKind) -> Vec<String> {
    match error.get(kind) {
        Some(ContextValue::String(value)) => vec![value.clone()],
        Some(ContextValue::Strings(values)) => values.clone(),
        _ => Vec::new(),
    }
}
