//! The values a verb answers with: the vocabulary every host renders, and
//! the one rendering the hosts share.
//!
//! They sit below the Notebook and below every derivation over it, so a
//! surface that computes one — Debt, Status, Check — never reaches up into
//! the module that hands it out.

use crate::finding::Finding;
use crate::record::{Record, RecordType};
use crate::resolve::path_stem;
use crate::{date, encode};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// A state move; on a replay `already` is true and `from` equals `to`.
#[derive(Debug, PartialEq, Eq)]
pub struct Transitioned {
    pub id: String,
    pub from: &'static str,
    pub to: &'static str,
    pub already: bool,
}

impl Transitioned {
    /// The replay of a move already made: from and to are the state the
    /// record already stands in.
    pub(crate) fn replayed(id: &str, state: &'static str) -> Transitioned {
        Transitioned {
            id: id.to_owned(),
            from: state,
            to: state,
            already: true,
        }
    }
}

/// A close, with the computed consequences it must not bury: the still-open
/// Questions born from this Task, and the open Tasks whose last live
/// blocker it was, in ready order.
#[derive(Debug, PartialEq, Eq)]
pub struct Closed {
    pub transition: Transitioned,
    pub open_questions: Vec<String>,
    pub unblocked: Vec<String>,
    /// The Note this close ingested its report into, when it did.
    pub report_note: Option<String>,
    /// The ingested report's own dangling citations; a report names ids as
    /// freely as any body, and the nudge belongs at the write.
    pub dangling_mentions: Vec<String>,
}

/// A hold set or cleared; `already` marks the replay.
#[derive(Debug, PartialEq, Eq)]
pub struct Held {
    pub id: String,
    pub already: bool,
}

/// A dependency edge written or erased; `already` marks the replay.
#[derive(Debug, PartialEq, Eq)]
pub struct Edged {
    pub id: String,
    pub on: String,
    pub already: bool,
}

/// One ready Task. It carries `created`, not an age: the Core holds no
/// clock, so "how old" is the caller's derivation from its own today.
#[derive(Debug, PartialEq, Eq)]
pub struct ReadyTask {
    pub id: String,
    pub priority: Option<u8>,
    pub created: String,
    /// The Task's title, cut by [`encode::bounded_text`] like every other
    /// text a derived reply carries.
    pub title: String,
}

impl ReadyTask {
    /// The ready table — header and the first `shown` rows, ages derived
    /// from `today_day`. The caller owns its own truncation hint.
    #[must_use]
    pub fn table(rows: &[ReadyTask], shown: usize, today_day: i64) -> String {
        let mut out = format!("ready[{}]{{id,priority,age,title}}:\n", rows.len());
        for row in rows.iter().take(shown) {
            let priority = row
                .priority
                .map_or_else(|| "-".to_owned(), |priority| priority.to_string());
            let _ = writeln!(
                out,
                "  {},{priority},{}d,{}",
                row.id,
                age_days(&row.created, today_day),
                encode::quoted_if_delimited(&row.title)
            );
        }
        out
    }
}

/// Whole days from `created` to `today_day`, floored at zero; an unreadable
/// date counts as today.
fn age_days(created: &str, today_day: i64) -> i64 {
    date::day_number(created).map_or(0, |day| (today_day - day).max(0))
}

/// One record cited on a Debt surface, with the attribution the undeclared
/// conflict posture requires: the tool prints both sides and stops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cited {
    pub id: String,
    pub by: Option<String>,
    pub via: Option<String>,
}
impl Cited {
    pub(crate) fn of(record: &Record) -> Cited {
        Cited {
            id: path_stem(record.path()).to_owned(),
            by: record.file().field("by").map(str::to_owned),
            via: record.file().field("via").map(str::to_owned),
        }
    }

    /// `by`, plus `/via` when an agent hand wrote it; no identity at all
    /// prints as `-` — the reader judges the pair, so a side is never blank.
    #[must_use]
    pub fn author(&self) -> String {
        match (&self.by, &self.via) {
            (Some(by), Some(via)) => format!("{by}/{via}"),
            (Some(by), None) => by.clone(),
            (None, Some(via)) => format!("-/{via}"),
            (None, None) => "-".to_owned(),
        }
    }
}

/// A record created, with the computed consequences its reply must not
/// bury.
#[derive(Debug, PartialEq, Eq)]
pub struct Created {
    pub id: String,
    pub path: String,
    pub superseded: Option<String>,
    pub may_conflict: Vec<Cited>,
    pub dangling_mentions: Vec<String>,
}

