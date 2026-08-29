//! One command in, one reply out: the dispatch from the parsed surface to
//! the Core, with nothing rendered yet — both output formats read the same
//! reply.

use crate::cli::{AddArgs, CloseArgs, Command, DecideArgs, DraftArgs, EditArgs, NoteArgs, Subject};
use anb_core::encode::ROW_BOUND;
use anb_core::notebook::path_stem;
use anb_core::{
    Archived, Budget, CitedProof, Closed, Commented, Created, Draft, Dropped, Edged, Edit, Edited,
    Expunged, FileFinding, Finding, Held, Link, ListedRecord, Notebook, NotebookError, Overview,
    Proof, ReadyTask, RecordType, Status, Storage, StorageError, Transitioned, View,
};

/// The first bounded prefix of a flat list; both renderers show the same
/// rows. `--all` is the one lift, and only a listing offers it.
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
    /// A Question routed into what its answer became.
    Routed {
        transition: Transitioned,
        to: String,
    },
    /// A Question closed without routing, for its stated reason.
    Dropped(Dropped),
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
    Checked {
        findings: Vec<FileFinding>,
        all: bool,
    },
    Archived(Archived),
    Expunged(Expunged),
    Edited(Edited),
    Searched {
        query: String,
        rows: Vec<ListedRecord>,
        all: bool,
    },
    Overviewed {
        overview: Overview,
        all: bool,
    },
    /// The hook's fail-soft outcome: no context rather than a blocked
    /// session.
    Silence,
}

impl Reply {
    /// Whether the command's outcome is a failing exit despite a rendered
    /// report: a check that found error findings gates a caller like CI.
    #[must_use]
    pub fn failed(&self) -> bool {
        match self {
            Reply::Checked { findings, .. } => findings
                .iter()
                .any(|located| located.finding.code.severity() == anb_core::Severity::Error),
            _ => false,
        }
    }
}

/// Everything the shell knows and the Core cannot compute, in one place so
/// a new host fact is a field rather than another parameter at every call
/// site.
///
/// `git_by` is the accountable identity as git knows it, read only by the
/// commands that write one; a command's own flag outranks it. `read_report`
/// opens a file by a path the caller typed, which Storage cannot serve:
/// Storage speaks only in paths under the notebook root, and a report is
/// written wherever the work happened. `today` is the host's date — the
/// Core holds no clock.
pub struct Host<'a, G, R, C> {
    pub git_by: G,
    pub read_report: R,
    /// Which of the proofs a notebook cites the world no longer holds. The
    /// Core holds neither git nor a filesystem, so the question is asked
    /// out here; a caller with nothing to ask answers with an empty list.
    pub lost_proofs: C,
    pub today: &'a str,
}

/// Run `command` against the notebook behind `storage`.
///
/// # Errors
/// The Core's refusal, or the shell's own argument refusal — either
/// renders as a recovery payload.
pub fn execute<S, G, R, C>(
    command: Command,
    storage: &mut S,
    host: Host<'_, G, R, C>,
) -> Result<Reply, NotebookError>
where
    S: Storage,
    G: FnOnce() -> Option<String>,
    R: FnOnce(&str) -> Result<String, StorageError>,
    C: FnOnce(&[CitedProof]) -> Vec<CitedProof>,
{
    let Host {
        git_by,
        read_report,
        lost_proofs,
        today,
    } = host;
    let mut notebook = Notebook::new(storage);
    match command {
        Command::Add(args) => created("add", &mut notebook, &task_draft(args, git_by), today),
        Command::Decide(args) => created(
            "decide",
            &mut notebook,
            &decision_draft(args, git_by),
            today,
        ),
        Command::Note(args) => created("note", &mut notebook, &note_draft(args, git_by), today),
        Command::Ask(args) => {
            let draft = draft(RecordType::Question, args, git_by);
            created("ask", &mut notebook, &draft, today)
        }
        Command::Answer { id, to, drop } => answered(&mut notebook, &id, to, drop, today),
        Command::Retire { id } => Ok(moved("retire", notebook.retire(&id, today)?)),
        Command::Start { id } => Ok(moved("start", notebook.start(&id, today)?)),
        Command::Submit { id } => Ok(moved("submit", notebook.submit(&id, today)?)),
        Command::Close(args) => Ok(Reply::Closed(close_reply(
            &mut notebook,
            args,
            read_report,
            git_by,
            today,
        )?)),
        Command::Return { id } => Ok(moved("return", notebook.return_task(&id, today)?)),
        Command::Reopen { id } => Ok(moved("reopen", notebook.reopen(&id, today)?)),
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
        Command::Ready { scope, all } => Ok(Reply::Ready {
            rows: queued(&notebook, scope.as_deref())?,
            all,
        }),
        Command::List { scope, all } => Ok(Reply::Listing {
            rows: listed(&notebook, scope.as_deref())?,
            all,
        }),
        Command::View { id } => Ok(Reply::Viewed(notebook.view(&id)?)),
        Command::Check { all } => Ok(Reply::Checked {
            findings: notebook.check()?,
            all,
        }),
        Command::Archive { id } => Ok(Reply::Archived(notebook.archive(&id)?)),
        Command::Expunge { id } => Ok(Reply::Expunged(notebook.expunge(&id)?)),
        Command::Edit(args) => edited(&mut notebook, args, today),
        Command::Search { query, all } => Ok(Reply::Searched {
            rows: notebook.search(&query)?,
            query,
            all,
        }),
        Command::Overview { all } => Ok(Reply::Overviewed {
            overview: notebook.overview()?,
            all,
        }),
        Command::Status { budget, hook } => {
            status_reply(&notebook, budget, hook, lost_proofs, today)
        }
    }
}

