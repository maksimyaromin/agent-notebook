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
    /// The user's notebook, `.agent-notebook` in the home directory,
    /// instead of the project's; it outranks `ANB_NOTEBOOK` like
    /// `--notebook` and is refused beside it. It holds knowledge that
    /// outlives one repository, so the verbs that create or move a task or
    /// a question refuse it.
    #[arg(long, global = true)]
    pub global: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a record: `add task|decision|note|question "<title>"`. The
    /// notebook appears on first write.
    Add(AddArgs),
    /// open | review → active: take the Task into work, or back into it;
    /// the Task records who took it, and one taken by someone else is refused.
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
    /// Append one entry to a Task's log, where the next session resumes;
    /// the entry is signed by the identity, `/` the tool when `--via` names one.
    Comment {
        id: String,
        text: String,
        /// The acting agent tool writing the entry.
        #[arg(long)]
        via: Option<String>,
    },
    /// active → retired: end a Decision or Note that has no successor.
    Retire { id: String },
    /// The dispatch queue: open, unblocked, unheld Tasks, most urgent first,
    /// each naming who took it when someone did.
    Ready {
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
        #[command(flatten)]
        narrowing: Narrowing,
    },
    /// The records, ids and titles out: every live one by default, or the
    /// ones the narrowing admits.
    List {
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
        #[command(flatten)]
        narrowing: Narrowing,
        #[command(flatten)]
        extent: Extent,
    },
    /// One record: envelope, body, and its mention blocks.
    Show {
        id: String,
        /// Every line and every mention; a long body and a crowded block
        /// print bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// The session Status: the work, one quiet line when there is none, or
    /// the budgeted composite.
    Status {
        /// Token ceiling for this call, outranking the config key; 0 = no ceiling.
        #[arg(long)]
        budget: Option<u32>,
        /// The session-start payload for an agent hook; fails soft.
        #[arg(long)]
        hook: bool,
        #[command(flatten)]
        whose: Whose,
    },
    /// Verify every file: each finding names where it is, why, and what repairs it.
    Check {
        /// Every row; the listing is bounded by default.
        #[arg(long)]
        all: bool,
    },
    /// The Debt: every sign of decay Status counts, each on its own line.
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
    /// Correct a live record's own fields; state stays a command's move.
    Edit(EditArgs),
    /// The notebook as records and the edges between them: `list` with
    /// edges.
    Graph(GraphArgs),
    /// Wire the named agents to the notebook, in this directory: the
    /// one-line snippet in the instruction file each reads, the
    /// `SessionStart` hook where its host runs one, and the anb skills
    /// where it looks for skills. Re-running patches in place.
    Setup {
        /// An agent to wire: `claude-code`, `codex`, or `agents-md` for any
        /// tool that reads `AGENTS.md` and `.agents/skills`; repeatable.
        #[arg(long = "agent", value_name = "NAME")]
        agents: Vec<String>,
        /// Take out what setup put in for the named agents, and nothing
        /// else.
        #[arg(long)]
        remove: bool,
    },
    /// The skill an agent learns the tool from, rendered from the binary:
    /// printed, written into a directory, or checked against one.
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
    /// The prose under the envelope, read from a file; `-` reads standard
    /// input. Refused beside --body.
    #[arg(long = "body-file", value_name = "PATH", conflicts_with = "body")]
    pub body_file: Option<String>,
    /// The accountable identity; omitted, `ANB_BY` or the git identity fills it.
    #[arg(long)]
    pub by: Option<String>,
    /// The acting agent tool.
    #[arg(long)]
    pub via: Option<String>,
    /// A task's urgency, 0 to 4, 0 the most urgent.
    #[arg(long)]
    pub priority: Option<u32>,
    /// A decision's rule, shape, or drift; a note's fact, term, guide, idea, model, or spec.
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
    /// The whole body, replaced with a file's text; `-` reads standard
    /// input. Refused beside --body.
    #[arg(long = "body-file", value_name = "PATH", conflicts_with = "body")]
    pub body_file: Option<String>,
    /// Add a tag; repeatable.
    #[arg(long = "tag", value_name = "TAG")]
    pub add_tags: Vec<String>,
    /// Remove a tag; repeatable.
    #[arg(long = "untag", value_name = "TAG")]
    pub remove_tags: Vec<String>,
    /// Add a link, `<kind> <target>`; repeatable. A Decision that cites
    /// another as context declares it here, and the pair leaves
    /// `may-conflict`.
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
    /// Who took the task: the hand-over that lets another identity start it.
    #[arg(long = "taken-by", value_name = "NAME")]
    pub taken_by: Option<String>,
    /// The optional field to erase: `from`, `priority`, `review-by`, or
    /// `taken-by`; repeatable.
    #[arg(long = "clear", value_name = "FIELD")]
    pub clear: Vec<String>,
}

#[derive(Args)]
pub struct CloseArgs {
    pub id: String,
    /// Proof, the default route: the report file, ingested as a Note the
    /// notebook carries, so a reader reaches it through the notebook alone;
    /// `-` reads standard input.
    #[arg(long)]
    pub note: Option<String>,
    /// Proof: the pull request that shipped the work.
    #[arg(long)]
    pub pr: Option<String>,
    /// Proof: the commit that shipped the work.
    #[arg(long)]
    pub sha: Option<String>,
    /// Proof: a file left where it lies, right for a living document,
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
    /// Only this record and the graph around it.
    #[arg(long, value_name = "ID")]
    pub focus: Option<String>,
    /// How many edges out from `--focus` the graph reaches; 1 by default.
    #[arg(long, value_name = "N", requires = "focus")]
    pub depth: Option<usize>,
    /// Each record's envelope and body as well.
    #[arg(long)]
    pub full: bool,
    /// Every row the plain text bounds. JSON is never bounded: a graph
    /// missing edges is not a smaller graph, it is a wrong one.
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
#[derive(Args)]
#[command(next_help_heading = NARROWING)]
pub struct Whose {
    /// Only records this identity created or took.
    #[arg(long, value_name = "NAME", conflicts_with_all = ["mine", "team"])]
    pub by: Option<String>,
    /// Only your own: `--by` with the identity the writers sign with.
    #[arg(long, conflicts_with = "team")]
    pub mine: bool,
    /// Everyone's, whatever the notebook's `scope` key says.
    #[arg(long)]
    pub team: bool,
}

/// What every listing narrows by. Each flag is a predicate over the same
/// notebook, so two flags ask for the intersection, and a narrowing
/// honoured while printing is honoured while drawing: a picture of another
/// notebook is no picture of this one.
#[derive(Args)]
#[command(next_help_heading = NARROWING)]
pub struct Narrowing {
    #[command(flatten)]
    pub whose: Whose,
    /// Only records inside this record's scope: an epic, what it waits on
    /// and what was born inside it.
    #[arg(long = "for", value_name = "ID")]
    pub scope: Option<String>,
    /// Only records carrying this tag; repeated, carrying every one.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Only records whose id, title, tags, people or body hold this text,
    /// whatever its case.
    #[arg(long = "match", value_name = "TEXT")]
    pub text: Option<String>,
}

/// Which records a listing reaches at all: their types, their kinds, and
/// the archive. The queue is live open Tasks by definition, so `ready`
/// takes none of these.
#[derive(Args, Default)]
#[command(next_help_heading = NARROWING)]
pub struct Extent {
    /// Only records of these types, comma-separated or repeated. Every
    /// type by default, including one whose own `type` field no notebook
    /// word matches.
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