/// A log entry appended; `already` marks the replay of the trail's tail.
#[derive(Debug, PartialEq, Eq)]
pub struct Commented {
    pub id: String,
    pub already: bool,
    pub dangling_mentions: Vec<String>,
}

/// A Question dropped; the replay of an already-dropped one carries no
/// mention nudge, since its reason wrote nothing.
#[derive(Debug, PartialEq, Eq)]
pub struct Dropped {
    pub transition: Transitioned,
    pub dangling_mentions: Vec<String>,
}

/// One record standing in the way of an expunge, and the edge that holds
/// it: an envelope key, or the body that cites the id in prose. One carrier
/// can hold several, so a blocker is an edge, not a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocker {
    pub carrier: String,
    pub through: &'static str,
}

impl std::fmt::Display for Blocker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} — {}", self.carrier, self.through)
    }
}

/// The distinct records among a blocker list, first appearance first: one
/// carrier is one thing to open, however many of its lines hold the record.
pub fn carriers_of(blockers: &[Blocker]) -> impl Iterator<Item = &str> {
    let mut seen = BTreeSet::new();
    blockers
        .iter()
        .map(|blocker| blocker.carrier.as_str())
        .filter(move |carrier| seen.insert(*carrier))
}

/// A record removed as a mistake, and every file that is gone. One id can
/// claim two files after a move interrupted in either direction; leaving
/// no trace means leaving neither.
#[derive(Debug, PartialEq, Eq)]
pub struct Expunged {
    pub id: String,
    pub paths: Vec<String>,
}

/// A record moved into the archive; `already` marks the replay.
#[derive(Debug, PartialEq, Eq)]
pub struct Archived {
    pub id: String,
    pub from: String,
    pub to: String,
    /// The report Notes this call filed alongside, in the order the record
    /// names them.
    pub carried: Vec<String>,
    pub already: bool,
}

/// A record moved back out of the archive; `already` marks the replay.
#[derive(Debug, PartialEq, Eq)]
pub struct Restored {
    pub id: String,
    pub from: String,
    pub to: String,
    pub already: bool,
}

/// An edit applied; `changed` names what moved in the file, so an empty
/// one is the replay whose write was skipped whole.
#[derive(Debug, PartialEq, Eq)]
pub struct Edited {
    pub id: String,
    pub changed: Vec<&'static str>,
    pub dangling_mentions: Vec<String>,
}

/// One row of the live listing; the id carries the type. An invalid
/// record's state shows as `invalid` — visible, never silently dropped.
#[derive(Debug, PartialEq, Eq)]
pub struct ListedRecord {
    pub id: String,
    pub state: String,
    pub priority: Option<u8>,
    /// Cut by [`encode::bounded_text`], as `ReadyTask`'s is.
    pub title: Option<String>,
}

/// An epic and where it stands: the hub Task, how many of the children it
/// waits on have closed, and the next dispatchable Task inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Epic {
    pub id: String,
    pub closed: usize,
    pub total: usize,
    pub next: Option<String>,
}

/// One record in the graph: what it is, what it relates to, and the record
/// itself for whoever draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    pub id: String,
    /// Which kind of record this is. The id carries it too, but a consumer
    /// that has to split a string to learn what it is drawing has been
    /// handed a puzzle rather than an answer. `None` is a record whose own
    /// `type` field is not a word this notebook knows.
    pub kind: Option<RecordType>,
    /// The state a reader sorts and colours by, `invalid` for a record its own
    /// findings exclude — the listing's word, so one vocabulary answers
    /// every surface.
    pub state: String,
    pub archived: bool,
    pub title: Option<String>,
    /// Where this hub stands, when the Task is one.
    pub epic: Option<Epic>,
    /// What a Task carries into the queue it waits in: nothing for a record
    /// that never queues.
    pub priority: Option<u8>,
    /// The day the record entered the notebook. The date rather than an age
    /// in days, because an age is only true on the day it was computed.
    pub created: String,
    /// Whether this Task can be started now — nothing blocks it and no hold
    /// stands. Derived from rules a consumer cannot see, so it travels with
    /// the node rather than being left to a second call. `None` where the
    /// question does not arise: a record that never queues, or work already
    /// filed.
    pub ready: Option<bool>,
    pub blocked_by: Vec<String>,
    pub origin: Option<String>,
    /// The envelope in file order and the body, for the record a reader
    /// reads rather than for the node.
    pub fields: Vec<(String, String)>,
    pub body: String,
    pub mentions: Vec<String>,
}