/// A record minted under the verb that asked for it.
fn created<S: Storage>(
    command: &'static str,
    notebook: &mut Notebook<'_, S>,
    draft: &Draft,
    today: &str,
) -> Result<Reply, NotebookError> {
    Ok(Reply::Created {
        command,
        created: notebook.create(draft, today)?,
    })
}

/// A state move under the verb that made it.
fn moved(command: &'static str, transition: Transitioned) -> Reply {
    Reply::Moved {
        command,
        transition,
    }
}

/// The Status, or the hook's fail-soft outcome: an empty context, never a
/// blocked session.
fn status_reply<S: Storage>(
    notebook: &Notebook<'_, S>,
    budget: Option<u32>,
    hook: bool,
    lost_proofs: impl FnOnce(&[CitedProof]) -> Vec<CitedProof>,
    today: &str,
) -> Result<Reply, NotebookError> {
    // Every failure here — reading the notebook to find the proofs
    // included — passes through the one funnel the hook's fail-soft needs.
    match (budgeted_status(notebook, budget, lost_proofs, today), hook) {
        (Ok(status), hook) => Ok(Reply::Status { status, hook }),
        (Err(_), true) => Ok(Reply::Silence),
        (Err(error), false) => Err(error),
    }
}

/// The Status under the resolved ceiling: the flag outranks the config key.
fn budgeted_status<S: Storage>(
    notebook: &Notebook<'_, S>,
    budget: Option<u32>,
    lost_proofs: impl FnOnce(&[CitedProof]) -> Vec<CitedProof>,
    today: &str,
) -> Result<Status, NotebookError> {
    let ceiling = match budget {
        Some(ceiling) => Budget::from_ceiling(ceiling),
        None => notebook.config()?.budget(),
    };
    notebook.status(today, ceiling, lost_proofs)
}

fn edited<S: Storage>(
    notebook: &mut Notebook<'_, S>,
    args: EditArgs,
    today: &str,
) -> Result<Reply, NotebookError> {
    let EditArgs {
        id,
        title,
        body,
        add_tags,
        remove_tags,
        from,
        priority,
        review_by,
    } = args;
    let edit = Edit {
        title,
        body,
        add_tags,
        remove_tags,
        from,
        priority,
        review_by,
    };
    Ok(Reply::Edited(notebook.edit(&id, &edit, today)?))
}

