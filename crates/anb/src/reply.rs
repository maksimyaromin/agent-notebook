//! One command in, one reply out: the dispatch from the parsed surface to
//! the Core, and the few decisions both renderings must make the same way —
//! how many rows a bounded list shows, and how a command line is spelled
//! when a reply names one.

use crate::cli::{AddArgs, CloseArgs, Command, EditArgs, GraphArgs, SliceArgs};
use crate::setup::{self, SetUp};
use crate::skill::{self, Drift};
use anb_core::encode::ROW_BOUND;
use anb_core::{
    Archived, Budget, CitedProof, Closed, Commented, Created, Deleted, Draft, Edged, Edit, Edited,
    FileFinding, Focus, Graph, GraphSlice, Held, Link, ListedRecord, Notebook, NotebookError,
    Overview, Proof, ReadyTask, RecordType, Repair, Restored, Status, Storage, StorageError,
    Transitioned, View, path_stem,
};
use std::fmt::Write as _;
use std::path::Path;

/// The first bounded prefix of a flat list; both renderers show the same
/// rows, and `--all` is the one lift.
#[must_use]
pub fn shown(total: usize, all: bool) -> usize {
    if all { total } else { total.min(ROW_BOUND) }
}

/// The command that lifts a bounded listing: the one the caller ran, with
/// `--all` on it. A scoped listing answers a different question from the
/// bare verb, so the scope travels with the hint.
#[must_use]
pub fn lifted(verb: &str, scope: Option<&str>) -> String {
    match scope {
        Some(scope) => format!("anb {verb} --for {scope} --all"),
        None => format!("anb {verb} --all"),
    }
}

/// The repair a finding names, as the command that runs it. A finding is
/// located by file, and the id a verb takes is that file's stem.
#[must_use]
pub fn repair_command(repair: &Repair, path: &str) -> String {
    let id = path_stem(path);
    match repair {
        Repair::Clear(field) => format!("anb edit {id} --clear {field}"),
        Repair::Unblock(on) => format!("anb unblock {id} {on}"),
        Repair::Unhold => format!("anb unhold {id}"),
        Repair::Archive => format!("anb archive {id}"),
        Repair::Restore => format!("anb restore {id}"),
    }
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
        scope: Option<String>,
        all: bool,
    },
    Listing {
        rows: Vec<ListedRecord>,
        scope: Option<String>,
        all: bool,
    },
    Viewed {
        view: View,
        all: bool,
    },
    Status {
        status: Status,
        hook: bool,
    },
    Checked {
        findings: Vec<FileFinding>,
        all: bool,
    },
    Archived(Archived),
    Restored(Restored),
    Deleted(Deleted),
    SetUp(SetUp),
    /// The skill rendered from the binary: printed, written, or held
    /// against a committed copy.
    Skill(SkillReply),
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
    /// The graph itself: the records and the edges between them.
    Graphed {
        graph: Graph,
        full: bool,
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
            Reply::Checked { findings, .. } => {
                findings.iter().any(|located| located.finding.is_error())
            }
            Reply::Skill(SkillReply::Checked { drift, .. }) => !drift.is_empty(),
            _ => false,
        }
    }
}

/// What `skill` answered with.
#[derive(Debug, PartialEq, Eq)]
pub enum SkillReply {
    /// `SKILL.md` itself, for a reader with no directory to write into.
    Printed(String),
    Written {
        dir: String,
        files: usize,
    },
    /// The committed copy held against the rendering; `drift` empty means
    /// the two agree.
    Checked {
        dir: String,
        drift: Vec<Drift>,
    },
}

fn skilled(dir: Option<&Path>, check: bool) -> Result<SkillReply, NotebookError> {
    let rendered = skill::render();
    let Some(dir) = dir else {
        return Ok(SkillReply::Printed(rendered.skill));
    };
    let shown = dir.display().to_string();
    if check {
        let drift = skill::drift(dir, &rendered)?;
        return Ok(SkillReply::Checked { dir: shown, drift });
    }
    skill::write_into(dir, &rendered)?;
    Ok(SkillReply::Written {
        dir: shown,
        files: rendered.files().len(),
    })
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
#[derive(Clone, Copy)]
pub struct Host<'a> {
    pub git_by: fn() -> Option<String>,
    /// Borrowed rather than a plain `fn`, because a caller may need to
    /// close over where it reads from — the tests hand in a table of
    /// reports, the shell reads the filesystem.
    pub read_report: &'a dyn Fn(&str) -> Result<String, StorageError>,
    /// Which of the proofs a notebook cites the world no longer holds. The
    /// Core holds neither git nor a filesystem, so the question is asked
    /// out here; a caller with nothing to ask answers with an empty list.
    pub lost_proofs: &'a dyn Fn(&[CitedProof]) -> Vec<CitedProof>,
    /// The user's notebook, when the host resolved a usable one. It stands
    /// behind every surface of the project's: Status pairs a project rule
    /// with the standing rule it shadows, a write's nudge names no id it
    /// holds as dangling, and `check` lets a link reach it. `None` names no
    /// such root, never an empty one.
    pub user_notebook: Option<&'a dyn Storage>,
    /// Where the session starts: the directory `setup` writes the agents'
    /// files into.
    pub project_dir: &'a Path,
    pub today: &'a str,
}

