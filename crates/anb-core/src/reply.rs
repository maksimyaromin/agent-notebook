//! The values returned by notebook operations, independent of output encoding.
//!
//! They sit below the Notebook and below every derivation over it, so a
//! surface that computes one — Debt, Status, Check — never reaches up into
//! the module that hands it out.

use crate::finding::Finding;
use crate::record::{Record, RecordType};
use crate::request::GraphSlice;
use std::collections::{BTreeMap, BTreeSet};

/// A checked batch of record files, either previewed or written.
#[derive(Debug, PartialEq, Eq)]
pub struct FileBatch {
    pub paths: Vec<String>,
    pub unchanged: usize,
    pub check: bool,
    /// The migration journal containing the original bytes, if one was written.
    pub backup: Option<String>,
}

/// Maintained knowledge and the invalid files excluded from recall.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Knowledge {
    pub records: Vec<Memory>,
    pub invalid: Vec<String>,
}

/// A recalled record. The source notebook supplies its audience; `by` is
/// provenance. Bodies and external links retain the recorded evidence.
#[derive(Debug, PartialEq, Eq)]
pub struct Memory {
    pub id: String,
    pub path: String,
    pub record_type: RecordType,
    pub kind: Option<String>,
    pub title: String,
    pub body: String,
    pub by: Option<String>,
    pub links: Vec<String>,
    pub related: bool,
}

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
    /// The Decision or Task a Question resolved into, when it did.
    pub resolved_by: Option<String>,
    /// Unresolved citations in the outcome or reason written by this close.
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

/// The people a record names: who wrote it, on a Task who holds it, and
/// whom it is addressed to. Whose the record is, is
/// [`Record::belongs_to`]'s to say, and whom it waits on
/// [`Record::waits_on`]'s.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attribution {
    pub by: Option<String>,
    pub taken_by: Option<String>,
    pub to: Option<String>,
}

impl Attribution {
    pub(crate) fn of(record: &Record) -> Attribution {
        Attribution {
            by: record.file().field("by").map(str::to_owned),
            taken_by: record.taken_by().map(str::to_owned),
            to: record.addressee().map(str::to_owned),
        }
    }
}

/// One ready Task. It carries `created`, not an age: the Core holds no
/// clock, so "how old" is the caller's derivation from its own today.
#[derive(Debug, PartialEq, Eq)]
pub struct ReadyTask {
    pub id: String,
    pub priority: Option<u8>,
    pub created: String,
    pub attribution: Attribution,
    pub title: String,
}

/// A record created, with the computed consequences its reply must not
/// bury.
#[derive(Debug, PartialEq, Eq)]
pub struct Created {
    pub id: String,
    pub path: String,
    pub superseded: Option<String>,
    pub dangling_mentions: Vec<String>,
}

/// A log entry appended; `already` marks the replay of the trail's tail.
#[derive(Debug, PartialEq, Eq)]
pub struct Commented {
    pub id: String,
    pub already: bool,
    pub dangling_mentions: Vec<String>,
}

/// One record standing in the way of a delete, and the edge that holds
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
pub struct Deleted {
    pub id: String,
    pub paths: Vec<String>,
}

/// A record moved into the archive; `already` marks the replay.
#[derive(Debug, PartialEq, Eq)]
pub struct Archived {
    pub id: String,
    pub from: String,
    pub to: String,
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
    pub attribution: Attribution,
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
    /// The records this one links, each under the kind its `link` line
    /// gives the relation.
    pub links: Vec<(String, String)>,
    /// The envelope in file order and the body, for the record a reader
    /// reads rather than for the node.
    pub fields: Vec<(String, String)>,
    pub body: String,
    pub mentions: Vec<String>,
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
            |from: &'a str, to: &'a str, kind: EdgeKind<'a>, edges: &mut Vec<GraphEdge<'a>>| {
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
            for (kind, linked) in &node.links {
                draw(&node.id, linked, EdgeKind::Link(kind), &mut edges);
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
    pub kind: EdgeKind<'a>,
}

/// How two records are related: by what one waits on, by what it was born
/// from, by a link one declares to the other, or by one naming the other
/// in its prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind<'a> {
    BlockedBy,
    Origin,
    /// A `link` line naming a record, under the kind the line gives it:
    /// the relation a record declares to what it belongs to, follows or
    /// cites, in the notebook's own vocabulary.
    Link(&'a str),
    /// One record naming another in its body: the web a notebook weaves
    /// beside the work it tracks.
    Mentions,
}

impl<'a> EdgeKind<'a> {
    /// The words of the relations the notebook draws itself. A link kind
    /// spelled like one would be read as that relation, so no link may
    /// carry one.
    pub(crate) const DRAWN_WORDS: [&'static str; 3] = ["waits", "born", "mentions"];

    /// The one word every surface prints for the edge: the notebook's for
    /// the three relations it draws itself, the link's own for a link.
    #[must_use]
    pub fn word(self) -> &'a str {
        match self {
            EdgeKind::BlockedBy => EdgeKind::DRAWN_WORDS[0],
            EdgeKind::Origin => EdgeKind::DRAWN_WORDS[1],
            EdgeKind::Link(kind) => kind,
            EdgeKind::Mentions => EdgeKind::DRAWN_WORDS[2],
        }
    }
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

/// One record read whole: the envelope as it stands, the body, the two
/// derived Mention blocks, and the live records whose `link` lines name
/// it.
#[derive(Debug, PartialEq, Eq)]
pub struct View {
    pub id: String,
    pub path: String,
    pub archived: bool,
    pub fields: Vec<(String, String)>,
    pub body: String,
    pub mentions: Vec<String>,
    pub mentioned_by: Vec<String>,
    /// Each as the link's kind and the record carrying it, by kind and
    /// then by id, so a reader sees one relation's members together.
    pub linked_by: Vec<(String, String)>,
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
    /// Erase the link on the finding's line.
    Unlink(String),
    /// Erase the hold the finding's line is half of.
    Unhold,
    /// File the settled record where it belongs.
    Archive,
    /// Bring the record back where the verbs can reach it.
    Restore,
}
