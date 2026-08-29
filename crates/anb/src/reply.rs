//! One command in, one reply out: the dispatch from the parsed surface to
//! the Core, with nothing rendered yet — both output formats read the same
//! reply.

use crate::cli::{AddArgs, Command, DecideArgs, DraftArgs, NoteArgs};
use anb_core::{
    Budget, Closed, Commented, Created, Draft, Edged, Held, Link, ListedRecord, Notebook,
    NotebookError, Proof, ReadyTask, RecordType, Status, Storage, Transitioned, View,
};

/// How many rows a flat list shows before the truncation hint; one
/// unbounded listing costs more tokens than any encoding saves.
pub const ROW_BOUND: usize = 20;

/// The first bounded prefix of a flat list; both renderers show the same
/// rows.
#[must_use]
pub fn shown(total: usize, all: bool) -> usize {
    if all { total } else { total.min(ROW_BOUND) }
}

/// What a command came to; the renderers turn one of these into text.
#[derive(Debug)]
pub enum Reply {
    Created {
        command: &'static str,
        created: Created,
    },
    Moved {
        command: &'static str,
        transition: Transitioned,
    },
    /// A Question closed: routed into `to`, or dropped when there is none.
    Answered {
        transition: Transitioned,
        to: Option<String>,
    },
    Closed(Closed),
    Held {
        held: Held,
        until: Option<String>,
    },
    Unheld(Held),
    Blocked(Edged),
    Unblocked(Edged),
    Commented(Commented),
    Ready {
        rows: Vec<ReadyTask>,
        all: bool,
    },
    Listing {
        rows: Vec<ListedRecord>,
        all: bool,
    },
    Viewed(View),
    Status {
        status: Status,
        hook: bool,
    },
    /// The hook's fail-soft outcome: no context rather than a blocked
    /// session.
    Silence,
}

/// Run `command` against the notebook behind `storage`.
///
/// `git_by` supplies the accountable identity from git, read only by the
/// commands that write one; a command's own flag outranks it. `today` is
/// the host's date — the Core holds no clock.
///
/// # Errors
/// The Core's refusal, or the shell's own argument refusal — either
/// renders as a recovery payload.
pub fn execute<S: Storage>(
    command: Command,
    storage: &mut S,
    git_by: impl FnOnce() -> Option<String>,
    today: &str,
) -> Result<Reply, NotebookError> {
    let mut notebook = Notebook::new(storage);
    match command {
        Command::Add(args) => Ok(Reply::Created {
            command: "add",
            created: notebook.create(&task_draft(args, git_by), today)?,
        }),
        Command::Decide(args) => Ok(Reply::Created {
            command: "decide",
            created: notebook.create(&decision_draft(args, git_by), today)?,
        }),
        Command::Note(args) => Ok(Reply::Created {
            command: "note",
            created: notebook.create(&note_draft(args, git_by), today)?,
        }),
        Command::Ask(args) => Ok(Reply::Created {
            command: "ask",
            created: notebook.create(&draft(RecordType::Question, args, git_by), today)?,
        }),
        Command::Answer { id, to, drop } => answered(&mut notebook, &id, to, drop, today),
        Command::Retire { id } => Ok(Reply::Moved {
            command: "retire",
            transition: notebook.retire(&id, today)?,
        }),
        Command::Start { id } => Ok(Reply::Moved {
            command: "start",
            transition: notebook.start(&id, today)?,
        }),
        Command::Submit { id } => Ok(Reply::Moved {
            command: "submit",
            transition: notebook.submit(&id, today)?,
        }),
        Command::Close(args) => {
            let proof = chosen_proof(args.pr, args.sha, args.report, args.no_proof)?;
            Ok(Reply::Closed(notebook.close(&args.id, &proof, today)?))
        }
        Command::Return { id } => Ok(Reply::Moved {
            command: "return",
            transition: notebook.return_task(&id, today)?,
        }),
        Command::Reopen { id } => Ok(Reply::Moved {
            command: "reopen",
            transition: notebook.reopen(&id, today)?,
        }),
        Command::Hold { id, reason, until } => Ok(Reply::Held {
            held: notebook.hold(
                &id,
                reason.as_deref().unwrap_or(""),
                until.as_deref(),
                today,
            )?,
            until,
        }),
        Command::Unhold { id } => Ok(Reply::Unheld(notebook.unhold(&id, today)?)),
        Command::Block { id, on } => Ok(Reply::Blocked(notebook.block(&id, &on, today)?)),
        Command::Unblock { id, on } => Ok(Reply::Unblocked(notebook.unblock(&id, &on, today)?)),
        Command::Comment { id, text, via } => {
            let author = via.or_else(git_by);
            Ok(Reply::Commented(notebook.comment(
                &id,
                author.as_deref(),
                &text,
                today,
            )?))
        }
        Command::Ready { all } => Ok(Reply::Ready {
            rows: notebook.ready()?,
            all,
        }),
        Command::List { all } => Ok(Reply::Listing {
            rows: notebook.list()?,
            all,
        }),
        Command::View { id } => Ok(Reply::Viewed(notebook.view(&id)?)),
        Command::Status { budget, hook } => {
            match (budgeted_status(&notebook, budget, today), hook) {
                (Ok(status), hook) => Ok(Reply::Status { status, hook }),
                // The session-start hook fails soft: an empty context,
                // never a blocked session.
                (Err(_), true) => Ok(Reply::Silence),
                (Err(error), false) => Err(error),
            }
        }
    }
}