/// Which records a graph is asked for. Every narrowing is a predicate over the
/// same notebook, so asking for two asks for the intersection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphSlice {
    /// Which kinds of record the graph holds; empty asks for all of them,
    /// the records with no readable kind among them.
    pub kinds: Vec<RecordType>,
    /// One epic's scope: the hub, what it waits on, and what was born
    /// inside it.
    pub hub: Option<String>,
    /// What can be started now.
    pub ready_only: bool,
    /// One record and the graph around it.
    pub focus: Option<Focus>,
    /// Whether filed work is in the graph. Most of what a long-lived
    /// notebook holds is finished, and drawing all of it buries the work in
    /// flight, so the archive stays off until it is asked for.
    pub archive: bool,
}

/// A record and how far around it the graph reaches, counted in edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Focus {
    pub id: String,
    pub depth: usize,
}

/// The graph one call asked for: every record the slice reaches, each with the
/// edges it draws.
#[derive(Debug, PartialEq, Eq)]
pub struct Graph {
    /// What was asked for, carried back so a graph says which slice it is.
    pub slice: GraphSlice,
    pub nodes: Vec<GraphNode>,
}

impl Graph {
    /// The edges between the records this graph holds. A declared relation
    /// runs the way work becomes possible — out of what must settle first,
    /// into what waits on it or was born from it — while a mention runs the
    /// way it was written, out of the record that names another. An edge
    /// with an end outside this slice is not one, since both ends have to
    /// be somewhere a reader can see.
    #[must_use]
    pub fn edges<'a>(&'a self) -> Vec<GraphEdge<'a>> {
        let on_the_map: BTreeSet<&str> = self.nodes.iter().map(|node| node.id.as_str()).collect();
        // One pair of records is one line however many times the notebook
        // says so: a `blocked-by` listed twice, or a body naming the record
        // its envelope already points at, is one relation stated twice. Two
        // edges between the same pair would also weigh it twice in every
        // degree computed from this.
        let mut drawn = BTreeSet::new();
        let mut edges = Vec::new();
        let mut draw =
            |from: &'a str, to: &'a str, kind: EdgeKind, edges: &mut Vec<GraphEdge<'a>>| {
                if on_the_map.contains(from) && on_the_map.contains(to) && drawn.insert((from, to))
                {
                    edges.push(GraphEdge { from, to, kind });
                }
            };
        // Declared relations first, so a mention of the same pair finds it
        // taken and the stronger word is the one that survives.
        for node in &self.nodes {
            for blocker in &node.blocked_by {
                draw(blocker, &node.id, EdgeKind::BlockedBy, &mut edges);
            }
            if let Some(origin) = node.origin.as_deref() {
                draw(origin, &node.id, EdgeKind::Origin, &mut edges);
            }
        }
        for node in &self.nodes {
            for mentioned in &node.mentions {
                draw(&node.id, mentioned, EdgeKind::Mentions, &mut edges);
            }
        }
        edges
    }

    /// How many edges meet at each record. A graph reads by weight, and weight
    /// is how much of the slice a Task holds together.
    #[must_use]
    pub fn degrees(&self) -> BTreeMap<&str, usize> {
        let mut degrees: BTreeMap<&str, usize> = self
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), 0))
            .collect();
        for edge in self.edges() {
            *degrees.entry(edge.from).or_default() += 1;
            *degrees.entry(edge.to).or_default() += 1;
        }
        degrees
    }
}

/// One edge between two records the graph holds. Which way it runs depends
/// on its kind, so the kind is read before the direction is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphEdge<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub kind: EdgeKind,
}

/// How two records are related: by what one waits on, by what it was born
/// from, or by one naming the other in its prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    BlockedBy,
    Origin,
    /// One record naming another in its body: the web a notebook weaves
    /// beside the work it tracks.
    Mentions,
}

/// One type's slice of the overview page.
#[derive(Debug, PartialEq, Eq)]
pub struct TypeSection {
    pub record_type: RecordType,
    pub rows: Vec<ListedRecord>,
}

/// Live records per type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub tasks: usize,
    pub decisions: usize,
    pub notes: usize,
    pub questions: usize,
}

impl Counts {
    /// A tally read by the type each count is of, so neither side depends
    /// on [`RecordType::ALL`]'s order.
    #[must_use]
    pub fn per_type(held: [(RecordType, usize); RecordType::ALL.len()]) -> Counts {
        let mut counts = Counts::default();
        for (record_type, held) in held {
            let count = match record_type {
                RecordType::Task => &mut counts.tasks,
                RecordType::Decision => &mut counts.decisions,
                RecordType::Note => &mut counts.notes,
                RecordType::Question => &mut counts.questions,
            };
            *count = held;
        }
        counts
    }
}

