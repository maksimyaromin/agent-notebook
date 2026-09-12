//! The command surface for project memory and its working lifecycle.
//!
//! Flags with domain semantics stay optional here and are judged by the
//! Core, so their refusals arrive as structured recovery payloads; clap
//! checks positional arguments and flag spelling.

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
    /// JSON instead of TOON, with the same fields and selected content.
    #[arg(long, global = true)]
    pub json: bool,
    /// Use this notebook path instead of the nearest .agent-notebook.
    /// Overrides `ANB_NOTEBOOK`; relative paths start at the working directory.
    #[arg(long, global = true, value_name = "PATH")]
    pub notebook: Option<std::path::PathBuf>,
    /// Your private knowledge across projects, in the home directory.
    /// Holds Notes and Decisions. Cannot combine with --notebook or --personal.
    #[arg(long, global = true)]
    pub global: bool,
    /// Your private knowledge for this project, outside the repository.
    /// Recall includes it automatically alongside shared project knowledge.
    #[arg(long, global = true, conflicts_with_all = ["global", "notebook"])]
    pub personal: bool,
    /// Local agent session; `ANB_SESSION` or `CODEX_THREAD_ID` supplies the default.
    #[arg(long, global = true, value_name = "ID")]
    pub session: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Native `SessionStart` adapter installed by setup.
    #[command(hide = true)]
    Hook,
    /// Recall your work, shared knowledge and personal practices. Search by
    /// a phrase, or name a record with --for to prioritize its context.
    Recall {
        /// Match this phrase in record titles, tags, authors or bodies.
        text: Option<String>,
        /// Prioritize knowledge related to this project record.
        #[arg(long = "for", value_name = "ID")]
        focus: Option<String>,
        /// Include every matching record and its full body.
        #[arg(long)]
        all: bool,
        #[command(flatten)]
        whose: Whose,
    },
    /// Create a record: `add task|decision|note|question "<title>"`. The
    /// notebook appears on first write.
    Add(AddArgs),
    /// Start or resume a Task. Records your assignment and refuses work held by someone else.
    Start {
        /// The Task to start; omitted, resume the current session's focus.
        id: Option<String>,
        /// Start the next ready Task: your assigned work first, then unclaimed work.
        #[arg(long, conflicts_with = "id")]
        next: bool,
        /// Restrict --next to this record's scope.
        #[arg(long = "for", value_name = "ID", requires = "next")]
        hub: Option<String>,
        /// Share the Task with another local session without taking its focus away.
        #[arg(long)]
        join: bool,
    },
    /// Submit active work for review, optionally naming the reviewer.
    Submit {
        id: String,
        /// The reviewer, whose Status will show this Task. If omitted,
        /// keep the existing recipient or leave the review unassigned.
        #[arg(long, value_name = "NAME")]
        to: Option<String>,
    },
    /// Close completed work with an outcome. Use --reason to cancel a Task
    /// or settle a Question, or --resolved-by to cite its answer.
    Close(CloseArgs),
    /// Reopen a closed Task, preserving its recorded outcome.
    Reopen { id: String },
    /// Pause a Task deliberately; the reason is mandatory.
    Hold {
        id: String,
        /// Why the Task cannot continue.
        #[arg(long, allow_hyphen_values = true)]
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
    /// Append an attributed entry to a record in the working set, preserving its body.
    Comment {
        id: String,
        /// Entry text; --body and --body-file are alternatives.
        #[arg(allow_hyphen_values = true, conflicts_with_all = ["body", "body_file"])]
        text: Option<String>,
        /// Entry text, including Markdown and line breaks.
        #[arg(long, allow_hyphen_values = true)]
        body: Option<String>,
        /// Read the entry from a file; `-` reads standard input.
        #[arg(long, value_name = "PATH", conflicts_with = "body")]
        body_file: Option<String>,
        /// The acting agent tool writing the entry.
        #[arg(long)]
        via: Option<String>,
    },
    /// Retire a Decision or Note that no longer applies and has no successor.
    Retire {
        id: String,
        /// Append the outcome before retiring the record.
        #[arg(long, allow_hyphen_values = true)]
        body: Option<String>,
        /// Read the outcome from a file; `-` reads standard input.
        #[arg(long, value_name = "PATH", conflicts_with = "body")]
        body_file: Option<String>,
        /// The acting agent tool recording the outcome.
        #[arg(long)]
        via: Option<String>,
    },
    /// List Tasks that can start now, ordered by urgency, creation date and id.
    Ready {
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
        #[command(flatten)]
        narrowing: Narrowing,
    },
    /// List records in the working set. Filter by subject, type or person.
    List {
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
        #[command(flatten)]
        narrowing: Narrowing,
        #[command(flatten)]
        extent: Extent,
    },
    /// Read one record and its incoming and outgoing relationships.
    Show {
        id: String,
        /// Include complete fields, body and relationships without display limits.
        #[arg(long)]
        all: bool,
    },
    /// Summarize active work, the ready queue, Questions and items needing attention.
    Status {
        /// Override the configured token budget for this call; 0 removes the ceiling.
        #[arg(long)]
        budget: Option<u32>,
        #[command(flatten)]
        whose: Whose,
    },
    /// Verify every file: each finding names where it is, why, and what repairs it.
    Check {
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// Import record directories and their archive, preserving ids, bodies and source dates.
    Import {
        /// Directory containing tasks/, decisions/, notes/, questions/ and optional archive/.
        dir: std::path::PathBuf,
        /// Validate the complete result and list new files without writing them.
        #[arg(long)]
        check: bool,
    },
    /// Normalize record envelopes to YAML, keeping recoverable originals and all record history.
    /// YAML quotes delimit text; review older values that used surrounding quotes literally.
    Migrate {
        /// Validate and list envelope changes without writing files or backups.
        #[arg(long)]
        check: bool,
    },
    /// List stale work, unresolved references and other items needing attention.
    Debt {
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
    /// Change a record's wording, assignment, tags or links. Use lifecycle commands for state.
    Edit(EditArgs),
    /// Read the relationship graph for a notebook or a selected group of records.
    Graph(GraphArgs),
    /// Install instructions, skills and supported session hooks at the project root.
    /// Re-running updates managed files and preserves project-owned instructions.
    Setup {
        /// An agent to wire: `claude-code`, `codex`, or `agents-md` for any
        /// tool that reads `AGENTS.md` and `.agents/skills`; repeatable.
        #[arg(long = "agent", value_name = "NAME")]
        agents: Vec<String>,
        /// Remove managed integration files for the named agents.
        #[arg(long)]
        remove: bool,
    },
    /// Print, write or check the workflow skill and its generated references.
    Skill {
        /// The skill directory to write `SKILL.md` and its references into;
        /// omitted, `SKILL.md` prints.
        dir: Option<std::path::PathBuf>,
        /// Compare the directory with the rendering instead of writing it;
        /// a difference is a failing exit, for CI.
        #[arg(long, requires = "dir")]
        check: bool,
    },
}

/// One creation command for every record type: the envelope flags all
/// four share, plus the ones only some types carry. The Core refuses a
/// flag foreign to the type by name.
#[derive(Args)]
pub struct AddArgs {
    /// task, decision, note, or question.
    #[arg(value_parser = a_record_type)]
    pub record_type: RecordType,
    #[arg(allow_hyphen_values = true)]
    pub title: String,
    /// Choose a stable id. By default the CLI allocates a random 128-bit id.
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
    /// Record body text; omitted, the body starts empty.
    #[arg(long, allow_hyphen_values = true)]
    pub body: Option<String>,
    /// Read the body from a file; `-` reads standard input. Cannot combine with --body.
    #[arg(long = "body-file", value_name = "PATH", conflicts_with = "body")]
    pub body_file: Option<String>,
    /// The accountable identity; omitted, `ANB_BY` or the git identity fills it.
    #[arg(long)]
    pub by: Option<String>,
    /// The acting agent tool.
    #[arg(long)]
    pub via: Option<String>,
    /// Assign the Task to this person. If omitted, leave it unassigned until someone starts it.
    #[arg(long = "taken-by", value_name = "NAME", conflicts_with = "mine")]
    pub taken_by: Option<String>,
    /// Assign the Task to yourself, using the current identity.
    #[arg(long)]
    pub mine: bool,
    /// Whom the record waits on: the person a question is put to, or the
    /// one a task's review will be handed to. Omitted, it waits on nobody
    /// in particular.
    #[arg(long, value_name = "NAME")]
    pub to: Option<String>,
    /// A task's urgency, 0 to 4, 0 the most urgent.
    #[arg(long)]
    pub priority: Option<u32>,
    /// A decision's rule, shape, or drift; a note's fact, term, guide, idea, model, or spec.
    #[arg(long)]
    pub kind: Option<String>,
    /// The Decision or Note this record replaces; marks its predecessor superseded in the same operation.
    #[arg(long)]
    pub supersedes: Option<String>,
}

#[derive(Args)]
pub struct EditArgs {
    pub id: String,
    /// The whole title, replaced.
    #[arg(long, allow_hyphen_values = true)]
    pub title: Option<String>,
    /// The whole body, replaced; empty clears it.
    #[arg(long, allow_hyphen_values = true)]
    pub body: Option<String>,
    /// Replace the body with a file's text; `-` reads standard input. Cannot combine with --body.
    #[arg(long = "body-file", value_name = "PATH", conflicts_with = "body")]
    pub body_file: Option<String>,
    /// Add a tag; repeatable.
    #[arg(long = "tag", value_name = "TAG")]
    pub add_tags: Vec<String>,
    /// Remove a tag; repeatable.
    #[arg(long = "untag", value_name = "TAG")]
    pub remove_tags: Vec<String>,
    /// Add a link, `<kind> <target>`; repeatable. Use a record id for
    /// a shared relationship or a URL for an external source.
    #[arg(long = "link", value_name = "LINK")]
    pub add_links: Vec<String>,
    /// Remove a link, spelled as it stands; repeatable.
    #[arg(long = "unlink", value_name = "LINK")]
    pub remove_links: Vec<String>,
    /// Origin: the record this record was born from.
    #[arg(long)]
    pub from: Option<String>,
    /// 0 to 4, 0 the most urgent.
    #[arg(long)]
    pub priority: Option<u32>,
    /// The explicit resurfacing date.
    #[arg(long, value_name = "DATE")]
    pub review_by: Option<String>,
    /// Assign the Task to this person, allowing them to start it.
    #[arg(long = "taken-by", value_name = "NAME")]
    pub taken_by: Option<String>,
    /// Whom the task or question waits on.
    #[arg(long, value_name = "NAME")]
    pub to: Option<String>,
    /// The optional field to erase: `from`, `priority`, `review-by`,
    /// `taken-by`, or `to`; repeatable.
    #[arg(long = "clear", value_name = "FIELD")]
    pub clear: Vec<String>,
}

#[derive(Args)]
pub struct CloseArgs {
    pub id: String,
    /// Record the completed Task's outcome in its body.
    #[arg(long, allow_hyphen_values = true)]
    pub body: Option<String>,
    /// Read the outcome from a file; `-` reads standard input.
    #[arg(long, value_name = "PATH", conflicts_with = "body")]
    pub body_file: Option<String>,
    /// The acting agent tool recording the outcome.
    #[arg(long)]
    pub via: Option<String>,
    /// Cancel a Task or settle a Question with an explanation recorded in its header.
    #[arg(long, value_name = "WHY", allow_hyphen_values = true)]
    pub reason: Option<String>,
    /// The Decision or Task that settled the Question.
    #[arg(long, value_name = "ID")]
    pub resolved_by: Option<String>,
}

#[derive(Args)]
pub struct GraphArgs {
    /// Only this record and the graph around it.
    #[arg(long, value_name = "ID")]
    pub focus: Option<String>,
    /// How many edges out from `--focus` the graph reaches; 1 by default.
    #[arg(long, value_name = "N", requires = "focus")]
    pub depth: Option<usize>,
    /// Each record's envelope and body as well.
    #[arg(long)]
    pub full: bool,
    /// Include complete record fields and bodies. Both formats always include every graph edge.
    #[arg(long)]
    pub all: bool,
    #[command(flatten)]
    pub narrowing: Narrowing,
    #[command(flatten)]
    pub extent: Extent,
}

/// The heading the narrowing flags print under, so `--help` and the
/// reference teach them once for every verb that takes them.
pub const NARROWING: &str = "Narrowing";

/// Whose records a read answers with. Without any of the three, the
/// notebook's `scope` key decides.
#[derive(Args, Default)]
#[command(next_help_heading = NARROWING)]
pub struct Whose {
    /// Only this identity's work: the tasks it holds, the records it wrote
    /// and the records waiting on it.
    #[arg(long, value_name = "NAME", conflicts_with_all = ["mine", "team"])]
    pub by: Option<String>,
    /// Only your own: `--by` with the identity the writers sign with.
    #[arg(long, conflicts_with = "team")]
    pub mine: bool,
    /// Everyone's, whatever the notebook's `scope` key says.
    #[arg(long)]
    pub team: bool,
}

/// Shared listing and graph filters. Multiple filters select their intersection.
#[derive(Args)]
#[command(next_help_heading = NARROWING)]
pub struct Narrowing {
    #[command(flatten)]
    pub whose: Whose,
    /// Only the tasks nobody holds, the pool anyone may take, whatever the
    /// notebook's `scope` key says.
    #[arg(long, conflicts_with_all = ["by", "mine", "team"])]
    pub untaken: bool,
    /// Only the records addressed to this person.
    #[arg(long, value_name = "NAME")]
    pub to: Option<String>,
    /// Only this subject and records created from it, following origins
    /// through every descendant. Dependencies still govern readiness.
    #[arg(long = "for", value_name = "ID")]
    pub scope: Option<String>,
    /// Only records carrying this tag; repeated, carrying every one.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Match text in record ids, titles, tags, people or bodies, ignoring case.
    #[arg(long = "match", value_name = "TEXT")]
    pub text: Option<String>,
}

/// Which records a listing reaches at all: their types, their kinds, and
/// the archive. The queue is live open Tasks by definition, so `ready`
/// takes none of these.
#[derive(Args, Default)]
#[command(next_help_heading = NARROWING)]
pub struct Extent {
    /// Select record types, comma-separated or repeated. By default, include
    /// every type and records whose type could not be parsed.
    #[arg(long = "type", value_name = "TYPE", value_delimiter = ',', value_parser = a_record_type)]
    pub types: Vec<RecordType>,
    /// Only records of these kinds, comma-separated or repeated: a
    /// Decision's rule, shape or drift; a Note's fact, term, guide, idea,
    /// model or spec.
    #[arg(long = "kind", value_name = "KIND", value_delimiter = ',')]
    pub kinds: Vec<String>,
    /// The archive too; by default only the work still in play.
    #[arg(long)]
    pub archive: bool,
}

/// One type word as the type it names. The types are asked of the Core
/// rather than retyped here, so a fifth one is accepted the day it exists.
fn a_record_type(word: &str) -> Result<RecordType, String> {
    RecordType::from_word(word).ok_or_else(|| {
        format!(
            "`{word}` is no type of record; try {}",
            RecordType::ALL.map(RecordType::word).join(", ")
        )
    })
}