fn answered<S: Storage>(
    notebook: &mut Notebook<'_, S>,
    id: &str,
    to: Option<String>,
    drop: Option<String>,
    today: &str,
) -> Result<Reply, NotebookError> {
    match chosen_routing(to, drop)? {
        Routing::To(target) => Ok(Reply::Routed {
            transition: notebook.route(id, &target, today)?,
            to: target,
        }),
        Routing::Drop(reason) => Ok(Reply::Dropped(notebook.drop_question(id, &reason, today)?)),
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
                recovery.details = bounded(findings.iter().map(finding_line).collect());
                recovery.tries.push(format!("anb view {}", path_stem(path)));
            }
            NotebookError::InvalidTransition { id, valid, .. } => {
                for action in valid {
                    recovery.tries.extend(transition_retries(action, id));
                }
            }
            NotebookError::StillReferenced { blockers, .. } => {
                recovery.details = bounded(blockers.iter().map(ToString::to_string).collect());
                recovery.tries.extend(
                    anb_core::carriers_of(blockers)
                        .take(ROW_BOUND)
                        .map(|carrier| format!("anb view {carrier}")),
                );
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

/// A valid command as its runnable shape: the verbs whose bare form clap
/// would refuse carry their required flag as a placeholder.
fn transition_retries(action: &str, id: &str) -> Vec<String> {
    match action {
        "close" => vec![format!("anb close {id} --note <path>")],
        "answer" => vec![
            format!("anb answer {id} --to <id>"),
            format!("anb answer {id} --drop \"<why>\""),
        ],
        action => vec![format!("anb {action} {id}")],
    }
}

/// The retry a refused argument points at, keyed by the verb it refused;
/// the create verbs carry no id and retry as a command shape.
fn argument_retries(subject: &Subject) -> Vec<String> {
    match (subject.verb, &subject.id) {
        ("close", Some(id)) => vec![
            format!("anb close {id} --note <path>"),
            format!("anb close {id} --no-proof"),
        ],
        ("hold", Some(id)) => vec![format!("anb hold {id} --reason \"<why>\"")],
        ("comment", Some(id)) => vec![format!("anb comment {id} \"<one line>\"")],
        ("answer", Some(id)) => vec![
            format!("anb answer {id} --to <id>"),
            format!("anb answer {id} --drop \"<why>\""),
        ],
        ("edit", Some(id)) => vec![format!("anb edit {id} --title \"<title>\"")],
        ("add", _) => vec!["anb add \"<title>\"".to_owned()],
        ("decide", _) => vec!["anb decide \"<title>\" --kind rule".to_owned()],
        ("note", _) => vec!["anb note \"<title>\" --kind fact".to_owned()],
        ("ask", _) => vec!["anb ask \"<title>\"".to_owned()],
        ("search", _) => vec!["anb search \"<text>\"".to_owned()],
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
        NotebookError::StillReferenced { .. } => "still-referenced",
        NotebookError::DanglingRef { .. } => "dangling-ref",
        NotebookError::CannotSupersede { .. } => "cannot-supersede",
        NotebookError::WouldCycle { .. } => "would-cycle",
        // The command-level code and the Check finding share one vocabulary.
        NotebookError::Storage(StorageError::NotUtf8 { .. }) => "not-utf8",
        NotebookError::Storage(_) => "storage",
    }
}

fn finding_line(finding: &Finding) -> String {
    match finding.line {
        Some(line) => format!("line {line}: {} {}", finding.code.as_str(), finding.message),
        None => format!("{} {}", finding.code.as_str(), finding.message),
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

/// The dispatch queue, whole or narrowed to one epic.
fn queued<S: Storage>(
    notebook: &Notebook<'_, S>,
    scope: Option<&str>,
) -> Result<Vec<ReadyTask>, NotebookError> {
    match scope {
        Some(hub) => notebook.ready_for(hub),
        None => notebook.ready(),
    }
}

/// The live listing, whole or narrowed to one epic.
fn listed<S: Storage>(
    notebook: &Notebook<'_, S>,
    scope: Option<&str>,
) -> Result<Vec<ListedRecord>, NotebookError> {
    match scope {
        Some(hub) => notebook.list_for(hub),
        None => notebook.list(),
    }
}

/// The one proof a close was given. `--note` names a file the shell must
/// read, since only the host can reach a path outside the notebook; every
/// other proof is a string the Core stores as given.
enum ChosenProof {
    Ingest(String),
    Stored(Proof),
}

/// The single proof among the flags, or the refusal that says which way the
/// caller missed: nothing offered, or more than one.
fn chosen_proof(args: &mut CloseArgs) -> Result<ChosenProof, NotebookError> {
    let mut offered: Vec<ChosenProof> = Vec::new();
    if let Some(path) = args.note.take() {
        offered.push(ChosenProof::Ingest(path));
    }
    for stored in [
        args.pr.take().map(Proof::Pr),
        args.sha.take().map(Proof::Sha),
        args.report.take().map(Proof::Report),
        args.no_proof.then_some(Proof::Waived),
    ]
    .into_iter()
    .flatten()
    {
        offered.push(ChosenProof::Stored(stored));
    }

    let mut offered = offered.into_iter();
    match (offered.next(), offered.next()) {
        (Some(only), None) => Ok(only),
        (None, _) => Err(NotebookError::InvalidArgument {
            reason: format!("close: a proof is required — pass {PROOF_FLAGS}"),
        }),
        (Some(_), Some(_)) => Err(NotebookError::InvalidArgument {
            reason: format!("close: pass exactly one of {PROOF_FLAGS}"),
        }),
    }
}

fn close_reply<S: Storage>(
    notebook: &mut Notebook<'_, S>,
    mut args: CloseArgs,
    read_report: impl FnOnce(&str) -> Result<String, StorageError>,
    git_by: impl FnOnce() -> Option<String>,
    today: &str,
) -> Result<Closed, NotebookError> {
    match chosen_proof(&mut args)? {
        ChosenProof::Ingest(path) => {
            let report = read_report(&path).map_err(|error| report_refusal(&error))?;
            notebook.close_with_report(&args.id, &report, git_by().as_deref(), today)
        }
        ChosenProof::Stored(proof) => notebook.close(&args.id, &proof, today),
    }
}

/// The proof flags as one phrase, so the two refusals name the same set.
const PROOF_FLAGS: &str = "--note <path>, --pr <url>, --sha <sha>, --report <path>, or --no-proof";

/// A report the caller named and the shell could not read. The path came
/// off the command line, so every way it can fail is a refused argument the
/// caller retypes — never the storage failure this error type carries when
/// it is the notebook itself that could not be read.
fn report_refusal(error: &StorageError) -> NotebookError {
    let reason = match error {
        StorageError::NotFound { path } => format!("note: no file at `{path}`"),
        StorageError::NotUtf8 { path } => format!("note: `{path}` is not UTF-8"),
        StorageError::Io { path, detail } => format!("note: cannot read `{path}` — {detail}"),
    };
    NotebookError::InvalidArgument { reason }
}