/// Run `command` against the notebook behind `storage`.
///
/// # Errors
/// The Core's refusal, or the shell's own argument refusal — either
/// renders as a recovery payload.
pub fn execute(
    command: Command,
    storage: &mut dyn Storage,
    host: Host<'_>,
) -> Result<Reply, NotebookError> {
    let Host {
        git_by,
        read_report,
        lost_proofs,
        user_notebook,
        project_dir,
        today,
    } = host;
    let mut notebook = Notebook::new(storage).with_user(user_notebook);
    match command {
        Command::Add(args) => created("add", &mut notebook, &draft(args, git_by), today),
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
            scope,
            all,
        }),
        Command::List { scope, all } => Ok(Reply::Listing {
            rows: listed(&notebook, scope.as_deref())?,
            scope,
            all,
        }),
        Command::Show { id, all } => Ok(Reply::Viewed {
            view: notebook.view(&id)?,
            all,
        }),
        Command::Check { all } => Ok(Reply::Checked {
            findings: notebook.check()?,
            all,
        }),
        Command::Archive { id } => Ok(Reply::Archived(notebook.archive(&id, today)?)),
        Command::Restore { id } => Ok(Reply::Restored(notebook.restore(&id)?)),
        Command::Delete { id } => Ok(Reply::Deleted(notebook.delete(&id)?)),
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
        Command::Graph(args) => graphed(&notebook, args),
        Command::Setup { remove } => Ok(Reply::SetUp(setup::apply(project_dir, remove)?)),
        Command::Skill { dir, check } => Ok(Reply::Skill(skilled(dir.as_deref(), check)?)),
        Command::Status { budget, hook } => {
            status_reply(&notebook, budget, hook, lost_proofs, today)
        }
    }
}

/// A record minted under the verb that asked for it.
fn created(
    command: &'static str,
    notebook: &mut Notebook<'_>,
    draft: &Draft,
    today: &str,
) -> Result<Reply, NotebookError> {
    Ok(Reply::Created {
        command,
        created: notebook.create(draft, today)?,
    })
}

/// How far around a focus a graph reaches when the caller names no depth:
/// the record and what touches it. A focus asks that before it asks
/// anything wider.
const FOCUS_DEPTH: usize = 1;

/// The graph the caller asked for, as the records and the edges between
/// them. Drawing is nobody's business here: a reader who wants a picture
/// has an agent that builds one, and it can only do that from a graph that
/// arrived whole.
fn graphed(notebook: &Notebook<'_>, args: GraphArgs) -> Result<Reply, NotebookError> {
    let GraphArgs { slice, full, all } = args;
    let graph = notebook.graph(&asked_for(slice))?;
    Ok(Reply::Graphed { graph, full, all })
}

/// The command line's slice as the Core reads it.
fn asked_for(args: SliceArgs) -> GraphSlice {
    GraphSlice {
        types: args.types,
        hub: args.scope,
        ready_only: args.ready,
        focus: args.focus.map(|id| Focus {
            id,
            depth: args.depth.unwrap_or(FOCUS_DEPTH),
        }),
        archive: args.archive,
    }
}