/// The Status under the resolved ceiling: the flag outranks the config key.
fn budgeted_status<S: Storage>(
    notebook: &Notebook<'_, S>,
    budget: Option<u32>,
    today: &str,
) -> Result<Status, NotebookError> {
    let ceiling = match budget {
        Some(ceiling) => Budget::from_ceiling(ceiling),
        None => notebook.config()?.budget(),
    };
    notebook.status(today, ceiling)
}

fn answered<S: Storage>(
    notebook: &mut Notebook<'_, S>,
    id: &str,
    to: Option<String>,
    drop: Option<String>,
    today: &str,
) -> Result<Reply, NotebookError> {
    match chosen_routing(to, drop)? {
        Routing::To(target) => Ok(Reply::Answered {
            transition: notebook.route(id, &target, today)?,
            to: Some(target),
        }),
        Routing::Drop(reason) => Ok(Reply::Answered {
            transition: notebook.drop_question(id, &reason, today)?,
            to: None,
        }),
    }
}

fn task_draft(args: AddArgs, git_by: impl FnOnce() -> Option<String>) -> Draft {
    let mut task = draft(RecordType::Task, args.draft, git_by);
    task.priority = args.priority;
    task
}

fn decision_draft(args: DecideArgs, git_by: impl FnOnce() -> Option<String>) -> Draft {
    let mut decision = draft(RecordType::Decision, args.draft, git_by);
    decision.kind = args.kind;
    decision.supersedes = args.supersedes;
    decision
}

fn note_draft(args: NoteArgs, git_by: impl FnOnce() -> Option<String>) -> Draft {
    let mut note = draft(RecordType::Note, args.draft, git_by);
    note.kind = args.kind;
    note.supersedes = args.supersedes;
    note
}

fn draft(
    record_type: RecordType,
    args: DraftArgs,
    git_by: impl FnOnce() -> Option<String>,
) -> Draft {
    let mut draft = Draft::new(record_type, &args.title);
    draft.id = args.id;
    draft.by = args.by.or_else(git_by);
    draft.via = args.via;
    draft.from = args.from;
    draft.tags = args.tags;
    draft.links = args.links.iter().map(|raw| parsed_link(raw)).collect();
    draft.body = args.body.unwrap_or_default();
    draft
}

/// `<kind> <target>` split at the first space; a link without one arrives
/// with an empty target, which the Core refuses by name.
fn parsed_link(raw: &str) -> Link {
    let (kind, target) = raw.split_once(' ').unwrap_or((raw, ""));
    Link {
        kind: kind.to_owned(),
        target: target.to_owned(),
    }
}

/// What closes the Question: the record its answer became, or a reasoned
/// drop.
enum Routing {
    To(String),
    Drop(String),
}

fn chosen_routing(to: Option<String>, drop: Option<String>) -> Result<Routing, NotebookError> {
    match (to, drop) {
        (Some(target), None) => Ok(Routing::To(target)),
        (None, Some(reason)) => Ok(Routing::Drop(reason)),
        (None, None) => Err(NotebookError::InvalidArgument {
            reason: "answer: a routing is required — pass --to <id> or --drop \"<reason>\""
                .to_owned(),
        }),
        (Some(_), Some(_)) => Err(NotebookError::InvalidArgument {
            reason: "answer: pass exactly one of --to, --drop".to_owned(),
        }),
    }
}

fn chosen_proof(
    pr: Option<String>,
    sha: Option<String>,
    report: Option<String>,
    no_proof: bool,
) -> Result<Proof, NotebookError> {
    match (pr, sha, report, no_proof) {
        (Some(target), None, None, false) => Ok(Proof::Pr(target)),
        (None, Some(target), None, false) => Ok(Proof::Sha(target)),
        (None, None, Some(target), false) => Ok(Proof::Report(target)),
        (None, None, None, true) => Ok(Proof::Waived),
        (None, None, None, false) => Err(NotebookError::InvalidArgument {
            reason: "close: a proof is required — pass --pr <url>, --sha <sha>, --report <path>, or --no-proof".to_owned(),
        }),
        _ => Err(NotebookError::InvalidArgument {
            reason: "close: pass exactly one of --pr, --sha, --report, --no-proof".to_owned(),
        }),
    }
}
