//! The values a verb answers with: the vocabulary every host renders.
//!
//! They sit below the Notebook and below every derivation over it, so a
//! surface that computes one — Debt, Status, the encoder — never reaches
//! up into the module that hands it out.

use crate::finding::Finding;
use crate::record::{Record, RecordType};
use crate::resolve::path_stem;

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
    pub title: String,
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
    pub entry: String,
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
    let mut seen: Vec<&str> = Vec::new();
    blockers.iter().filter_map(move |blocker| {
        let carrier = blocker.carrier.as_str();
        (!seen.contains(&carrier)).then(|| {
            seen.push(carrier);
            carrier
        })
    })
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

/// One type's slice of the overview page.
#[derive(Debug, PartialEq, Eq)]
pub struct TypeSection {
    pub record_type: RecordType,
    pub rows: Vec<ListedRecord>,
}

/// Live records per type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub tasks: usize,
    pub decisions: usize,
    pub notes: usize,
    pub questions: usize,
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
}