/// The `graph` call that answers a slice, as the caller would type it
/// again: a truncation hint has to name the same slice it cut, or it lifts
/// a different graph.
#[must_use]
pub fn slice_command(slice: &GraphSlice, full: bool) -> String {
    let mut out = "anb graph".to_owned();
    if !slice.types.is_empty() {
        let words: Vec<&str> = slice.types.iter().copied().map(RecordType::word).collect();
        let _ = write!(out, " --type {}", words.join(","));
    }
    if let Some(hub) = &slice.hub {
        let _ = write!(out, " --for {hub}");
    }
    if slice.ready_only {
        out.push_str(" --ready");
    }
    if let Some(focus) = &slice.focus {
        let _ = write!(out, " --focus {} --depth {}", focus.id, focus.depth);
    }
    if slice.archive {
        out.push_str(" --archive");
    }
    if full {
        out.push_str(" --full");
    }
    out
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
fn status_reply(
    notebook: &Notebook<'_>,
    budget: Option<u32>,
    hook: bool,
    lost_proofs: &dyn Fn(&[CitedProof]) -> Vec<CitedProof>,
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
fn budgeted_status(
    notebook: &Notebook<'_>,
    budget: Option<u32>,
    lost_proofs: &dyn Fn(&[CitedProof]) -> Vec<CitedProof>,
    today: &str,
) -> Result<Status, NotebookError> {
    let ceiling = match budget {
        Some(ceiling) => Budget::from_ceiling(ceiling),
        None => notebook.config()?.budget(),
    };
    notebook.status(today, ceiling, lost_proofs)
}

fn edited(
    notebook: &mut Notebook<'_>,
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
        clear,
    } = args;
    let edit = Edit {
        title,
        body,
        add_tags,
        remove_tags,
        from,
        priority,
        review_by,
        clear,
    };
    Ok(Reply::Edited(notebook.edit(&id, &edit, today)?))
}

fn draft(args: AddArgs, git_by: impl FnOnce() -> Option<String>) -> Draft {
    let mut draft = Draft::new(args.record_type, &args.title);
    draft.id = args.id;
    draft.by = args.by.or_else(git_by);
    draft.via = args.via;
    draft.from = args.from;
    draft.tags = args.tags;
    draft.links = args.links.iter().map(|raw| parsed_link(raw)).collect();
    draft.body = args.body.unwrap_or_default();
    draft.priority = args.priority;
    draft.kind = args.kind;
    draft.supersedes = args.supersedes;
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

/// The dispatch queue, whole or narrowed to one epic.
fn queued(notebook: &Notebook<'_>, scope: Option<&str>) -> Result<Vec<ReadyTask>, NotebookError> {
    match scope {
        Some(hub) => notebook.ready_for(hub),
        None => notebook.ready(),
    }
}

/// The live listing, whole or narrowed to one epic.
fn listed(
    notebook: &Notebook<'_>,
    scope: Option<&str>,
) -> Result<Vec<ListedRecord>, NotebookError> {
    match scope {
        Some(hub) => notebook.list_for(hub),
        None => notebook.list(),
    }
}

/// The one way a close was told to end the record. `--note` names a file
/// the shell must read, since only the host can reach a path outside the
/// notebook; every other proof is a string the Core stores as given; a
/// reason ends a Task or a Question without a proof; a resolver is the
/// record a Question closed into.
enum Closing {
    Ingest(String),
    Stored(Proof),
    Reason(String),
    ResolvedBy(String),
}

/// The single closing among the flags, or the refusal that says which way
/// the caller missed: nothing offered, or more than one.
fn chosen_closing(offered: [Option<Closing>; 7]) -> Result<Closing, NotebookError> {
    let mut offered = offered.into_iter().flatten();
    match (offered.next(), offered.next()) {
        (Some(only), None) => Ok(only),
        (None, _) => Err(NotebookError::InvalidArgument {
            reason: format!("close: pass one of {CLOSE_FLAGS}"),
        }),
        (Some(_), Some(_)) => Err(NotebookError::InvalidArgument {
            reason: format!("close: pass exactly one of {CLOSE_FLAGS}"),
        }),
    }
}

fn close_reply(
    notebook: &mut Notebook<'_>,
    args: CloseArgs,
    read_report: &dyn Fn(&str) -> Result<String, StorageError>,
    git_by: impl FnOnce() -> Option<String>,
    today: &str,
) -> Result<Closed, NotebookError> {
    let CloseArgs {
        id,
        note,
        pr,
        sha,
        report,
        no_proof,
        reason,
        resolved_by,
    } = args;
    // Each flag builds its own answer, so no two can be transposed.
    match chosen_closing([
        note.map(Closing::Ingest),
        pr.map(|url| Closing::Stored(Proof::Pr(url))),
        sha.map(|sha| Closing::Stored(Proof::Sha(sha))),
        report.map(|path| Closing::Stored(Proof::Report(path))),
        no_proof.then_some(Closing::Stored(Proof::Waived)),
        reason.map(Closing::Reason),
        resolved_by.map(Closing::ResolvedBy),
    ])? {
        Closing::Ingest(path) => {
            let report = read_report(&path).map_err(|error| report_refusal(&error))?;
            notebook.close_with_report(&id, &report, git_by().as_deref(), today)
        }
        Closing::Stored(proof) => notebook.close(&id, &proof, today),
        Closing::Reason(reason) => notebook.close_with_reason(&id, &reason, today),
        Closing::ResolvedBy(resolver) => notebook.resolve_question(&id, &resolver, today),
    }
}

/// The close flags as one phrase, so the two refusals name the same set.
const CLOSE_FLAGS: &str = "--note <path>, --pr <url>, --sha <sha>, --report <path>, --no-proof, --reason \"<why>\", or --resolved-by <id>";

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
