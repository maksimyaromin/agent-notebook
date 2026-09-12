//! A refusal decomposed for either output format, and what it points back
//! at. Both renderings read this one value, so a refusal reads like a
//! reply and is bounded like one.

use crate::cli::Command;
use anb_core::StorageError;
use anb_core::encode::ROW_BOUND;
use anb_core::path_stem;
use anb_core::{Finding, NotebookError, RecordType};
use clap::CommandFactory as _;
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
        Command::Start { id, .. } => ("start", id.as_ref()),
        Command::Submit { id, .. } => ("submit", Some(id)),
        Command::Close(args) => ("close", Some(&args.id)),
        Command::Reopen { id } => ("reopen", Some(id)),
        Command::Hold { id, .. } => ("hold", Some(id)),
        Command::Unhold { id } => ("unhold", Some(id)),
        Command::Block { id, .. } => ("block", Some(id)),
        Command::Unblock { id, .. } => ("unblock", Some(id)),
        Command::Comment { id, .. } => ("comment", Some(id)),
        Command::Retire { id, .. } => ("retire", Some(id)),
        Command::Show { id, .. } => ("show", Some(id)),
        Command::Ready { .. } => ("ready", None),
        Command::Recall { .. } => ("recall", None),
        Command::Hook => ("hook", None),
        Command::List { .. } => ("list", None),
        Command::Status { .. } => ("status", None),
        Command::Check { .. } => ("check", None),
        Command::Import { .. } => ("import", None),
        Command::Migrate { .. } => ("migrate", None),
        Command::Debt { .. } => ("debt", None),
        Command::Archive { id } => ("archive", Some(id)),
        Command::Restore { id } => ("restore", Some(id)),
        Command::Delete { id } => ("delete", Some(id)),
        Command::Edit(args) => ("edit", Some(&args.id)),
        Command::Graph(_) => ("graph", None),
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
/// Details are complete; the shared reply projection selects a bounded
/// prefix and reports its total. Retries are at most [`ROW_BOUND`]
/// alternatives, not an exhaustive enumeration.
pub struct Recovery {
    pub code: &'static str,
    pub message: String,
    pub context: Vec<(&'static str, String)>,
    pub details: Vec<String>,
    pub tries: Vec<String>,
}

impl Recovery {
    #[must_use]
    pub fn new(error: &NotebookError, subject: &Subject) -> Self {
        let mut recovery = Recovery {
            code: error.code(),
            message: error.to_string(),
            context: Vec::new(),
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
            // Reading archived history does not require restoring it.
            NotebookError::Archived { id } => {
                recovery.tries.push(format!("anb show {id}"));
                recovery.tries.push(format!("anb restore {id}"));
            }
            NotebookError::WrongType { id, .. } => {
                recovery.tries.push(format!("anb show {id}"));
            }
            // Reassignment requires the caller to choose the intended person.
            NotebookError::Taken { id, .. } => {
                recovery
                    .tries
                    .push(format!("anb edit {id} --taken-by \"<name>\""));
                recovery.tries.push(format!("anb show {id}"));
            }
            NotebookError::SessionConflict { id, session } => {
                recovery.context = vec![("id", id.clone()), ("session", session.clone())];
                recovery.tries.push(format!("anb show {id}"));
                recovery.tries.push(format!("anb start {id} --join"));
            }
            NotebookError::SessionRecovery {
                session,
                path,
                reason,
            } => {
                recovery.context = vec![("path", path.clone()), ("reason", reason.clone())];
                if let Some(session) = session {
                    recovery.context.push(("session", session.clone()));
                    recovery.tries.push(format!(
                        "anb start --session {}",
                        anb_core::encode::shell_word(session)
                    ));
                } else {
                    recovery.tries.push("anb start --help".to_owned());
                }
            }
            NotebookError::InvalidRecord { path, findings } => {
                recovery.details = findings.iter().map(finding_line).collect();
                recovery.tries.push(format!("anb show {}", path_stem(path)));
            }
            NotebookError::InvalidTransition { id, valid, .. } => {
                recovery.invalid_transition(id, valid);
            }
            NotebookError::StillReferenced { blockers, .. } => {
                recovery.details = blockers.iter().map(ToString::to_string).collect();
                recovery.tries.extend(
                    anb_core::carriers_of(blockers)
                        .take(ROW_BOUND)
                        .map(|carrier| format!("anb show {carrier}")),
                );
            }
            NotebookError::UnfinishedDependencies { id, blockers } => {
                recovery.unfinished_dependencies(id, blockers);
            }
            NotebookError::DuplicateId { id, holder } => {
                recovery.duplicate_id(id, holder, subject.verb);
            }
            NotebookError::InvalidArgument { .. } => {
                recovery.tries = argument_retries(subject);
            }
            NotebookError::WouldCycle { chain } => {
                // The first pair is the refused edge; only existing edges can be removed.
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
        if subject.verb == "import" {
            // A failed import may name a source file that never entered
            // the target notebook. A target-only `show` would misdirect it.
            recovery.tries = vec!["anb import --help".to_owned()];
        }
        recovery
    }

    fn invalid_transition(&mut self, id: &str, valid: &[&str]) {
        for retry in valid
            .iter()
            .flat_map(|action| transition_retries(action, id))
        {
            if !self.tries.contains(&retry) {
                self.tries.push(retry);
            }
        }
    }

    fn unfinished_dependencies(&mut self, id: &str, blockers: &[String]) {
        self.context.push(("id", id.to_owned()));
        self.details = blockers.to_vec();
        self.tries.extend(
            blockers
                .iter()
                .take(ROW_BOUND)
                .map(|blocker| format!("anb show {blocker}")),
        );
    }

    fn duplicate_id(&mut self, id: &str, holder: &str, verb: &str) {
        if matches!(verb, "archive" | "restore") {
            let (live, archived) = if verb == "archive" {
                (
                    holder.strip_prefix("archive/").unwrap_or(holder).to_owned(),
                    holder.to_owned(),
                )
            } else {
                (holder.to_owned(), format!("archive/{holder}"))
            };
            self.message =
                format!("`{id}` has different or unreadable copies; the move preserved both files");
            self.details = vec![
                format!("live: {live}"),
                format!("archived: {archived}"),
                "Paths are relative to the selected notebook. Preserve both originals, compare their contents, and reconcile the intended record before removing either copy. Do not use anb delete: it removes both copies.".to_owned(),
            ];
            self.tries.push("anb check --all".to_owned());
        } else {
            self.tries.push(format!("anb show {id}"));
            self.tries.push("anb add task \"<title>\"".to_owned());
        }
    }
}

/// Recovery templates for commands that need additional arguments.
/// `None` leaves the caller to choose its fallback. CLI tests verify each template parses.
#[must_use]
pub fn runnable(verb: &str, id: Option<&str>) -> Option<Vec<String>> {
    let shapes = match (verb, id) {
        ("close", Some(id)) if id.starts_with("question.") => vec![
            format!("anb close {id} --resolved-by <id>"),
            format!("anb close {id} --reason \"<why>\""),
        ],
        ("close", Some(id)) => vec![
            format!("anb close {id} --body \"<outcome>\""),
            format!("anb close {id} --reason \"<why>\""),
        ],
        ("close --reason", Some(id)) => vec![format!("anb close {id} --reason \"<why>\"")],
        ("hold", Some(id)) => vec![format!("anb hold {id} --reason \"<why>\"")],
        ("comment", Some(id)) => vec![format!("anb comment {id} --body \"<text>\"")],
        ("retire", Some(id)) => vec![format!("anb retire {id} --body \"<outcome>\"")],
        ("edit", Some(id)) => vec![format!("anb edit {id} --title \"<title>\"")],
        ("add", _) => vec!["anb add task \"<title>\"".to_owned()],
        ("start", None) => vec!["anb start --next".to_owned(), "anb start <id>".to_owned()],
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

/// Supply the missing arguments where a command has a recovery template.
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

/// Convert argument-parser failures to the reply contract. Help and
/// version requests keep clap's own output and successful exit status.
#[must_use]
pub fn parse_recovery(error: &clap::Error) -> Option<Recovery> {
    if matches!(
        error.kind(),
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
    ) {
        return None;
    }
    if error.kind() != ErrorKind::InvalidSubcommand {
        let description = error.kind().as_str().unwrap_or("a command is required");
        let arguments = context_strings(error, ContextKind::InvalidArg);
        let message = if arguments.is_empty() {
            description.to_owned()
        } else {
            format!("{description}: {}", arguments.join(", "))
        };
        let tries = if error.kind() == ErrorKind::UnknownArgument {
            unknown_argument_retries(error)
        } else {
            vec!["anb --help".to_owned()]
        };
        return Some(Recovery {
            code: "invalid-argument",
            message,
            context: Vec::new(),
            details: Vec::new(),
            tries,
        });
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
        context: Vec::new(),
        details: Vec::new(),
        tries,
    })
}

fn unknown_argument_retries(error: &clap::Error) -> Vec<String> {
    let usage = error
        .get(ContextKind::Usage)
        .map(ToString::to_string)
        .unwrap_or_default();
    let command = crate::cli::Cli::command();
    let Some(verb) = usage
        .split_whitespace()
        .nth(2)
        .filter(|verb| command.find_subcommand(verb).is_some())
    else {
        return vec!["anb --help".to_owned()];
    };
    let mut tries = vec![format!("anb {verb} --help")];
    match verb {
        "comment" => tries.push("anb comment <id> -- \"<text>\"".to_owned()),
        "edit" => tries.push("anb edit <id> --body=\"<text>\"".to_owned()),
        _ => {}
    }
    tries
}

/// The strings clap recorded under `kind`, however it wrapped them.
fn context_strings(error: &clap::Error, kind: ContextKind) -> Vec<String> {
    match error.get(kind) {
        Some(ContextValue::String(value)) => vec![value.clone()],
        Some(ContextValue::Strings(values)) => values.clone(),
        _ => Vec::new(),
    }
}
