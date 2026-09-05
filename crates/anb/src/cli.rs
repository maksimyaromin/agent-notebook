//! The command surface: conventional tracker verbs over the notebook.
//!
//! Flags with domain semantics stay optional here and are judged by the
//! Core, so their refusals arrive as structured recovery payloads; clap
//! keeps only the structural surface — positionals and flag spelling.

use anb_core::RecordType;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "anb",
    version,
    about = "A project's working memory as typed records in plain files"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    /// Compact JSON instead of plain text, on every command.
    #[arg(long, global = true)]
    pub json: bool,
    /// Where the notebook lives, read from here and outranking
    /// `ANB_NOTEBOOK`, which is read from the project. By default the
    /// nearest `.agent-notebook` at or above the working directory.
    #[arg(long, global = true, value_name = "PATH")]
    pub notebook: Option<std::path::PathBuf>,
    /// The user's notebook — `.agent-notebook` in the home directory —
    /// instead of the project's, outranking `ANB_NOTEBOOK` like
    /// `--notebook` and refused beside it. It holds knowledge that outlives
    /// one repository, so the verbs that create or move a task or a
    /// question refuse it.
    #[arg(long, global = true)]
    pub global: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a record — `add task|decision|note|question "<title>"`; the
    /// notebook appears on first write.
    Add(AddArgs),
    /// open | review → active: take the Task into work, or back into it.
    Start { id: String },
    /// active → review: hand the work to a human for acceptance.
    Submit { id: String },
    /// active | review → closed, carrying its proof; --reason ends a Task or
    /// a Question without work, from open too; --resolved-by closes a
    /// Question into the record that settled it.
    Close(CloseArgs),
    /// closed → open, explicitly.
    Reopen { id: String },
    /// Pause a Task deliberately; the reason is mandatory.
    Hold {
        id: String,
        /// Why the Task waits; an unreasoned hold is where work rots.
        #[arg(long)]
        reason: Option<String>,
        /// Calendar hold: the date to resume on.
        #[arg(long, value_name = "DATE")]
        until: Option<String>,
    },
    /// Resume a held Task.
    Unhold { id: String },
    /// Write a dependency edge: this Task waits on another.
    Block {
        /// The Task that waits.
        id: String,
        /// The Task waited on; it closes first.
        on: String,
    },
    /// Erase a dependency edge.
    Unblock {
        /// The Task that was waiting.
        id: String,
        /// The Task no longer waited on.
        on: String,
    },
    /// Append one entry to a Task's log — where the next session resumes.
    Comment {
        id: String,
        text: String,
        /// The acting agent tool writing the entry.
        #[arg(long)]
        via: Option<String>,
    },
    /// active → retired: end a Decision or Note that has no successor.
    Retire { id: String },
    /// The dispatch queue: open, unblocked, unheld Tasks, most urgent first.
    Ready {
        /// Only work this record's scope reaches: an epic's own queue.
        #[arg(long = "for", value_name = "ID")]
        scope: Option<String>,
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// Every live record.
    List {
        /// Only records this one's scope reaches: an epic and its work.
        #[arg(long = "for", value_name = "ID")]
        scope: Option<String>,
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// One record: envelope, body, and its mention blocks.
    Show {
        id: String,
        /// Every line and every mention; a long body and a crowded block
        /// print bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// The session Status: one quiet line, or the budgeted composite.
    Status {
        /// Token ceiling for this call, outranking the config key; 0 = no ceiling.
        #[arg(long)]
        budget: Option<u32>,
        /// The session-start payload for an agent hook; fails soft.
        #[arg(long)]
        hook: bool,
    },
    /// Verify every file: each finding names where it is, why, and what repairs it.
    Check {
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// Move a settled record into the archive; history moves with it.
    Archive { id: String },
    /// Move an archived record back into the working set: same filename, same bytes.
    Restore { id: String },
    /// Delete a record born by mistake; refuses while anything cites it.
    Delete { id: String },
    /// Correct a live record's own fields; state stays a command's move.
    Edit(EditArgs),
    /// Find records — the archive included — by substring.
    Search {
        query: String,
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// The notebook as records and the edges between them.
    Graph(GraphArgs),
    /// The whole notebook as one page, grouped by type.
    Overview {
        /// Every row; each section is bounded by default.
        #[arg(long)]
        all: bool,
    },
}

/// One creation command for every record type: the envelope flags all
/// four share, plus the ones only some types carry — the Core refuses a
/// flag foreign to the type by name.
#[derive(Args)]
pub struct AddArgs {
    /// task, decision, note, or question.
    #[arg(value_parser = a_record_type)]
    pub record_type: RecordType,
    pub title: String,
    /// Explicit id; omitted, one is minted from the title.
    #[arg(long)]
    pub id: Option<String>,
    /// Origin: the record this record was born from.
    #[arg(long)]
    pub from: Option<String>,
    /// A tag; repeatable.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// `<kind> <target>`, e.g. `pr https://…`; repeatable.
    #[arg(long = "link", value_name = "LINK")]
    pub links: Vec<String>,
    /// The prose under the envelope; omitted, the record opens empty.
    #[arg(long)]
    pub body: Option<String>,
    /// The accountable identity; omitted, git identity fills it.
    #[arg(long)]
    pub by: Option<String>,
    /// The acting agent tool.
    #[arg(long)]
    pub via: Option<String>,
    /// A task's urgency, 0–4, 0 the most urgent.
    #[arg(long)]
    pub priority: Option<u32>,
    /// A decision's rule, shape, or drift; a note's fact, term, or guide.
    #[arg(long)]
    pub kind: Option<String>,
    /// The Decision or Note this one replaces; it flips in the same move.
    #[arg(long)]
    pub supersedes: Option<String>,
}

#[derive(Args)]
pub struct EditArgs {
    pub id: String,
    /// The whole title, replaced.
    #[arg(long)]
    pub title: Option<String>,
    /// The whole body, replaced; empty clears it.
    #[arg(long)]
    pub body: Option<String>,
    /// Add a tag; repeatable.
    #[arg(long = "tag", value_name = "TAG")]
    pub add_tags: Vec<String>,
    /// Remove a tag; repeatable.
    #[arg(long = "untag", value_name = "TAG")]
    pub remove_tags: Vec<String>,
    /// Origin: the record this record was born from.
    #[arg(long)]
    pub from: Option<String>,
    /// 0–4, 0 the most urgent.
    #[arg(long)]
    pub priority: Option<u32>,
    /// The explicit resurfacing date.
    #[arg(long, value_name = "DATE")]
    pub review_by: Option<String>,
    /// The optional field to erase: `from`, `priority`, or `review-by`;
    /// repeatable.
    #[arg(long = "clear", value_name = "FIELD")]
    pub clear: Vec<String>,
}

#[derive(Args)]
pub struct CloseArgs {
    pub id: String,
    /// Proof, the default route: the report file, ingested as a Note the
    /// notebook carries, so a reader reaches it through the notebook alone.
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
    /// End a Task or a Question without work, stating why; the reason lands
    /// in the envelope and no proof is written.
    #[arg(long, value_name = "WHY")]
    pub reason: Option<String>,
    /// The Decision or Task that settled the Question.
    #[arg(long, value_name = "ID")]
    pub resolved_by: Option<String>,
}

#[derive(Args)]
pub struct GraphArgs {
    #[command(flatten)]
    pub slice: SliceArgs,
    /// Each record's envelope and body as well.
    #[arg(long)]
    pub full: bool,
    /// Every row the plain text bounds. JSON is never bounded: a graph
    /// missing edges is not a smaller graph, it is a wrong one.
    #[arg(long)]
    pub all: bool,
}

/// Which Tasks the graph holds. They travel together because a narrowing
/// honoured while printing and dropped while drawing hands the reader a
/// picture of another notebook.
#[derive(Args)]
pub struct SliceArgs {
    /// Only the work this record's scope reaches: one epic's branch.
    #[arg(long = "for", value_name = "ID")]
    pub scope: Option<String>,
    /// Only records of these types. Every type by default, including one
    /// whose own `type` field no notebook word matches.
    #[arg(long = "type", value_name = "TYPE", value_delimiter = ',', value_parser = a_record_type)]
    pub types: Vec<RecordType>,
    /// Only what can be started now: the ready lens.
    #[arg(long)]
    pub ready: bool,
    /// Only this record and the graph around it.
    #[arg(long, value_name = "ID")]
    pub focus: Option<String>,
    /// How many edges out from `--focus` the graph reaches; 1 by default.
    #[arg(long, value_name = "N", requires = "focus")]
    pub depth: Option<usize>,
    /// The archive too; by default only the work still in play.
    #[arg(long)]
    pub archive: bool,
}

/// One type word as the type it names. The types are asked of the Core
/// rather than retyped here, so a fifth one is accepted the day it exists.
fn a_record_type(word: &str) -> Result<RecordType, String> {
    RecordType::from_word(word).ok_or_else(|| {
        format!(
            "`{word}` is no type of record — try {}",
            RecordType::ALL.map(RecordType::word).join(", ")
        )
    })
}
