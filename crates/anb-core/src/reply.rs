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
/// claim two files after an interrupted archive move; leaving no trace
/// means leaving neither.
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

/// One Task on the map: what its tile says, and the record a reader opens
/// on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    pub id: String,
    /// The state its tile is coloured by, `invalid` for a record its own
    /// findings exclude — the listing's word, so one vocabulary answers
    /// every surface.
    pub state: String,
    pub archived: bool,
    pub title: Option<String>,
    /// Where this hub stands, when the Task is one.
    pub epic: Option<Epic>,
    pub blocked_by: Vec<String>,
    pub origin: Option<String>,
    /// The envelope in file order and the body, for the record a reader
    /// opens on a tile rather than for the tile.
    pub fields: Vec<(String, String)>,
    pub body: String,
    pub mentions: Vec<String>,
}

/// Which Tasks a map is asked for. Every narrowing is a predicate over the
/// same notebook, so asking for two asks for the intersection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphSlice {
    /// One epic's scope: the hub, what it waits on, and what was born
    /// inside it.
    pub hub: Option<String>,
    /// What can be started now.
    pub ready_only: bool,
    /// One record and the graph around it.
    pub focus: Option<Focus>,
    /// Whether archived Tasks are on the map. Most of what a long-lived
    /// notebook holds is finished, and drawing all of it buries the work in
    /// flight, so the archive stays off until it is asked for.
    pub archive: bool,
}

/// A record and how far around it the map reaches, counted in edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Focus {
    pub id: String,
    pub depth: usize,
}

/// The map one call asked for: every Task the slice reaches, each with the
/// edges it draws.
#[derive(Debug, PartialEq, Eq)]
pub struct Graph {
    /// What was asked for, carried back so a map says which slice it is.
    pub slice: GraphSlice,
    pub nodes: Vec<GraphNode>,
}

impl Graph {
    /// The edges the map draws, each running the way work becomes possible:
    /// out of what must settle first, into what waits on it or was born
    /// from it. An edge whose far end is off the map is not one, since a
    /// line has to land on a tile.
    #[must_use]
    pub fn edges(&self) -> Vec<GraphEdge<'_>> {
        let on_the_map: BTreeSet<&str> = self.nodes.iter().map(|node| node.id.as_str()).collect();
        let mut edges = Vec::new();
        for node in &self.nodes {
            for blocker in &node.blocked_by {
                if on_the_map.contains(blocker.as_str()) {
                    edges.push(GraphEdge {
                        from: blocker,
                        to: &node.id,
                        kind: EdgeKind::BlockedBy,
                    });
                }
            }
            if let Some(origin) = node.origin.as_deref()
                && on_the_map.contains(origin)
            {
                edges.push(GraphEdge {
                    from: origin,
                    to: &node.id,
                    kind: EdgeKind::Origin,
                });
            }
        }
        edges
    }

    /// How many lines meet at each tile. A web reads by weight, and weight
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

/// One line on the map, from what must settle first to what waits on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphEdge<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub kind: EdgeKind,
}

/// Which of the two edges a record draws: what it waits on, and what it was
/// born from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    BlockedBy,
    Origin,
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
/// a record in the archive, in the wrong directory, or under a filename
/// that is no id.
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
