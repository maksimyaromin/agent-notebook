//! One command in, one reply out: the dispatch from the parsed surface to
//! the Core, and the few decisions both renderings must make the same way —
//! how many rows a bounded list shows, and how a command line is spelled
//! when a reply names one.

use crate::cli::{AddArgs, CloseArgs, Command, EditArgs, Extent, GraphArgs, Narrowing, Whose};
use crate::setup::{self, SetUp};
use crate::skill::{self, Drift};
use anb_core::encode::{ROW_BOUND, shell_word};
use anb_core::{
    Archived, Budget, CitedProof, Closed, Commented, Created, DebtSignal, Deleted, Draft, Edged,
    Edit, Edited, FileFinding, Filter, Focus, Graph, GraphSlice, Held, Link, ListedRecord,
    Notebook, NotebookError, Proof, ReadyTask, RecordType, Repair, Restored, Scope, Status,
    Storage, StorageError, Transitioned, View, path_stem,
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
/// `--all` on it. A narrowed listing answers a different question from the
/// bare verb, so every narrowing travels with the hint.
#[must_use]
pub fn lifted(verb: &str, filter: &Filter) -> String {
    format!("anb {verb}{} --all", narrowing_flags(filter))
}

/// A filter as the flags that spell it, each with a leading space; a
/// narrowing left out of a hint lifts a different listing than the one the
/// reader is looking at. `--mine` is spelled as the `--by` it stands for,
/// so the hint runs the same for whoever types it.
fn narrowing_flags(filter: &Filter) -> String {
    let mut out = String::new();
    if !filter.types.is_empty() {
        let words: Vec<&str> = filter.types.iter().copied().map(RecordType::word).collect();
        let _ = write!(out, " --type {}", words.join(","));
    }
    if !filter.kinds.is_empty() {
        let _ = write!(out, " --kind {}", filter.kinds.join(","));
    }
    for tag in &filter.tags {
        let _ = write!(out, " --tag {tag}");
    }
    if let Some(hub) = &filter.hub {
        let _ = write!(out, " --for {hub}");
    }
    if let Some(by) = &filter.by {
        let _ = write!(out, " --by {}", shell_word(by));
    }
    if filter.untaken {
        out.push_str(" --untaken");
    }
    if let Some(text) = &filter.text {
        let _ = write!(out, " --match {}", shell_word(text));
    }
    if filter.archive {
        out.push_str(" --archive");
    }
    out
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
        filter: Filter,
        all: bool,
    },
    Listing {
        rows: Vec<ListedRecord>,
        filter: Filter,
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
    Debt {
        signals: Vec<DebtSignal>,
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
    let rendered = skill::anb::render();
    let Some(dir) = dir else {
        return Ok(SkillReply::Printed(rendered.skill));
    };
    let shown = dir.display().to_string();
    if check {
        let drift = skill::drift(dir, &rendered.files())?;
        return Ok(SkillReply::Checked { dir: shown, drift });
    }
    skill::write_into(dir, &rendered.files())?;
    Ok(SkillReply::Written {
        dir: shown,
        files: rendered.files().len(),
    })
}

/// Everything the shell knows and the Core cannot compute, in one place so
/// a new host fact is a field rather than another parameter at every call
/// site.
///
/// `identity` is the accountable identity the host acts as — `ANB_BY`, else
/// git's `user.name` — which the notebook signs with, takes Tasks as, and
/// reads `--mine` by; a command's own `--by` outranks it. `read_file`
/// opens a file by a path the caller typed, or standard input for `-`,
/// which Storage cannot serve: Storage speaks only in paths under the
/// notebook root, and a report or a body is written wherever the work
/// happened. `today` is the host's date — the Core holds no clock.
#[derive(Clone, Copy)]
pub struct Host<'a> {
    pub identity: fn() -> Option<String>,
    /// Borrowed rather than a plain `fn`, because a caller may need to
    /// close over where it reads from — the tests hand in a table of
    /// files, the shell reads the filesystem.
    pub read_file: &'a dyn Fn(&str) -> Result<String, StorageError>,
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
        identity,
        read_file,
        lost_proofs,
        user_notebook,
        project_dir,
        today,
    } = host;
    let identity = identity();
    let mut notebook = Notebook::new(storage)
        .with_user(user_notebook)
        .with_identity(identity.as_deref());
    match command {
        Command::Add(mut args) => {
            let body = body_text(args.body.take(), args.body_file.take(), read_file)?;
            let draft = draft(args, body.unwrap_or_default(), identity.as_deref())?;
            created("add", &mut notebook, &draft, today)
        }
        Command::Retire { id } => Ok(moved("retire", notebook.retire(&id, today)?)),
        Command::Start { id } => Ok(moved("start", notebook.start(&id, today)?)),
        Command::Submit { id } => Ok(moved("submit", notebook.submit(&id, today)?)),
        Command::Close(args) => Ok(Reply::Closed(close_reply(
            &mut notebook,
            args,
            read_file,
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
        Command::Comment { id, text, via } => Ok(Reply::Commented(notebook.comment(
            &id,
            via.as_deref(),
            &text,
            today,
        )?)),
        Command::Ready { narrowing, all } => queued(&notebook, identity.as_deref(), narrowing, all),
        Command::List {
            narrowing,
            extent,
            all,
        } => listed(&notebook, identity.as_deref(), narrowing, extent, all),
        Command::Show { id, all } => Ok(Reply::Viewed {
            view: notebook.view(&id)?,
            all,
        }),
        Command::Check { all } => Ok(Reply::Checked {
            findings: notebook.check()?,
            all,
        }),
        Command::Debt { all } => Ok(Reply::Debt {
            signals: notebook.debt(today, lost_proofs)?,
            all,
        }),
        Command::Archive { id } => Ok(Reply::Archived(notebook.archive(&id, today)?)),
        Command::Restore { id } => Ok(Reply::Restored(notebook.restore(&id)?)),
        Command::Delete { id } => Ok(Reply::Deleted(notebook.delete(&id)?)),
        Command::Edit(args) => edited(&mut notebook, args, read_file, today),
        Command::Graph(args) => graphed(&notebook, identity.as_deref(), args),
        Command::Setup { agents, remove } => {
            Ok(Reply::SetUp(setup::apply(project_dir, remove, &agents)?))
        }
        Command::Skill { dir, check } => Ok(Reply::Skill(skilled(dir.as_deref(), check)?)),
        Command::Status {
            budget,
            hook,
            whose,
        } => status_reply(
            &notebook,
            identity.as_deref(),
            budget,
            whose,
            hook,
            lost_proofs,
            today,
        ),
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
fn graphed(
    notebook: &Notebook<'_>,
    identity: Option<&str>,
    args: GraphArgs,
) -> Result<Reply, NotebookError> {
    let GraphArgs {
        focus,
        depth,
        full,
        all,
        narrowing,
        extent,
    } = args;
    let slice = GraphSlice {
        filter: filter(narrowing, extent, notebook, identity)?,
        focus: focus.map(|id| Focus {
            id,
            depth: depth.unwrap_or(FOCUS_DEPTH),
        }),
    };
    let graph = notebook.graph(&slice)?;
    Ok(Reply::Graphed { graph, full, all })
}

/// The `graph` call that answers a slice, as the caller would type it
/// again: a truncation hint has to name the same slice it cut, or it lifts
/// a different graph.
#[must_use]
pub fn slice_command(slice: &GraphSlice, full: bool) -> String {
    let mut out = format!("anb graph{}", narrowing_flags(&slice.filter));
    if let Some(focus) = &slice.focus {
        let _ = write!(out, " --focus {} --depth {}", focus.id, focus.depth);
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
    identity: Option<&str>,
    budget: Option<u32>,
    whose: Whose,
    hook: bool,
    lost_proofs: &dyn Fn(&[CitedProof]) -> Vec<CitedProof>,
    today: &str,
) -> Result<Reply, NotebookError> {
    // Every failure here — reading the notebook to find the proofs
    // included — passes through the one funnel the hook's fail-soft needs.
    let status = configured_status(notebook, identity, budget, whose, lost_proofs, today);
    match (status, hook) {
        (Ok(status), hook) => Ok(Reply::Status { status, hook }),
        (Err(_), true) => Ok(Reply::Silence),
        (Err(error), false) => Err(error),
    }
}

/// The Status under the resolved ceiling and scope: a flag outranks the
/// config key for each.
fn configured_status(
    notebook: &Notebook<'_>,
    identity: Option<&str>,
    budget: Option<u32>,
    whose: Whose,
    lost_proofs: &dyn Fn(&[CitedProof]) -> Vec<CitedProof>,
    today: &str,
) -> Result<Status, NotebookError> {
    let config = notebook.config()?;
    let ceiling = match budget {
        Some(ceiling) => Budget::from_ceiling(ceiling),
        None => config.budget(),
    };
    let by = named(whose, config.scope(), identity)?;
    notebook.status(today, ceiling, by.as_deref(), lost_proofs)
}

fn edited(
    notebook: &mut Notebook<'_>,
    args: EditArgs,
    read_file: &dyn Fn(&str) -> Result<String, StorageError>,
    today: &str,
) -> Result<Reply, NotebookError> {
    let EditArgs {
        id,
        title,
        body,
        body_file,
        add_tags,
        remove_tags,
        add_links,
        remove_links,
        from,
        priority,
        review_by,
        taken_by,
        clear,
    } = args;
    let edit = Edit {
        title,
        body: body_text(body, body_file, read_file)?,
        add_tags,
        remove_tags,
        add_links: add_links.iter().map(|raw| parsed_link(raw)).collect(),
        remove_links: remove_links.iter().map(|raw| parsed_link(raw)).collect(),
        from,
        priority,
        review_by,
        taken_by,
        clear,
    };
    Ok(Reply::Edited(notebook.edit(&id, &edit, today)?))
}

/// The body a command carries: the text on the command line, or the text
/// of the file `--body-file` names, read by the host. Clap refuses the two
/// flags together, so the text wins where both arrive.
fn body_text(
    inline: Option<String>,
    file: Option<String>,
    read_file: &dyn Fn(&str) -> Result<String, StorageError>,
) -> Result<Option<String>, NotebookError> {
    match (inline, file) {
        (Some(text), _) => Ok(Some(text)),
        (None, Some(path)) => read_file(&path)
            .map(Some)
            .map_err(|error| file_refusal("body-file", &error)),
        (None, None) => Ok(None),
    }
}

/// The draft as the Core takes it. `--mine` is `--taken-by` with the
/// identity the host acts as, so a host that knows nobody has nobody to
/// take the task for and says so.
fn draft(args: AddArgs, body: String, identity: Option<&str>) -> Result<Draft, NotebookError> {
    let mut draft = Draft::new(args.record_type, &args.title);
    draft.id = args.id;
    draft.by = args.by;
    draft.via = args.via;
    draft.taken_by = if args.mine {
        Some(own_name(
            identity,
            "mine: no identity to take the task for",
            "",
        )?)
    } else {
        args.taken_by
    };
    draft.from = args.from;
    draft.tags = args.tags;
    draft.links = args.links.iter().map(|raw| parsed_link(raw)).collect();
    draft.body = body;
    draft.priority = args.priority;
    draft.kind = args.kind;
    draft.supersedes = args.supersedes;
    Ok(draft)
}

/// The identity a call stands in for, or the refusal that names how a
/// host that knows nobody gets one, and the `way_around` when the call has
/// one.
fn own_name(
    identity: Option<&str>,
    lacking: &str,
    way_around: &str,
) -> Result<String, NotebookError> {
    identity
        .map(str::to_owned)
        .ok_or_else(|| NotebookError::InvalidArgument {
            reason: format!(
                "{lacking}; set git user.name or {}{way_around}",
                crate::identity::IDENTITY_ENV
            ),
        })
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

/// The dispatch queue, narrowed as the caller asked; a queue reaches
/// live open Tasks alone, so it has no extent to widen.
fn queued(
    notebook: &Notebook<'_>,
    identity: Option<&str>,
    narrowing: Narrowing,
    all: bool,
) -> Result<Reply, NotebookError> {
    let filter = filter(narrowing, Extent::default(), notebook, identity)?;
    Ok(Reply::Ready {
        rows: notebook.ready(&filter)?,
        filter,
        all,
    })
}

/// The listing, narrowed as the caller asked.
fn listed(
    notebook: &Notebook<'_>,
    identity: Option<&str>,
    narrowing: Narrowing,
    extent: Extent,
    all: bool,
) -> Result<Reply, NotebookError> {
    let filter = filter(narrowing, extent, notebook, identity)?;
    Ok(Reply::Listing {
        rows: notebook.list(&filter)?,
        filter,
        all,
    })
}

/// The command line's narrowing as the Core reads it, whose records it
/// answers with settled against the notebook's `scope` key.
fn filter(
    narrowing: Narrowing,
    extent: Extent,
    notebook: &Notebook<'_>,
    identity: Option<&str>,
) -> Result<Filter, NotebookError> {
    let Narrowing {
        whose,
        untaken,
        scope,
        tags,
        text,
    } = narrowing;
    let Extent {
        types,
        kinds,
        archive,
    } = extent;
    // The pool is nobody's by definition, so asking for it answers the
    // whose question outright and leaves the config key aside.
    let by = if untaken {
        None
    } else {
        named(whose, notebook.config()?.scope(), identity)?
    };
    Ok(Filter {
        types,
        kinds,
        tags,
        hub: scope,
        by,
        untaken,
        text,
        archive,
    })
}

/// The one identity a read is narrowed to, or nobody. A flag outranks the
/// config key; `--mine`, and a `scope: mine` the call does not widen, is
/// `--by` with the identity the host acts as, so a host that knows nobody
/// has nothing to narrow by and says so, rather than answering an empty
/// list that reads as "nothing is yours".
fn named(
    whose: Whose,
    scope: Scope,
    identity: Option<&str>,
) -> Result<Option<String>, NotebookError> {
    let Whose { by, mine, team } = whose;
    let own = |lacking: &str| own_name(identity, lacking, ", or pass --team");
    if let Some(by) = by {
        return Ok(Some(by));
    }
    if mine {
        return own("mine: no identity to match").map(Some);
    }
    if team {
        return Ok(None);
    }
    match scope {
        Scope::Team => Ok(None),
        Scope::Mine => own("scope: mine needs an identity").map(Some),
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
    read_file: &dyn Fn(&str) -> Result<String, StorageError>,
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
            let report = read_file(&path).map_err(|error| file_refusal("note", &error))?;
            notebook.close_with_report(&id, &report, today)
        }
        Closing::Stored(proof) => notebook.close(&id, &proof, today),
        Closing::Reason(reason) => notebook.close_with_reason(&id, &reason, today),
        Closing::ResolvedBy(resolver) => notebook.resolve_question(&id, &resolver, today),
    }
}

/// The close flags as one phrase, so the two refusals name the same set.
const CLOSE_FLAGS: &str = "--note <path>, --pr <url>, --sha <sha>, --report <path>, --no-proof, --reason \"<why>\", or --resolved-by <id>";

/// A file the caller named under `flag` and the shell could not read. The
/// path came off the command line, so every way it can fail is a refused
/// argument the caller retypes — never the storage failure this error type
/// carries when it is the notebook itself that could not be read.
fn file_refusal(flag: &str, error: &StorageError) -> NotebookError {
    let reason = match error {
        StorageError::NotFound { path } => format!("{flag}: no file at `{path}`"),
        StorageError::NotUtf8 { path } => format!("{flag}: `{path}` is not UTF-8"),
        StorageError::Io { path, detail } => format!("{flag}: cannot read `{path}` — {detail}"),
    };
    NotebookError::InvalidArgument { reason }
}
