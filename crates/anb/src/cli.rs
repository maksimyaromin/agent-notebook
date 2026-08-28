//! The command surface: conventional tracker verbs over the notebook.
//!
//! Flags with domain semantics stay optional here and are judged by the
//! Core, so their refusals arrive as structured recovery payloads; clap
//! keeps only the structural surface — positionals and flag spelling.

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
        Command::View { id } => ("view", Some(id)),
        Command::Ready { .. } => ("ready", None),
        Command::List { .. } => ("list", None),
        Command::Status { .. } => ("status", None),
    };
    Subject {
        verb,
        id: id.cloned(),
    }
}

#[derive(Args)]
pub struct AddArgs {
    pub title: String,
    /// Explicit id; omitted, one is minted from the title.
    #[arg(long)]
    pub id: Option<String>,
    /// 0–4, 0 the most urgent.
    #[arg(long)]
    pub priority: Option<u8>,
    /// Origin: the record this Task was born from.
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
pub struct CloseArgs {
    pub id: String,
    /// Proof: the pull request that shipped the work.
    #[arg(long)]
    pub pr: Option<String>,
    /// Proof: the commit that shipped the work.
    #[arg(long)]
    pub sha: Option<String>,
    /// Proof: the report that documents the work.
    #[arg(long)]
    pub report: Option<String>,
    /// The explicit waiver: close stating there is no proof.
    #[arg(long)]
    pub no_proof: bool,
}
