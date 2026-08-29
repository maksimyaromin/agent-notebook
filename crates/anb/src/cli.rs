//! The command surface: conventional tracker verbs over the notebook.
//!
//! Flags with domain semantics stay optional here and are judged by the
//! Core, so their refusals arrive as structured recovery payloads; clap
//! keeps only the structural surface — positionals and flag spelling.

use crate::reply::Recovery;
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "anb",
    version,
    about = "A project's working memory as typed records in the repository"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    /// Compact JSON instead of plain text, on every command.
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a Task; the notebook appears on first write.
    Add(AddArgs),
    /// open → active: take the Task into work.
    Start { id: String },
    /// active → review: hand the work to a human for acceptance.
    Submit { id: String },
    /// active | review → closed, carrying its proof.
    Close(CloseArgs),
    /// review → active: the human returned the work.
    Return { id: String },
    /// closed → open, explicitly.
    Reopen { id: String },
    /// Pause a Task deliberately; the reason is mandatory.
    Hold {
        id: String,
        /// Why the Task waits; an unreasoned hold is where work rots.
        #[arg(long)]
        reason: Option<String>,
        /// Calendar hold: the date to resume on.
        #[arg(long)]
        until: Option<String>,
    },
    /// Resume a held Task.
    Unhold { id: String },
    /// Write a dependency edge: this Task waits on another.
    Block { id: String, on: String },
    /// Erase a dependency edge.
    Unblock { id: String, on: String },
    /// Append one entry to a Task's log — where the next session resumes.
    Comment {
        id: String,
        text: String,
        /// The acting agent tool writing the entry.
        #[arg(long)]
        via: Option<String>,
    },
    /// Record a Decision; replacing a conflicting one takes --supersedes.
    Decide(DecideArgs),
    /// Record a Note: curated knowledge, corrected in place.
    Note(NoteArgs),
    /// File a Question: a doubt parked without scope creep.
    Ask(DraftArgs),
    /// Close a Question by routing it into what its answer became.
    Answer {
        id: String,
        /// The Decision or Task the answer became.
        #[arg(long)]
        to: Option<String>,
        /// Close without routing, stating why.
        #[arg(long)]
        drop: Option<String>,
    },
    /// active → retired: end a Decision or Note that has no successor.
    Retire { id: String },
    /// The dispatch queue: open, unblocked, unheld Tasks, most urgent first.
    Ready {
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// Every live record.
    List {
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// One record whole: envelope, body, and its mention blocks.
    View { id: String },
    /// The session Status: one quiet line, or the budgeted composite.
    Status {
        /// Token ceiling for this call, outranking the config key; 0 = no ceiling.
        #[arg(long)]
        budget: Option<u32>,
        /// The session-start payload for an agent hook; fails soft.
        #[arg(long)]
        hook: bool,
    },
    /// Verify every file: each finding names file, line, and reason.
    Check {
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// Move a settled record into the archive; history moves with it.
    Archive { id: String },
    /// Correct a live record's own fields; state stays a command's move.
    Edit(EditArgs),
    /// Find records — the archive included — by substring.
    Search {
        query: String,
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// The whole notebook as one page, grouped by type.
    Overview,
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

/// What a refusal can point back at: the verb and the record it named.
pub struct Subject {
    pub verb: &'static str,
    pub id: Option<String>,
}

/// The command's subject, taken before dispatch consumes the command.
#[must_use]
pub fn subject(command: &Command) -> Subject {
    let (verb, id) = match command {
        Command::Add(_) => ("add", None),
        Command::Start { id } => ("start", Some(id)),
        Command::Submit { id } => ("submit", Some(id)),
        Command::Close(args) => ("close", Some(&args.id)),
        Command::Return { id } => ("return", Some(id)),
        Command::Reopen { id } => ("reopen", Some(id)),
        Command::Hold { id, .. } => ("hold", Some(id)),
        Command::Unhold { id } => ("unhold", Some(id)),
        Command::Block { id, .. } => ("block", Some(id)),
        Command::Unblock { id, .. } => ("unblock", Some(id)),
        Command::Comment { id, .. } => ("comment", Some(id)),
        Command::Decide(_) => ("decide", None),
        Command::Note(_) => ("note", None),
        Command::Ask(_) => ("ask", None),
        Command::Answer { id, .. } => ("answer", Some(id)),
        Command::Retire { id } => ("retire", Some(id)),
        Command::View { id } => ("view", Some(id)),
        Command::Ready { .. } => ("ready", None),
        Command::List { .. } => ("list", None),
        Command::Status { .. } => ("status", None),
        Command::Check { .. } => ("check", None),
        Command::Archive { id } => ("archive", Some(id)),
        Command::Edit(args) => ("edit", Some(&args.id)),
        Command::Search { .. } => ("search", None),
        Command::Overview => ("overview", None),
    };
    Subject {
        verb,
        id: id.cloned(),
    }
}

/// The envelope flags every create shares; each command adds its type's
/// own on top.
#[derive(Args)]
pub struct DraftArgs {
    pub title: String,
    /// Explicit id; omitted, one is minted from the title.
    #[arg(long)]
    pub id: Option<String>,
    /// Origin: the record this record was born from.
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    /// `<kind> <target>`, e.g. `pr https://…`; repeatable.
    #[arg(long = "link")]
    pub links: Vec<String>,
    #[arg(long)]
    pub body: Option<String>,
    /// The accountable identity; omitted, git identity fills it.
    #[arg(long)]
    pub by: Option<String>,
    /// The acting agent tool.
    #[arg(long)]
    pub via: Option<String>,
}

#[derive(Args)]
pub struct AddArgs {
    #[command(flatten)]
    pub draft: DraftArgs,
    /// 0–4, 0 the most urgent.
    #[arg(long)]
    pub priority: Option<u8>,
}

#[derive(Args)]
pub struct DecideArgs {
    #[command(flatten)]
    pub draft: DraftArgs,
    /// rule, shape, or drift.
    #[arg(long)]
    pub kind: Option<String>,
    /// The Decision this one replaces; it flips in the same move.
    #[arg(long)]
    pub supersedes: Option<String>,
}

#[derive(Args)]
pub struct NoteArgs {
    #[command(flatten)]
    pub draft: DraftArgs,
    /// fact, term, or guide.
    #[arg(long)]
    pub kind: Option<String>,
    /// The Note this one replaces; it retires in the same move.
    #[arg(long)]
    pub supersedes: Option<String>,
}

#[derive(Args)]
pub struct EditArgs {
    pub id: String,
    #[arg(long)]
    pub title: Option<String>,
    /// The whole body, replaced; empty clears it.
    #[arg(long)]
    pub body: Option<String>,
    /// Add a tag; repeatable.
    #[arg(long = "tag")]
    pub add_tags: Vec<String>,
    /// Remove a tag; repeatable.
    #[arg(long = "untag")]
    pub remove_tags: Vec<String>,
    /// Origin: the record this record was born from.
    #[arg(long)]
    pub from: Option<String>,
    /// 0–4, 0 the most urgent.
    #[arg(long)]
    pub priority: Option<u8>,
    /// The explicit resurfacing date.
    #[arg(long)]
    pub review_by: Option<String>,
}

#[derive(Args)]
pub struct CloseArgs {
    pub id: String,
    /// Proof, the default route: the report file, ingested as a Note the
    /// notebook carries, so a reader reaches it without leaving the repo.
    #[arg(long)]
    pub note: Option<String>,
    /// Proof: the pull request that shipped the work.
    #[arg(long)]
    pub pr: Option<String>,
    /// Proof: the commit that shipped the work.
    #[arg(long)]
    pub sha: Option<String>,
    /// Proof: a file left where it lies — right for a living document,
    /// which a Note would freeze into a second source of truth.
    #[arg(long)]
    pub report: Option<String>,
    /// The explicit waiver: close stating there is no proof.
    #[arg(long)]
    pub no_proof: bool,
}