/// The whole notebook as one page: every live record grouped by type, the
/// archive as counts — history is recoverable, not re-read.
#[derive(Debug, PartialEq, Eq)]
pub struct Overview {
    pub live: Counts,
    pub epics: Vec<Epic>,
    pub sections: Vec<TypeSection>,
    pub archived: Counts,
}

/// A proof one record cites, and what kind of thing it names. The host
/// settles these: the Core holds neither git nor a filesystem, and a proof
/// is a claim about both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitedProof {
    pub record: String,
    pub kind: String,
    pub target: String,
}

impl std::fmt::Display for CitedProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.kind, self.target)
    }
}

/// One record read whole: the envelope as it stands, the body, and the two
/// derived Mention blocks.
#[derive(Debug, PartialEq, Eq)]
pub struct View {
    pub id: String,
    pub path: String,
    pub archived: bool,
    pub fields: Vec<(String, String)>,
    pub body: String,
    pub mentions: Vec<String>,
    pub mentioned_by: Vec<String>,
}

/// One finding against the file that carries it: `check`'s row, and the
/// payload of a refusal that names a record it will not touch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFinding {
    pub path: String,
    pub finding: Finding,
    /// The move that erases this finding, when the notebook has one.
    pub repair: Option<Repair>,
}

impl FileFinding {
    /// A finding against a file, with no repair named yet.
    pub(crate) fn on(path: &str, finding: Finding) -> FileFinding {
        FileFinding {
            path: path.to_owned(),
            finding,
            repair: None,
        }
    }
}

/// The move that erases a finding: a verb the notebook already has,
/// aimed at the record the finding sits on.
///
/// The host spells it as a command. A finding on a line no verb writes
/// carries none, and neither does one on a file no verb can reach by id —
/// a record in the wrong directory, or under a filename that is no id. On
/// an archived file only the move that brings the record back is named;
/// its other findings wait for the record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Repair {
    /// Erase the optional field on the finding's line.
    Clear(&'static str),
    /// Erase the dependency edge on the finding's line.
    Unblock(String),
    /// Erase the hold the finding's line is half of.
    Unhold,
    /// File the settled record where it belongs.
    Archive,
    /// Bring the record back where the verbs can reach it.
    Restore,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, priority: Option<u8>, created: &str, title: &str) -> ReadyTask {
        ReadyTask {
            id: id.to_owned(),
            priority,
            created: created.to_owned(),
            title: title.to_owned(),
        }
    }

    #[test]
    fn a_row_derives_its_age_and_quotes_a_delimited_title() {
        let rows = [row(
            "task.demo",
            None,
            "2026-08-24",
            "Degrades sections, keeps Budget",
        )];
        let today_day = date::day_number("2026-08-28").unwrap();
        assert_eq!(
            ReadyTask::table(&rows, 1, today_day),
            "ready[1]{id,priority,age,title}:\n  task.demo,-,4d,\"Degrades sections, keeps Budget\"\n"
        );
    }

    #[test]
    fn an_unreadable_created_date_counts_as_today() {
        let rows = [row("task.demo", Some(1), "", "A demo record")];
        assert_eq!(
            ReadyTask::table(&rows, 1, 20_000),
            "ready[1]{id,priority,age,title}:\n  task.demo,1,0d,A demo record\n"
        );
    }

    /// A clock behind the notebook's own dates would otherwise age a record
    /// backwards; nothing is younger than new.
    #[test]
    fn a_record_created_after_today_is_no_age_at_all() {
        let rows = [row("task.demo", None, "2026-08-30", "A demo record")];
        let today_day = date::day_number("2026-08-28").unwrap();
        assert_eq!(
            ReadyTask::table(&rows, 1, today_day),
            "ready[1]{id,priority,age,title}:\n  task.demo,-,0d,A demo record\n"
        );
    }

    /// A terminal obeys the escape sequences in a title, so a row must not
    /// carry one: `\r` alone reprints the line as another record's row.
    #[test]
    fn a_control_character_in_a_title_is_escaped_into_its_own_cell() {
        let rows = [row(
            "task.demo",
            None,
            "2026-08-28",
            "Harmless\u{1b}[2K\rShipped",
        )];
        let today_day = date::day_number("2026-08-28").unwrap();
        assert_eq!(
            ReadyTask::table(&rows, 1, today_day),
            "ready[1]{id,priority,age,title}:\n  task.demo,-,0d,\"Harmless\\u001b[2K\\rShipped\"\n"
        );
    }
}
