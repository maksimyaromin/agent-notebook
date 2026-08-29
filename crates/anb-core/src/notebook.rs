//! The Notebook: the records under one root, read and mutated through
//! Storage.
//!
//! Write-time invariants live here, and they bind every author equally: a
//! declared supersession writes the back-pointer and flips the victim, a
//! Question closes only by routing or an explicit reasoned drop, a close
//! carries its proof. Every mutation is idempotent — a replayed call answers
//! `already: true` and leaves every byte of every file unchanged.
//!
//! The mutation gate holds a record's own error findings and its dangling
//! references against it; findings that need a second record — a broken
//! supersession pair, a multi-file dependency cycle — are `check`'s alone
//! and freeze nothing. The one admission through the gate: `unblock` runs
//! over errors sitting on the very `blocked-by` lines it erases, so a
//! corrupted edge never freezes its own repair.

use crate::config::{CONFIG_PATH, Config};
use crate::debt::{self, Cited, DebtSources};
use crate::finding::{Finding, FindingCode, Severity};
use crate::grammar::{self, RecordFile, Residence};
use crate::graph::{TaskGraph, TaskNode};
use crate::mention;
use crate::record::{Record, RecordType, TaskAction, TaskState, Transition, not_utf8_finding};
use crate::status::{self, ActiveTask, Budget, Counts, Status, StatusInputs, StatusRule};
use crate::storage::{Storage, StorageError};
use std::collections::{BTreeMap, BTreeSet};

/// The envelope keys whose values point at other records.
pub(crate) const REF_KEYS: [&str; 5] = [
    "from",
    "supersedes",
    "superseded-by",
    "routed-to",
    "blocked-by",
];

/// The record a `link` line points at, if it points at one at all.
///
/// A link is `<kind> <target>`, and its target is a pull request, a commit,
/// a path, or — since a close may carry its report as a Note — a record id.
/// Only the id-shaped target names a record; nothing else can be resolved,
/// and nothing else may be mistaken for a reference.
pub(crate) fn linked_record(link: &str) -> Option<&str> {
    let (_, target) = link.split_once(char::is_whitespace)?;
    let target = target.trim();
    grammar::id_error(target).is_none().then_some(target)
}

/// A record to be created; `id: None` mints one from the title.
pub struct Draft {
    pub record_type: RecordType,
    pub title: String,
    pub id: Option<String>,
    pub kind: Option<String>,
    pub by: Option<String>,
    pub via: Option<String>,
    pub from: Option<String>,
    pub tags: Vec<String>,
    pub links: Vec<Link>,
    pub supersedes: Option<String>,
    pub priority: Option<u8>,
    pub body: String,
}

impl Draft {
    #[must_use]
    pub fn new(record_type: RecordType, title: &str) -> Self {
        Draft {
            record_type,
            title: title.to_owned(),
            id: None,
            kind: None,
            by: None,
            via: None,
            from: None,
            tags: Vec::new(),
            links: Vec::new(),
            supersedes: None,
            priority: None,
            body: String::new(),
        }
    }
}

pub struct Link {
    pub kind: String,
    pub target: String,
}

/// The auditable evidence a close carries. `Waived` is the explicit
/// override: the caller states there is no proof rather than omitting it.
pub enum Proof {
    Pr(String),
    Sha(String),
    Report(String),
    /// A Note holding the report, so the proof travels with the notebook.
    Note(String),
    Waived,
}

impl Proof {
    fn link_value(&self) -> Option<String> {
        match self {
            Proof::Pr(target) => Some(format!("pr {target}")),
            Proof::Sha(target) => Some(format!("sha {target}")),
            Proof::Report(target) => Some(format!("report {target}")),
            Proof::Note(target) => Some(format!("note {target}")),
            Proof::Waived => None,
        }
    }
}

/// A state move; on a replay `already` is true and `from` equals `to`.
#[derive(Debug, PartialEq, Eq)]
pub struct Transitioned {
    pub id: String,
    pub from: &'static str,
    pub to: &'static str,
    pub already: bool,
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

/// A report landed in the notebook, and what its body cited.
struct IngestedReport {
    id: String,
    dangling_mentions: Vec<String>,
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

/// One row of the live listing; the id carries the type. An invalid
/// record's state shows as `invalid` — visible, never silently dropped.
#[derive(Debug, PartialEq, Eq)]
pub struct ListedRecord {
    pub id: String,
    pub state: String,
    pub priority: Option<u8>,
    pub title: Option<String>,
}

/// A record removed as a mistake, and every file that is gone. One id can
/// claim two files after an interrupted archive move; leaving no trace
/// means leaving neither.
#[derive(Debug, PartialEq, Eq)]
pub struct Expunged {
    pub id: String,
    pub paths: Vec<String>,
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

/// A record moved into the archive; `already` marks the replay.
#[derive(Debug, PartialEq, Eq)]
pub struct Archived {
    pub id: String,
    pub from: String,
    pub to: String,
    pub already: bool,
}

/// The deliberate corrections `edit` applies to a live record's own fields.
/// State, id, and the envelope dates stay the commands' territory.
#[derive(Default)]
pub struct Edit {
    pub title: Option<String>,
    pub body: Option<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub from: Option<String>,
    pub priority: Option<u8>,
    pub review_by: Option<String>,
}

impl Edit {
    fn changes_nothing(&self) -> bool {
        self.title.is_none()
            && self.body.is_none()
            && self.add_tags.is_empty()
            && self.remove_tags.is_empty()
            && self.from.is_none()
            && self.priority.is_none()
            && self.review_by.is_none()
    }
}

/// An edit applied; `changed` names what moved in the file, so an empty
/// one is the replay whose write was skipped whole.
#[derive(Debug, PartialEq, Eq)]
pub struct Edited {
    pub id: String,
    pub changed: Vec<&'static str>,
    pub dangling_mentions: Vec<String>,
}

/// One type's slice of the overview page.
#[derive(Debug, PartialEq, Eq)]
pub struct TypeSection {
    pub record_type: RecordType,
    pub rows: Vec<ListedRecord>,
}

/// The whole notebook as one page: every live record grouped by type, the
/// archive as counts — history is recoverable, not re-read.
#[derive(Debug, PartialEq, Eq)]
pub struct Overview {
    pub live: Counts,
    pub sections: Vec<TypeSection>,
    pub archived: Counts,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFinding {
    pub path: String,
    pub finding: Finding,
}

/// One variant per outcome a caller tells apart; the CLI turns each into a
/// structured `error[code]` payload with computed `try:` lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotebookError {
    /// No live or archived record carries this id.
    UnknownId {
        id: String,
    },
    /// The record exists only in the archive; archived records are read,
    /// never mutated in place.
    Archived {
        id: String,
    },
    /// The record carries error findings, which exclude it from mutation.
    InvalidRecord {
        path: String,
        findings: Vec<Finding>,
    },
    /// The id names a type this command does not act on.
    WrongType {
        id: String,
        expected: String,
    },
    /// The record's state does not allow this move; `valid` names the
    /// commands it does allow.
    InvalidTransition {
        id: String,
        state: String,
        valid: Vec<&'static str>,
    },
    /// An argument is malformed before any record is touched.
    InvalidArgument {
        reason: String,
    },
    /// The id is taken; ids are never reused, archive included.
    DuplicateId {
        id: String,
        holder: String,
    },
    /// An expunge would leave the notebook pointing at nothing. Every
    /// blocker is named, since repairing them is the whole path forward.
    StillReferenced {
        id: String,
        blockers: Vec<Blocker>,
    },
    /// A reference argument names a record that does not exist.
    DanglingRef {
        field: &'static str,
        target: String,
    },
    /// The supersession victim cannot die by supersession.
    CannotSupersede {
        id: String,
        reason: String,
    },
    /// The edge would close a dependency cycle; `chain` walks it,
    /// first and last the same Task.
    WouldCycle {
        chain: Vec<String>,
    },
    Storage(StorageError),
}

impl std::fmt::Display for NotebookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotebookError::UnknownId { id } => write!(f, "no record `{id}`"),
            NotebookError::Archived { id } => write!(f, "`{id}` is archived"),
            NotebookError::InvalidRecord { path, findings } => {
                write!(f, "{path} is invalid ({} findings)", findings.len())
            }
            NotebookError::WrongType { id, expected } => {
                write!(f, "`{id}` is not {expected}")
            }
            NotebookError::InvalidTransition { id, state, valid } => {
                if valid.is_empty() {
                    write!(f, "`{id}` is {state}; no move is valid from `{state}`")
                } else {
                    write!(f, "`{id}` is {state}; valid: {}", valid.join(", "))
                }
            }
            NotebookError::InvalidArgument { reason } => f.write_str(reason),
            NotebookError::DuplicateId { id, holder } => {
                write!(f, "`{id}` already exists at {holder}")
            }
            NotebookError::StillReferenced { id, blockers } => {
                let records = carriers_of(blockers).count();
                let unit = if records == 1 { "record" } else { "records" };
                write!(f, "`{id}` is still referenced by {records} {unit}")
            }
            NotebookError::DanglingRef { field, target } => {
                write!(f, "{field}: `{target}` names no record")
            }
            NotebookError::CannotSupersede { id, reason } => {
                write!(f, "cannot supersede `{id}`: {reason}")
            }
            NotebookError::WouldCycle { chain } => {
                write!(
                    f,
                    "the edge would close a dependency cycle: {}",
                    chain.join(" → ")
                )
            }
            NotebookError::Storage(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for NotebookError {}

impl From<StorageError> for NotebookError {
    fn from(error: StorageError) -> Self {
        NotebookError::Storage(error)
    }
}

pub struct Notebook<'a, S: Storage> {
    storage: &'a mut S,
}

impl<'a, S: Storage> Notebook<'a, S> {
    pub fn new(storage: &'a mut S) -> Self {
        Notebook { storage }
    }

    /// Read one record by id, live or archived.
    ///
    /// # Errors
    /// [`NotebookError::UnknownId`], [`NotebookError::InvalidArgument`] on a
    /// malformed id, or a storage failure.
    pub fn record(&self, id: &str) -> Result<Record, NotebookError> {
        parsed_type(id)?;
        match self.holder_path(id)? {
            Some(path) => {
                let text = self.storage.read(&path)?;
                Ok(Record::parse(&path, &text))
            }
            None => Err(NotebookError::UnknownId { id: id.to_owned() }),
        }
    }

    /// Verify the whole notebook: every record's own findings plus the
    /// cross-record rules — duplicate ids, dangling references, supersession
    /// pairs, routing threads. Errors first, then by file and line.
    ///
    /// # Errors
    /// A storage failure.
    pub fn check(&self) -> Result<Vec<FileFinding>, NotebookError> {
        let records = self.read_records()?;
        let resolvable = resolvable_by_id(&records);
        let by_stem: BTreeMap<&str, &Record> = records
            .iter()
            .map(|record| (path_stem(record.path()), record))
            .collect();

        let mut located = Vec::new();
        for record in &records {
            for finding in record.findings() {
                located.push(FileFinding {
                    path: record.path().to_owned(),
                    finding: finding.clone(),
                });
            }
            check_refs(record, &resolvable, &mut located);
            check_supersession_pair(record, &by_stem, &mut located);
        }
        check_duplicate_ids(&records, &mut located);
        check_dep_cycles(&records, &by_stem, &mut located);
        self.check_config(&mut located)?;

        located.sort_by(|left, right| finding_order(left).cmp(&finding_order(right)));
        Ok(located)
    }

    /// The config file's findings, against its own path; the same closed
    /// finding set covers records and config alike.
    fn check_config(&self, out: &mut Vec<FileFinding>) -> Result<(), NotebookError> {
        let text = match self.storage.read(CONFIG_PATH) {
            Ok(text) => text,
            Err(StorageError::NotFound { .. }) => return Ok(()),
            Err(StorageError::NotUtf8 { .. }) => {
                out.push(FileFinding {
                    path: CONFIG_PATH.to_owned(),
                    finding: not_utf8_finding(),
                });
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        for finding in Config::parse(&text).findings() {
            out.push(FileFinding {
                path: CONFIG_PATH.to_owned(),
                finding: finding.clone(),
            });
        }
        Ok(())
    }

    /// The dispatch queue: open, unblocked, unheld Tasks, the most
    /// urgent first. Invalid records are excluded, as from every derived
    /// query; `check` names them.
    ///
    /// # Errors
    /// A storage failure.
    pub fn ready(&self) -> Result<Vec<ReadyTask>, NotebookError> {
        let records = self.read_records()?;
        Ok(ready_rows(&records, &resolvable_by_id(&records)))
    }

    /// The session-start dashboard under `budget`, gated: one quiet line
    /// when the notebook carries no signal, the full budgeted composite
    /// otherwise. The caller resolves `budget` — a CLI flag outranks the
    /// config key, which defaults to 1500.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on a malformed `today`, or a
    /// storage failure.
    pub fn status(&self, today: &str, budget: Budget) -> Result<Status, NotebookError> {
        let today_day = guarded_day(today)?;
        let thresholds = self.config()?.debt_thresholds();
        let records = self.read_records()?;
        let resolvable = resolvable_by_id(&records);
        let live_valid: Vec<&Record> = records
            .iter()
            .filter(|record| !is_archived(record.path()) && !debt::is_excluded(record, &resolvable))
            .collect();

        let sources = DebtSources {
            records: &records,
            resolvable: &resolvable,
            today_day,
        };
        let inputs = StatusInputs {
            counts: live_counts(&records),
            in_flight: in_flight_tasks(&live_valid),
            review: review_tasks(&live_valid),
            rules: standing_rules(&live_valid),
            ready: ready_rows(&records, &resolvable),
            debt: debt::signals(&sources, &thresholds),
            today_day,
        };
        Ok(status::assemble(inputs, budget))
    }

    /// The live listing, in type-major file order. An invalid record is a
    /// row of state `invalid` — excluded from mutation and derived queries,
    /// never from sight; `check` names its findings.
    ///
    /// # Errors
    /// A storage failure.
    pub fn list(&self) -> Result<Vec<ListedRecord>, NotebookError> {
        let records = self.read_records()?;
        let resolvable = resolvable_by_id(&records);
        Ok(records
            .iter()
            .filter(|record| !is_archived(record.path()))
            .map(|record| listed_row(record, &resolvable))
            .collect())
    }

    /// Find records by case-insensitive substring over id, title, tags, and
    /// body — live and archived alike: the archive is history, and history
    /// is findable. Rows share the listing shape.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on an empty query, or a storage
    /// failure.
    pub fn search(&self, query: &str) -> Result<Vec<ListedRecord>, NotebookError> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Err(NotebookError::InvalidArgument {
                reason: "search: the query must not be empty".to_owned(),
            });
        }
        let records = self.read_records()?;
        let resolvable = resolvable_by_id(&records);
        Ok(records
            .iter()
            .filter(|record| matches_query(record, &needle))
            .map(|record| listed_row(record, &resolvable))
            .collect())
    }

    /// The whole notebook as one page: every live record grouped by type,
    /// deliberately unbounded — this is the read-it-whole surface — with the
    /// archive reduced to counts.
    ///
    /// # Errors
    /// A storage failure.
    pub fn overview(&self) -> Result<Overview, NotebookError> {
        let records = self.read_records()?;
        let resolvable = resolvable_by_id(&records);
        let sections = RecordType::ALL
            .into_iter()
            .map(|record_type| TypeSection {
                record_type,
                rows: records
                    .iter()
                    .filter(|record| sits_in(record, record_type, Residence::Live))
                    .map(|record| listed_row(record, &resolvable))
                    .collect(),
            })
            .collect();
        Ok(Overview {
            live: live_counts(&records),
            sections,
            archived: archived_counts(&records),
        })
    }

    /// Read one record whole, live or archived: every envelope field in file
    /// order, the body, the ids its body cites, and the live records whose
    /// bodies cite it. A record's own id never enters its blocks. Reading
    /// never gates: an invalid record shows as it stands.
    ///
    /// # Errors
    /// See [`Notebook::record`].
    pub fn view(&self, id: &str) -> Result<View, NotebookError> {
        let record = self.record(id)?;
        let records = self.read_records()?;
        let mentions = mention::mentions(record.file().body())
            .into_iter()
            .filter(|target| *target != id)
            .map(str::to_owned)
            .collect();
        let mentioned_by = records
            .iter()
            .filter(|other| !is_archived(other.path()) && path_stem(other.path()) != id)
            .filter(|other| mention::mentions(other.file().body()).contains(&id))
            .map(|other| path_stem(other.path()).to_owned())
            .collect();
        Ok(View {
            id: id.to_owned(),
            path: record.path().to_owned(),
            archived: is_archived(record.path()),
            fields: record
                .file()
                .fields()
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect(),
            body: record.file().body().to_owned(),
            mentions,
            mentioned_by,
        })
    }

    /// The notebook config; an absent file is the defaults.
    ///
    /// # Errors
    /// A storage failure.
    pub fn config(&self) -> Result<Config, NotebookError> {
        match self.storage.read(CONFIG_PATH) {
            Ok(text) => Ok(Config::parse(&text)),
            Err(StorageError::NotFound { .. }) => Ok(Config::default()),
            Err(error) => Err(error.into()),
        }
    }

    /// Create a record, minting an id from the title unless one is given.
    /// A draft declaring `supersedes` performs the whole supersession: the
    /// new record is written first, then the victim gains the back-pointer
    /// and flips to its superseded state — a rule recorded as replaced can
    /// never be read as live. A Decision declaring none is answered with
    /// the standing Decisions it may conflict with — a nudge in the reply,
    /// never a block.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on a malformed draft,
    /// [`NotebookError::DanglingRef`] when `from` or `supersedes` names no
    /// record, [`NotebookError::CannotSupersede`] when the victim cannot die
    /// by supersession, [`NotebookError::DuplicateId`] on a taken id, or a
    /// storage failure.
    pub fn create(&mut self, draft: &Draft, today: &str) -> Result<Created, NotebookError> {
        guard_today(today)?;
        validate_draft(draft)?;
        if let Some(origin) = &draft.from {
            self.guard_ref_exists("from", origin)?;
        }
        let victim = self.guard_supersession(draft)?;

        let records = self.read_records()?;
        let id = resolve_draft_id(draft, &id_claims(&records))?;
        let may_conflict = conflict_candidates(draft, &records);
        let path = record_path(&id, draft.record_type, false);
        self.storage
            .write(&path, &render_draft(draft, &id, today))?;

        if let Some(victim) = victim {
            self.flip_victim(victim, &id, today)?;
        }

        // Probed after the write, so a body citing its own record resolves.
        let dangling_mentions = self.dangling_mentions(&draft.body)?;
        Ok(Created {
            id,
            path,
            superseded: draft.supersedes.clone(),
            may_conflict,
            dangling_mentions,
        })
    }

    /// The victim's half of a supersession: the back-pointer and the flip
    /// to its dead state.
    fn flip_victim(
        &mut self,
        victim: Victim,
        superseder: &str,
        today: &str,
    ) -> Result<(), NotebookError> {
        let mut file = victim.record.into_file();
        file.set_field("superseded-by", superseder);
        file.set_field("state", victim.dead_state);
        file.set_field("updated", today);
        self.storage.write(&victim.path, &file.render())?;
        Ok(())
    }

    /// `open → active`.
    ///
    /// # Errors
    /// See [`Notebook::close`]; `start` carries no proof.
    pub fn start(&mut self, id: &str, today: &str) -> Result<Transitioned, NotebookError> {
        self.task_transition(id, TaskAction::Start, today, |_| {})
    }

    /// `active → review`: hand the work to a human for acceptance.
    ///
    /// # Errors
    /// See [`Notebook::close`]; `submit` carries no proof.
    pub fn submit(&mut self, id: &str, today: &str) -> Result<Transitioned, NotebookError> {
        self.task_transition(id, TaskAction::Submit, today, |_| {})
    }

    /// `active | review → closed`, stamping the close date and the proof
    /// link. Replies name the still-open Questions born from this Task, so
    /// a close never buries deferred findings.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on a malformed proof,
    /// [`NotebookError::InvalidTransition`] naming the valid commands,
    /// [`NotebookError::WrongType`], [`NotebookError::UnknownId`],
    /// [`NotebookError::Archived`], [`NotebookError::InvalidRecord`], or a
    /// storage failure.
    pub fn close(&mut self, id: &str, proof: &Proof, today: &str) -> Result<Closed, NotebookError> {
        guard_proof(proof)?;
        let transition = self.task_transition(id, TaskAction::Close, today, |file| {
            file.set_field("closed", today);
            if let Some(link) = proof.link_value() {
                file.append_field("link", &link);
            }
        })?;
        let records = self.read_records()?;
        let resolvable = resolvable_by_id(&records);
        Ok(Closed {
            transition,
            open_questions: open_questions_from(&records, &resolvable, id),
            unblocked: unblocked_by_close(&records, &resolvable, id),
            report_note: None,
            dangling_mentions: Vec::new(),
        })
    }

    /// Close carrying `report` as its proof: the text becomes a Note born
    /// from the Task, and the close links that Note. One motion, and the
    /// proof travels with the notebook instead of pointing out of it.
    ///
    /// The Note is minted only when the close is a real move, so a replay
    /// creates nothing; the one Note this call will ever reuse is the one
    /// its own interrupted run left behind, recognised by
    /// [`Notebook::standing_report`].
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on an empty report — a proof with
    /// nothing in it is not a proof — plus the refusals of
    /// [`Notebook::close`] and the draft refusals of [`Notebook::create`].
    pub fn close_with_report(
        &mut self,
        id: &str,
        report: &str,
        by: Option<&str>,
        today: &str,
    ) -> Result<Closed, NotebookError> {
        if report.trim().is_empty() {
            return Err(NotebookError::InvalidArgument {
                reason: "note: the report is empty — a close carries a proof or waives one"
                    .to_owned(),
            });
        }
        let task = self.load_live(id, &[RecordType::Task])?;
        let moves = TaskState::from_word(task.state_word()).is_some_and(|state| {
            matches!(
                state.transition(TaskAction::Close),
                Ok(Transition::Move { .. })
            )
        });
        if !moves {
            // Nothing will be written, so nothing may be minted. `close`
            // owns the transition rules: it answers a replay with `already`
            // and anything else with its refusal, and the waiver claims
            // nothing because no proof is ever reached.
            return self.close(id, &Proof::Waived, today);
        }
        let title = report_note_title(task.record.file().field("title").unwrap_or(id));
        let note = self.ingest_report(id, &title, report, by, today)?;
        let mut closed = self.close(id, &Proof::Note(note.id.clone()), today)?;
        closed.report_note = Some(note.id);
        closed.dangling_mentions = note.dangling_mentions;
        Ok(closed)
    }

    /// The Note holding `report`, recovered if this call already wrote it,
    /// created otherwise.
    fn ingest_report(
        &mut self,
        origin: &str,
        title: &str,
        report: &str,
        by: Option<&str>,
        today: &str,
    ) -> Result<IngestedReport, NotebookError> {
        if let Some(id) = self.standing_report(origin, report)? {
            return Ok(IngestedReport {
                id,
                dangling_mentions: Vec::new(),
            });
        }
        let mut draft = Draft::new(RecordType::Note, title);
        draft.from = Some(origin.to_owned());
        draft.by = by.map(str::to_owned);
        report.clone_into(&mut draft.body);
        let created = self.create(&draft, today)?;
        Ok(IngestedReport {
            id: created.id,
            dangling_mentions: created.dangling_mentions,
        })
    }

    /// The Note an interrupted [`Notebook::close_with_report`] left behind,
    /// if this is that call resuming.
    ///
    /// Only one shape can be that Note: live, born from this Task, and
    /// holding this very report. Each condition rules out a record that
    /// merely resembles it — an archived one is history a new close must
    /// not revive, and a differing body is an earlier report that a reopened
    /// Task is now replacing, which is why the id is never recomputed to
    /// find it.
    fn standing_report(&self, origin: &str, report: &str) -> Result<Option<String>, NotebookError> {
        let wanted = edited_body(report);
        Ok(self
            .records_of(RecordType::Note)?
            .into_iter()
            .find(|note| {
                !is_archived(note.path())
                    && note.origin() == Some(origin)
                    && note.file().body() == wanted
            })
            .map(|note| path_stem(note.path()).to_owned()))
    }

    /// `review → active`: the human returned the work; what to fix comes
    /// from them, not from the tool.
    ///
    /// # Errors
    /// See [`Notebook::close`]; `return` carries no proof.
    pub fn return_task(&mut self, id: &str, today: &str) -> Result<Transitioned, NotebookError> {
        self.task_transition(id, TaskAction::Return, today, |_| {})
    }

    /// `closed → open`, dropping the close date; the proof links stay as
    /// history.
    ///
    /// # Errors
    /// See [`Notebook::close`]; `reopen` carries no proof.
    pub fn reopen(&mut self, id: &str, today: &str) -> Result<Transitioned, NotebookError> {
        self.task_transition(id, TaskAction::Reopen, today, |file| {
            file.remove_field("closed");
        })
    }

    /// Pause a Task deliberately, keeping its state. The reason is
    /// mandatory: an unreasoned hold is where work goes to rot.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on an empty reason or a malformed
    /// `until` date, plus the resolution errors of [`Notebook::close`].
    pub fn hold(
        &mut self,
        id: &str,
        reason: &str,
        until: Option<&str>,
        today: &str,
    ) -> Result<Held, NotebookError> {
        guard_today(today)?;
        let reason = reason.trim();
        guard_single_line("hold", reason)?;
        if reason.is_empty() {
            return Err(NotebookError::InvalidArgument {
                reason: "hold: the reason must not be empty".to_owned(),
            });
        }
        if let Some(until) = until
            && let Some(why) = grammar::date_error(until)
        {
            return Err(NotebookError::InvalidArgument {
                reason: format!("hold-until: {why}"),
            });
        }

        let loaded = self.load_live(id, &[RecordType::Task])?;
        if loaded.record.hold() == Some(reason) && loaded.record.hold_until() == until {
            return Ok(Held {
                id: id.to_owned(),
                already: true,
            });
        }
        let mut file = loaded.record.into_file();
        file.set_field("hold", reason);
        match until {
            Some(until) => {
                file.set_field("hold-until", until);
            }
            None => {
                file.remove_field("hold-until");
            }
        }
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Held {
            id: id.to_owned(),
            already: false,
        })
    }

    /// Resume a held Task: clears `hold` and `hold-until`.
    ///
    /// # Errors
    /// The resolution errors of [`Notebook::close`].
    pub fn unhold(&mut self, id: &str, today: &str) -> Result<Held, NotebookError> {
        guard_today(today)?;
        let loaded = self.load_live(id, &[RecordType::Task])?;
        if loaded.record.hold().is_none() && loaded.record.hold_until().is_none() {
            return Ok(Held {
                id: id.to_owned(),
                already: true,
            });
        }
        let mut file = loaded.record.into_file();
        file.remove_field("hold");
        file.remove_field("hold-until");
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Held {
            id: id.to_owned(),
            already: false,
        })
    }

    /// Append one dated log entry to a Task's body — the append-only
    /// progress trail, in the log convention `- <date> <author>: <text>`.
    /// The Task's state does not gate the verb: the trail may narrate a
    /// close as well as the work. Replaying the trail's tail answers
    /// `already: true` and changes no byte.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on an empty or multi-line entry or
    /// a multi-line author, [`NotebookError::WrongType`],
    /// [`NotebookError::UnknownId`], [`NotebookError::Archived`],
    /// [`NotebookError::InvalidRecord`], or a storage failure.
    pub fn comment(
        &mut self,
        id: &str,
        author: Option<&str>,
        text: &str,
        today: &str,
    ) -> Result<Commented, NotebookError> {
        guard_today(today)?;
        let text = text.trim();
        guard_single_line("comment", text)?;
        if text.is_empty() {
            return Err(NotebookError::InvalidArgument {
                reason: "comment: the text must not be empty".to_owned(),
            });
        }
        let author = author.map(str::trim).filter(|name| !name.is_empty());
        if let Some(author) = author {
            guard_single_line("author", author)?;
        }

        let entry = format!("- {today} {}: {text}", author.unwrap_or("-"));
        let loaded = self.load_live(id, &[RecordType::Task])?;
        // A replay carries the nudge too: the entry is the trail's tail, so
        // its citations stand in the body either way.
        let dangling_mentions = self.dangling_mentions(text)?;
        if last_log_line(&loaded.record).as_deref() == Some(entry.as_str()) {
            return Ok(Commented {
                id: id.to_owned(),
                entry,
                already: true,
                dangling_mentions,
            });
        }
        let mut file = loaded.record.into_file();
        file.append_body(&entry);
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Commented {
            id: id.to_owned(),
            entry,
            already: false,
            dangling_mentions,
        })
    }

    /// Write a dependency edge: this Task waits on `on`. An edge that would
    /// close a cycle is rejected with the cycle walked in full, so
    /// `ready` can never silently empty forever.
    ///
    /// # Errors
    /// [`NotebookError::WrongType`] when `on` is not a task,
    /// [`NotebookError::DanglingRef`] when it does not exist,
    /// [`NotebookError::WouldCycle`] naming the cycle, plus the resolution
    /// errors of [`Notebook::close`].
    pub fn block(&mut self, id: &str, on: &str, today: &str) -> Result<Edged, NotebookError> {
        guard_today(today)?;
        if parsed_type(on)? != RecordType::Task {
            return Err(NotebookError::WrongType {
                id: on.to_owned(),
                expected: "a task".to_owned(),
            });
        }
        self.guard_ref_exists("blocked-by", on)?;
        let loaded = self.load_live(id, &[RecordType::Task])?;
        if edge_exists(&loaded.record, on) {
            return Ok(Edged {
                id: id.to_owned(),
                on: on.to_owned(),
                already: true,
            });
        }
        if let Some(chain) = self.would_cycle(id, on)? {
            return Err(NotebookError::WouldCycle { chain });
        }
        let mut file = loaded.record.into_file();
        file.append_field("blocked-by", on);
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Edged {
            id: id.to_owned(),
            on: on.to_owned(),
            already: false,
        })
    }

    /// Erase a dependency edge; the reverse of [`Notebook::block`], and the
    /// repair for a corrupted one. `on` may be any well-formed id — the edge
    /// being erased may be exactly the wrong-typed target `check` named —
    /// and errors sitting on the record's own `blocked-by` lines do not
    /// freeze the verb that erases them. An edge hand-edited into duplicates
    /// is erased whole, so the replay stays true.
    ///
    /// # Errors
    /// The resolution errors of [`Notebook::close`].
    pub fn unblock(&mut self, id: &str, on: &str, today: &str) -> Result<Edged, NotebookError> {
        guard_today(today)?;
        parsed_type(on)?;
        let loaded = self.load_live_admitting(id, &[RecordType::Task], edge_borne)?;
        if !edge_exists(&loaded.record, on) {
            return Ok(Edged {
                id: id.to_owned(),
                on: on.to_owned(),
                already: true,
            });
        }
        let mut file = loaded.record.into_file();
        file.remove_field_value("blocked-by", on);
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Edged {
            id: id.to_owned(),
            on: on.to_owned(),
            already: false,
        })
    }

    /// Route a Question into the Decision or Task its answer became:
    /// `open → routed`, writing `routed-to` in the same move so the record
    /// can never lose the thread of what closed it.
    ///
    /// # Errors
    /// [`NotebookError::WrongType`] when `to` is not a decision or a task,
    /// [`NotebookError::DanglingRef`] when it does not exist,
    /// [`NotebookError::InvalidTransition`] from a settled state, plus the
    /// resolution errors of [`Notebook::close`].
    pub fn route(
        &mut self,
        id: &str,
        to: &str,
        today: &str,
    ) -> Result<Transitioned, NotebookError> {
        guard_today(today)?;
        let target_type = parsed_type(to)?;
        if !matches!(target_type, RecordType::Decision | RecordType::Task) {
            return Err(NotebookError::WrongType {
                id: to.to_owned(),
                expected: "a decision or a task".to_owned(),
            });
        }
        self.guard_ref_exists("routed-to", to)?;

        let loaded = self.load_live(id, &[RecordType::Question])?;
        let state = loaded.state_word();
        if state == "routed" && loaded.record.routed_to() == Some(to) {
            return Ok(replayed(id, "routed"));
        }
        if state != "open" {
            return Err(settled_question(id, state));
        }
        let mut file = loaded.record.into_file();
        file.set_field("state", "routed");
        file.set_field("routed-to", to);
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Transitioned {
            id: id.to_owned(),
            from: "open",
            to: "routed",
            already: false,
        })
    }

    /// Close a Question without routing: `open → dropped`, appending
    /// `Dropped <date>: <reason>` to the body — deliberately not the Task-log
    /// entry shape, whose author slot this line has no author for.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on an empty reason,
    /// [`NotebookError::InvalidTransition`] from a settled state, plus the
    /// resolution errors of [`Notebook::close`].
    pub fn drop_question(
        &mut self,
        id: &str,
        reason: &str,
        today: &str,
    ) -> Result<Dropped, NotebookError> {
        guard_today(today)?;
        let reason = reason.trim();
        guard_single_line("drop reason", reason)?;
        if reason.is_empty() {
            return Err(NotebookError::InvalidArgument {
                reason: "drop: the reason must not be empty".to_owned(),
            });
        }

        let loaded = self.load_live(id, &[RecordType::Question])?;
        let state = loaded.state_word();
        if state == "dropped" {
            return Ok(Dropped {
                transition: replayed(id, "dropped"),
                dangling_mentions: Vec::new(),
            });
        }
        if state != "open" {
            return Err(settled_question(id, state));
        }
        let mut file = loaded.record.into_file();
        file.set_field("state", "dropped");
        file.set_field("updated", today);
        file.append_body(&format!("Dropped {today}: {reason}"));
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Dropped {
            transition: Transitioned {
                id: id.to_owned(),
                from: "open",
                to: "dropped",
                already: false,
            },
            dangling_mentions: self.dangling_mentions(reason)?,
        })
    }

    /// Retire a Decision or Note without a successor: `active → retired`.
    /// Retirement with a successor is the successor's create declaring
    /// `supersedes`.
    ///
    /// # Errors
    /// [`NotebookError::InvalidTransition`] from a settled state, plus the
    /// resolution errors of [`Notebook::close`].
    pub fn retire(&mut self, id: &str, today: &str) -> Result<Transitioned, NotebookError> {
        guard_today(today)?;
        let loaded = self.load_live(id, &[RecordType::Decision, RecordType::Note])?;
        let state = loaded.state_word();
        if state == "retired" {
            return Ok(replayed(id, "retired"));
        }
        if state != "active" {
            return Err(NotebookError::InvalidTransition {
                id: id.to_owned(),
                state: state.to_owned(),
                valid: Vec::new(),
            });
        }
        let mut file = loaded.record.into_file();
        file.set_field("state", "retired");
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Transitioned {
            id: id.to_owned(),
            from: "active",
            to: "retired",
            already: false,
        })
    }

    /// Move a settled record into the archive: same filename, same bytes,
    /// so `git log --follow` keeps its history and the round-trip contract
    /// holds. The archive copy lands before the live file goes — a failure
    /// between the two leaves a loud `duplicate-id`, never a lost record.
    ///
    /// # Errors
    /// [`NotebookError::InvalidTransition`] on a record still live, naming
    /// the commands that settle it, plus the resolution errors of
    /// [`Notebook::close`].
    pub fn archive(&mut self, id: &str) -> Result<Archived, NotebookError> {
        let record_type = parsed_type(id)?;
        let moved = |already| Archived {
            id: id.to_owned(),
            from: record_path(id, record_type, false),
            to: record_path(id, record_type, true),
            already,
        };
        let loaded = match self.resolve_live(id, record_type) {
            Ok(loaded) => loaded,
            Err(NotebookError::Archived { .. }) => return self.replayed_archive(moved(true)),
            Err(error) => return Err(error),
        };
        if loaded.record.is_live() {
            return Err(NotebookError::InvalidTransition {
                id: id.to_owned(),
                state: loaded.state_word().to_owned(),
                valid: settling_commands(record_type, loaded.state_word()),
            });
        }
        let moved = moved(false);
        self.guard_archive_destination_free(id, &moved.to, loaded.record.file())?;
        self.storage
            .write(&moved.to, &loaded.record.file().render())?;
        self.storage.remove(&moved.from)?;
        Ok(moved)
    }

    /// Judge the copy already in the archive before calling the move done.
    /// Only a clean one proves this very move happened; on any error finding
    /// — the same gate every write passes — answering `already` would report
    /// a corruption as a success.
    fn replayed_archive(&self, moved: Archived) -> Result<Archived, NotebookError> {
        let record = match self.storage.read(&moved.to) {
            Ok(text) => Record::parse(&moved.to, &text),
            Err(StorageError::NotUtf8 { .. }) => Record::unreadable(&moved.to),
            Err(error) => return Err(error.into()),
        };
        if !record.has_errors() {
            return Ok(moved);
        }
        Err(NotebookError::InvalidRecord {
            path: moved.to,
            findings: error_findings(&record),
        })
    }

    /// Delete a record born by mistake, wherever it sits. Retirement ends
    /// something that was once true and the archive keeps history; a record
    /// that should never have existed is the third case, and it leaves no
    /// trace.
    ///
    /// The guard is the whole verb: every inbound edge is named and the
    /// call refuses, since the only alternative to repairing them first is
    /// a notebook pointing at nothing.
    ///
    /// A second call finds nothing and says so: with the record gone there
    /// is nothing left to tell an expunge already done from an id that
    /// never existed, and reporting a typo as success would be worse than
    /// refusing a replay.
    ///
    /// # Errors
    /// [`NotebookError::StillReferenced`] naming every blocker,
    /// [`NotebookError::UnknownId`], [`NotebookError::InvalidArgument`] on
    /// a malformed id, or a storage failure.
    pub fn expunge(&mut self, id: &str) -> Result<Expunged, NotebookError> {
        parsed_type(id)?;
        let paths = self.holder_paths(id)?;
        if paths.is_empty() {
            return Err(NotebookError::UnknownId { id: id.to_owned() });
        }
        let blockers = inbound_edges(&self.read_records()?, id);
        if !blockers.is_empty() {
            return Err(NotebookError::StillReferenced {
                id: id.to_owned(),
                blockers,
            });
        }
        for path in &paths {
            self.storage.remove(path)?;
        }
        Ok(Expunged {
            id: id.to_owned(),
            paths,
        })
    }

    /// Refuse the move when the destination already holds different bytes —
    /// ids are never reused, and overwriting a divergent archived copy would
    /// delete history. Identical bytes pass: that is the replay of a move
    /// that crashed between its write and its remove.
    fn guard_archive_destination_free(
        &self,
        id: &str,
        destination: &str,
        file: &RecordFile,
    ) -> Result<(), NotebookError> {
        let holds_other = match self.storage.read(destination) {
            Ok(existing) => existing != file.render(),
            Err(StorageError::NotFound { .. }) => false,
            Err(StorageError::NotUtf8 { .. }) => true,
            Err(error) => return Err(error.into()),
        };
        if holds_other {
            return Err(NotebookError::DuplicateId {
                id: id.to_owned(),
                holder: destination.to_owned(),
            });
        }
        Ok(())
    }

    /// Apply the deliberate corrections of [`Edit`] to a live record of any
    /// state — a closed record's title is as correctable as an open one's.
    /// `changed` names what moved in the file; a requested value equal to
    /// the standing one moves nothing, though a non-canonical line is a move
    /// even when its value stands. A new body carries the quotation rule's
    /// dangling-mention nudge, like every body-writing verb.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on a malformed value or an `Edit`
    /// requesting nothing, [`NotebookError::DanglingRef`] when `from` names
    /// no record, plus the resolution errors of [`Notebook::close`].
    pub fn edit(&mut self, id: &str, edit: &Edit, today: &str) -> Result<Edited, NotebookError> {
        guard_today(today)?;
        let record_type = parsed_type(id)?;
        validate_edit(record_type, edit)?;
        if let Some(origin) = &edit.from {
            if origin == id {
                return Err(NotebookError::InvalidArgument {
                    reason: "from: a record cannot be its own origin".to_owned(),
                });
            }
            self.guard_ref_exists("from", origin)?;
        }

        let loaded = self.resolve_live(id, record_type)?;
        let dangling_mentions = match &edit.body {
            Some(body) => self.dangling_mentions(body)?,
            None => Vec::new(),
        };
        let mut file = loaded.record.into_file();
        let changed = spliced(&mut file, edit);
        if changed.is_empty() {
            return Ok(Edited {
                id: id.to_owned(),
                changed,
                dangling_mentions,
            });
        }
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Edited {
            id: id.to_owned(),
            changed,
            dangling_mentions,
        })
    }

    /// The shared shape of every Task move: decide first, splice only on a
    /// real move, and let the verb add its own fields before the write.
    fn task_transition(
        &mut self,
        id: &str,
        action: TaskAction,
        today: &str,
        stamp_extras: impl FnOnce(&mut RecordFile),
    ) -> Result<Transitioned, NotebookError> {
        guard_today(today)?;
        let loaded = self.load_live(id, &[RecordType::Task])?;
        let state_word = loaded.state_word();
        let state = TaskState::from_word(state_word)
            .expect("a clean task carries a state from the task enum");

        let transition =
            state
                .transition(action)
                .map_err(|valid| NotebookError::InvalidTransition {
                    id: id.to_owned(),
                    state: state_word.to_owned(),
                    valid: valid.into_iter().map(TaskAction::word).collect(),
                })?;
        let Transition::Move { from, to } = transition else {
            return Ok(replayed(id, state.word()));
        };

        let mut file = loaded.record.into_file();
        file.set_field("state", to.word());
        stamp_extras(&mut file);
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Transitioned {
            id: id.to_owned(),
            from: from.word(),
            to: to.word(),
            already: false,
        })
    }

    /// Resolve `id` to its live record, cleared for mutation: the type
    /// matches the command, the record exists live, and it carries no error
    /// finding — from its own bytes or from a dangling reference.
    fn load_live(&self, id: &str, expected: &[RecordType]) -> Result<LoadedLive, NotebookError> {
        self.load_live_admitting(id, expected, |_, _| false)
    }

    /// [`Notebook::load_live`] with an admission: `admits` names the error
    /// findings this one verb may run over instead of freezing on.
    fn load_live_admitting(
        &self,
        id: &str,
        expected: &[RecordType],
        admits: impl Fn(&Record, &Finding) -> bool,
    ) -> Result<LoadedLive, NotebookError> {
        let record_type = parsed_type(id)?;
        if !expected.contains(&record_type) {
            return Err(NotebookError::WrongType {
                id: id.to_owned(),
                expected: type_list(expected),
            });
        }
        self.resolve_live_admitting(id, record_type, admits)
    }

    /// The one gate every write passes: the record is live and carries no
    /// error finding, so no path — a verb or a supersession flip — can
    /// rewrite an invalid record.
    fn resolve_live(&self, id: &str, record_type: RecordType) -> Result<LoadedLive, NotebookError> {
        self.resolve_live_admitting(id, record_type, |_, _| false)
    }

    fn resolve_live_admitting(
        &self,
        id: &str,
        record_type: RecordType,
        admits: impl Fn(&Record, &Finding) -> bool,
    ) -> Result<LoadedLive, NotebookError> {
        let path = record_path(id, record_type, false);
        let text = match self.storage.read(&path) {
            Ok(text) => text,
            Err(StorageError::NotFound { .. }) => {
                return Err(match self.holder_path(id)? {
                    Some(_) => NotebookError::Archived { id: id.to_owned() },
                    None => NotebookError::UnknownId { id: id.to_owned() },
                });
            }
            Err(StorageError::NotUtf8 { .. }) => {
                return Err(NotebookError::InvalidRecord {
                    path,
                    findings: vec![not_utf8_finding()],
                });
            }
            Err(error) => return Err(error.into()),
        };
        let record = Record::parse(&path, &text);
        let errors: Vec<Finding> = self
            .exclusion_errors(&record)?
            .into_iter()
            .filter(|finding| !admits(&record, finding))
            .collect();
        if !errors.is_empty() {
            return Err(NotebookError::InvalidRecord {
                path,
                findings: errors,
            });
        }
        Ok(LoadedLive { path, record })
    }

    /// The cycle the edge `id → on` would close, walked in full: `on`
    /// already waits on `id`, or is `id` itself.
    fn would_cycle(&self, id: &str, on: &str) -> Result<Option<Vec<String>>, NotebookError> {
        if id == on {
            return Ok(Some(vec![id.to_owned(), on.to_owned()]));
        }
        let tasks = self.records_of(RecordType::Task)?;
        let Some(chain) = task_graph(&tasks).path(on, id) else {
            return Ok(None);
        };
        let mut chain_from_dependent = vec![id.to_owned()];
        chain_from_dependent.extend(chain);
        Ok(Some(chain_from_dependent))
    }

    /// The error findings that exclude a record from mutation: its own,
    /// plus a dangling reference probed against storage — the same rule the
    /// derived queries answer from the resolvable map, adapted here to a
    /// single record so the gate never has to read the whole notebook.
    fn exclusion_errors(&self, record: &Record) -> Result<Vec<Finding>, NotebookError> {
        let mut errors = error_findings(record);
        for key in REF_KEYS {
            for (target, line) in record.file().field_entries(key) {
                if grammar::id_error(target).is_some() {
                    continue;
                }
                if self.holder_path(target)?.is_none() {
                    errors.push(dangling_finding(record, key, target, line));
                }
            }
        }
        Ok(errors)
    }

    /// The write-time half of the quotation rule: the bare ids `text` cites
    /// that resolve to no record, live or archived. A nudge for the reply,
    /// never a gate — a forward reference is legal and the text lands as
    /// given.
    fn dangling_mentions(&self, text: &str) -> Result<Vec<String>, NotebookError> {
        let mut dangling = Vec::new();
        for target in mention::mentions(text) {
            if self.holder_path(target)?.is_none() {
                dangling.push(target.to_owned());
            }
        }
        Ok(dangling)
    }

    fn guard_ref_exists(&self, field: &'static str, target: &str) -> Result<(), NotebookError> {
        if grammar::id_error(target).is_some() {
            return Err(NotebookError::InvalidArgument {
                reason: format!("{field}: `{target}` is not a record id"),
            });
        }
        if self.holder_path(target)?.is_none() {
            return Err(NotebookError::DanglingRef {
                field,
                target: target.to_owned(),
            });
        }
        Ok(())
    }

    /// Validate a declared supersession and hand back the victim ready to
    /// flip; `None` when the draft declares none.
    fn guard_supersession(&self, draft: &Draft) -> Result<Option<Victim>, NotebookError> {
        let Some(target) = &draft.supersedes else {
            return Ok(None);
        };
        self.guard_ref_exists("supersedes", target)?;
        let victim_type = parsed_type(target)?;
        let dead_state = match victim_type {
            RecordType::Decision => "superseded",
            RecordType::Note => "retired",
            RecordType::Task | RecordType::Question => {
                return Err(NotebookError::CannotSupersede {
                    id: target.clone(),
                    reason: format!(
                        "a {} closes through its own lifecycle, not supersession",
                        victim_type.word()
                    ),
                });
            }
        };

        let loaded = match self.resolve_live(target, victim_type) {
            Ok(loaded) => loaded,
            Err(NotebookError::Archived { id }) => {
                return Err(NotebookError::CannotSupersede {
                    id,
                    reason: "it is archived".to_owned(),
                });
            }
            Err(error) => return Err(error),
        };
        let LoadedLive { path, record } = loaded;
        if let Some(superseder) = record.superseded_by() {
            return Err(NotebookError::CannotSupersede {
                id: target.clone(),
                reason: format!("it is already superseded by `{superseder}`"),
            });
        }
        if record.state() != Some("active") {
            return Err(NotebookError::CannotSupersede {
                id: target.clone(),
                reason: format!(
                    "its state is `{}`, not active",
                    record.state().unwrap_or_default()
                ),
            });
        }
        Ok(Some(Victim {
            path,
            record,
            dead_state,
        }))
    }

    /// Where `id` lives, if anywhere: its live path, else its archive path.
    fn holder_path(&self, id: &str) -> Result<Option<String>, NotebookError> {
        Ok(self.holder_paths(id)?.into_iter().next())
    }

    /// Every canonical path holding `id`, live before archived. Two is the
    /// `duplicate-id` corruption an interrupted archive move leaves; the
    /// verbs that read one record take the first, and only expunge, which
    /// must leave nothing behind, needs them all.
    fn holder_paths(&self, id: &str) -> Result<Vec<String>, NotebookError> {
        let Some(record_type) = id
            .split_once('.')
            .and_then(|(word, _)| RecordType::from_word(word))
        else {
            return Ok(Vec::new());
        };
        let mut holders = Vec::new();
        for archived in [false, true] {
            let path = record_path(id, record_type, archived);
            match self.storage.read(&path) {
                // A file that is not UTF-8 still claims its id — ids are
                // never reused, unreadable holders included.
                Ok(_) | Err(StorageError::NotUtf8 { .. }) => holders.push(path),
                Err(StorageError::NotFound { .. }) => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(holders)
    }

    fn read_records(&self) -> Result<Vec<Record>, NotebookError> {
        let mut records = Vec::new();
        for record_type in RecordType::ALL {
            records.extend(self.records_of(record_type)?);
        }
        Ok(records)
    }

    /// One type's records, live and archived, invalid ones included: an
    /// invalid record is a visible first-class state, never a silent drop.
    fn records_of(&self, record_type: RecordType) -> Result<Vec<Record>, NotebookError> {
        let live = record_type.directory().to_owned();
        let mut records = Vec::new();
        for dir in [live.clone(), format!("archive/{live}")] {
            for path in self.storage.list(&dir)? {
                if !is_record_file(&path) {
                    continue;
                }
                match self.storage.read(&path) {
                    Ok(text) => records.push(Record::parse(&path, &text)),
                    // The adapter's duty ends at naming the encoding; the
                    // file stays a visible invalid record, not an abort.
                    Err(StorageError::NotUtf8 { .. }) => records.push(Record::unreadable(&path)),
                    Err(error) => return Err(error.into()),
                }
            }
        }
        Ok(records)
    }
}

struct LoadedLive {
    path: String,
    record: Record,
}

impl LoadedLive {
    fn state_word(&self) -> &str {
        self.record.state().expect("a clean record carries a state")
    }
}

struct Victim {
    path: String,
    record: Record,
    dead_state: &'static str,
}

fn replayed(id: &str, state: &'static str) -> Transitioned {
    Transitioned {
        id: id.to_owned(),
        from: state,
        to: state,
        already: true,
    }
}

fn settled_question(id: &str, state: &str) -> NotebookError {
    NotebookError::InvalidTransition {
        id: id.to_owned(),
        state: state.to_owned(),
        valid: Vec::new(),
    }
}

/// The draft's id: the caller's, validated and free, or one minted from
/// the title — retried with a two-character suffix on collision, since
/// ids are never reused.
fn resolve_draft_id(
    draft: &Draft,
    claims: &BTreeMap<String, String>,
) -> Result<String, NotebookError> {
    if let Some(id) = &draft.id {
        if let Some(why) = grammar::id_error(id) {
            return Err(NotebookError::InvalidArgument {
                reason: format!("id: {why}"),
            });
        }
        let id_type = parsed_type(id)?;
        if id_type != draft.record_type {
            return Err(NotebookError::InvalidArgument {
                reason: format!(
                    "id: `{id}` names a {}, the draft is a {}",
                    id_type.word(),
                    draft.record_type.word()
                ),
            });
        }
        if let Some(holder) = claims.get(id.as_str()) {
            return Err(NotebookError::DuplicateId {
                id: id.clone(),
                holder: holder.clone(),
            });
        }
        return Ok(id.clone());
    }

    let slug = slugify(&draft.title);
    if slug.is_empty() {
        return Err(NotebookError::InvalidArgument {
            reason: "title: yields an empty id — pass an explicit id".to_owned(),
        });
    }
    let base = format!("{}.{slug}", draft.record_type.word());
    if !claims.contains_key(&base) {
        return Ok(base);
    }
    for attempt in 0..1296 {
        let candidate = format!(
            "{base}-{}",
            base36_pair(fnv1a(&draft.title).wrapping_add(attempt))
        );
        if !claims.contains_key(&candidate) {
            return Ok(candidate);
        }
    }
    Err(NotebookError::InvalidArgument {
        reason: format!("id: no free id near `{base}`"),
    })
}

/// Every id claimed anywhere, mapped to its claimant: by filename, and
/// by the `id` field of a misnamed file — write-side uniqueness cannot
/// trust the convention whose violation is the very finding it guards
/// against.
fn id_claims(records: &[Record]) -> BTreeMap<String, String> {
    let mut claims = BTreeMap::new();
    for record in records {
        claims
            .entry(path_stem(record.path()).to_owned())
            .or_insert_with(|| record.path().to_owned());
        if let Some(id) = record.id() {
            claims
                .entry(id.to_owned())
                .or_insert_with(|| record.path().to_owned());
        }
    }
    claims
}

/// The standing Decisions a draft may conflict with. Computed at write
/// time because the writing agent, holding full context, is the cheapest
/// judge that will ever see the pair; the tool prints it and stops.
fn conflict_candidates(draft: &Draft, records: &[Record]) -> Vec<Cited> {
    if draft.record_type != RecordType::Decision || draft.supersedes.is_some() {
        return Vec::new();
    }
    let resolvable = resolvable_by_id(records);
    let draft_tags: BTreeSet<&str> = draft.tags.iter().map(String::as_str).collect();
    let cited_ids = mention::mentions(&draft.body);
    let mut hits: Vec<&Record> = records
        .iter()
        .filter(|record| {
            is_standing_decision(record, &resolvable)
                && looks_related(record, &draft_tags, &cited_ids)
        })
        .collect();
    hits.sort_by(|left, right| oldest_first(left, right));
    hits.into_iter().map(Cited::of).collect()
}

fn is_standing_decision(record: &Record, resolvable: &BTreeMap<&str, &Record>) -> bool {
    record.record_type() == Some(RecordType::Decision)
        && !is_archived(record.path())
        && record.is_live()
        && !debt::is_excluded(record, resolvable)
}

fn looks_related(record: &Record, draft_tags: &BTreeSet<&str>, cited_ids: &[&str]) -> bool {
    shared_tag_count(record, draft_tags) >= 2 || cited_ids.contains(&path_stem(record.path()))
}

fn shared_tag_count(record: &Record, draft_tags: &BTreeSet<&str>) -> usize {
    let Some(tags) = record.file().field("tags") else {
        return 0;
    };
    tags.split(',')
        .map(str::trim)
        .collect::<BTreeSet<&str>>()
        .intersection(draft_tags)
        .count()
}

/// What an ingested report is called, so a reader scanning `list` sees
/// whose report it is and never mistakes it for the Task itself.
fn report_note_title(task_title: &str) -> String {
    format!("Report: {task_title}")
}

/// A proof is a link value: one non-empty line, or an explicit waiver.
fn guard_proof(proof: &Proof) -> Result<(), NotebookError> {
    let (Proof::Pr(target) | Proof::Sha(target) | Proof::Report(target) | Proof::Note(target)) =
        proof
    else {
        return Ok(());
    };
    guard_single_line("proof", target)?;
    if target.trim().is_empty() {
        return Err(NotebookError::InvalidArgument {
            reason: "proof: the target must not be empty".to_owned(),
        });
    }
    Ok(())
}

/// [`guard_today`] plus the day number the clocks subtract from.
fn guarded_day(today: &str) -> Result<i64, NotebookError> {
    guard_today(today)?;
    grammar::day_number(today).ok_or_else(|| NotebookError::InvalidArgument {
        reason: format!("today: `{today}` is not a date"),
    })
}

fn guard_today(today: &str) -> Result<(), NotebookError> {
    match grammar::date_error(today) {
        None => Ok(()),
        Some(why) => Err(NotebookError::InvalidArgument {
            reason: format!("today: {why}"),
        }),
    }
}

fn guard_single_line(field: &str, value: &str) -> Result<(), NotebookError> {
    if value.contains('\n') {
        return Err(NotebookError::InvalidArgument {
            reason: format!("{field}: must be one line"),
        });
    }
    Ok(())
}

fn parsed_type(id: &str) -> Result<RecordType, NotebookError> {
    if let Some(why) = grammar::id_error(id) {
        return Err(NotebookError::InvalidArgument {
            reason: format!("id: {why}"),
        });
    }
    let (word, _) = id.split_once('.').expect("a valid id contains a dot");
    Ok(RecordType::from_word(word).expect("a valid id starts with a type word"))
}

fn record_path(id: &str, record_type: RecordType, archived: bool) -> String {
    let dir = record_type.directory();
    if archived {
        format!("archive/{dir}/{id}.md")
    } else {
        format!("{dir}/{id}.md")
    }
}

/// The format is byte-exact: `.MD` is not a record file.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn is_record_file(path: &str) -> bool {
    path.ends_with(".md")
}

fn is_archived(path: &str) -> bool {
    path.starts_with("archive/")
}

fn edge_exists(record: &Record, target: &str) -> bool {
    record
        .file()
        .field_values("blocked-by")
        .any(|value| value == target)
}

/// An error finding sitting on one of the record's own `blocked-by` lines:
/// the class `unblock` exists to erase.
fn edge_borne(record: &Record, finding: &Finding) -> bool {
    finding.line.is_some()
        && record
            .file()
            .field_entries("blocked-by")
            .any(|(_, line)| line == finding.line)
}

/// The dependency graph over the Tasks handed in, keyed by file stem — the
/// name graph findings report against. Archived Tasks enter too: a cycle
/// through the archive is still a cycle.
fn task_graph(records: &[Record]) -> TaskGraph {
    let mut nodes = BTreeMap::new();
    for record in records {
        if record.record_type() != Some(RecordType::Task) {
            continue;
        }
        let blocked_by = record
            .file()
            .field_values("blocked-by")
            .filter(|target| grammar::id_error(target).is_none())
            .map(str::to_owned)
            .collect();
        nodes
            .entry(path_stem(record.path()).to_owned())
            .or_insert(TaskNode {
                closed: record.state() == Some("closed"),
                blocked_by,
            });
    }
    TaskGraph::new(nodes)
}

fn ready_row(record: &Record) -> ReadyTask {
    let file = record.file();
    ReadyTask {
        id: path_stem(record.path()).to_owned(),
        priority: file.field("priority").and_then(|value| value.parse().ok()),
        created: file.field("created").unwrap_or_default().to_owned(),
        title: file.field("title").unwrap_or_default().to_owned(),
    }
}

/// Ready order: the most urgent priority first (0 is the most urgent; none
/// ranks at the neutral middle — priority is an override, not a promotion
/// over the untriaged), then oldest first, then id. The ISO date orders as
/// text.
fn ready_rank(row: &ReadyTask) -> (u8, &str, &str) {
    (row.priority.unwrap_or(2), &row.created, &row.id)
}

/// Errors first, then file, then line (file-level findings last), then code.
/// Severity outranks the file because the reply prints a bounded head: a
/// warning the tool's own happy path produces must never push an error out
/// of a truncated report.
fn finding_order(located: &FileFinding) -> (u8, &str, usize, &'static str) {
    let severity = match located.finding.code.severity() {
        Severity::Error => 0,
        Severity::Warning => 1,
    };
    (
        severity,
        located.path.as_str(),
        located.finding.line.unwrap_or(usize::MAX),
        located.finding.code.as_str(),
    )
}

/// The id a canonical record path carries: the filename minus `.md`. The
/// path↔id rule has this one home; a host never re-derives it.
#[must_use]
pub fn path_stem(path: &str) -> &str {
    let filename = path.rsplit('/').next().unwrap_or(path);
    filename.strip_suffix(".md").unwrap_or(filename)
}

fn type_list(types: &[RecordType]) -> String {
    let words: Vec<String> = types
        .iter()
        .map(|record_type| format!("a {}", record_type.word()))
        .collect();
    words.join(" or ")
}

/// The records a reference can resolve to, keyed by id: exactly those
/// sitting at their id's canonical live or archive path — the listing-side
/// twin of the mutation gate's storage probe, so a file in the wrong
/// directory resolves nothing anywhere.
fn resolvable_by_id(records: &[Record]) -> BTreeMap<&str, &Record> {
    records
        .iter()
        .filter_map(|record| {
            let stem = path_stem(record.path());
            let record_type = stem
                .split_once('.')
                .and_then(|(word, _)| RecordType::from_word(word))?;
            let canonical = record_path(stem, record_type, is_archived(record.path()));
            (grammar::id_error(stem).is_none() && record.path() == canonical)
                .then_some((stem, record))
        })
        .collect()
}

fn listed_row(record: &Record, resolvable: &BTreeMap<&str, &Record>) -> ListedRecord {
    let file = record.file();
    let state = if debt::is_excluded(record, resolvable) {
        "invalid".to_owned()
    } else {
        record.state().unwrap_or_default().to_owned()
    };
    ListedRecord {
        id: path_stem(record.path()).to_owned(),
        state,
        priority: file.field("priority").and_then(|value| value.parse().ok()),
        title: file.field("title").map(str::to_owned),
    }
}

fn matches_query(record: &Record, needle: &str) -> bool {
    let file = record.file();
    [
        Some(path_stem(record.path())),
        file.field("title"),
        file.field("tags"),
        Some(file.body()),
    ]
    .into_iter()
    .flatten()
    .any(|surface| surface.to_lowercase().contains(needle))
}

/// Whether the record's file sits in `record_type`'s live or archive
/// directory — residence, the axis a reader browsing the tree sees.
fn sits_in(record: &Record, record_type: RecordType, home: Residence) -> bool {
    grammar::residence(record.path(), record_type.word()) == Some(home)
}

fn archived_counts(records: &[Record]) -> Counts {
    let of = |record_type| {
        records
            .iter()
            .filter(|record| sits_in(record, record_type, Residence::Archive))
            .count()
    };
    Counts {
        tasks: of(RecordType::Task),
        decisions: of(RecordType::Decision),
        notes: of(RecordType::Note),
        questions: of(RecordType::Question),
    }
}

/// The commands that carry a still-live record toward its settled state —
/// the recovery payload of an archive refused too early.
fn settling_commands(record_type: RecordType, state: &str) -> Vec<&'static str> {
    match (record_type, state) {
        (RecordType::Task, "open") => vec!["start"],
        (RecordType::Task, _) => vec!["close"],
        (RecordType::Question, _) => vec!["answer"],
        (RecordType::Decision | RecordType::Note, _) => vec!["retire"],
    }
}

/// The dispatch queue's rows: live, open, valid, unblocked, unheld Tasks
/// in ready order.
fn ready_rows(records: &[Record], resolvable: &BTreeMap<&str, &Record>) -> Vec<ReadyTask> {
    let graph = task_graph(records);
    let keep =
        |record: &Record| record.hold().is_none() && !graph.is_blocked(path_stem(record.path()));
    open_rows(records, resolvable, keep)
}

/// The open Tasks a close of `id` released, in ready order: their last live
/// blocker was that Task. A held one is named too — the hold gates
/// `ready`, not the fact.
fn unblocked_by_close(
    records: &[Record],
    resolvable: &BTreeMap<&str, &Record>,
    id: &str,
) -> Vec<String> {
    let freed = task_graph(records).unblocked_by(id);
    let keep = |record: &Record| {
        freed
            .iter()
            .any(|freed_id| freed_id == path_stem(record.path()))
    };
    open_rows(records, resolvable, keep)
        .into_iter()
        .map(|row| row.id)
        .collect()
}

/// The live, open, valid Tasks passing `keep`, in ready order.
fn open_rows(
    records: &[Record],
    resolvable: &BTreeMap<&str, &Record>,
    keep: impl Fn(&Record) -> bool,
) -> Vec<ReadyTask> {
    let mut rows: Vec<ReadyTask> = records
        .iter()
        .filter(|record| {
            record.record_type() == Some(RecordType::Task)
                && !is_archived(record.path())
                && record.state() == Some("open")
                && keep(record)
                && !debt::is_excluded(record, resolvable)
        })
        .map(ready_row)
        .collect();
    rows.sort_by(|left, right| ready_rank(left).cmp(&ready_rank(right)));
    rows
}

/// The still-open, valid Questions whose Origin is this Task; an invalid
/// one is `check`'s to name, as everywhere.
fn open_questions_from(
    records: &[Record],
    resolvable: &BTreeMap<&str, &Record>,
    task_id: &str,
) -> Vec<String> {
    records
        .iter()
        .filter(|record| {
            record.record_type() == Some(RecordType::Question)
                && !is_archived(record.path())
                && record.origin() == Some(task_id)
                && record.state() == Some("open")
                && !debt::is_excluded(record, resolvable)
        })
        .map(|record| path_stem(record.path()).to_owned())
        .collect()
}

fn live_counts(records: &[Record]) -> Counts {
    let of = |record_type| {
        records
            .iter()
            .filter(|record| {
                !is_archived(record.path()) && record.record_type() == Some(record_type)
            })
            .count()
    };
    Counts {
        tasks: of(RecordType::Task),
        decisions: of(RecordType::Decision),
        notes: of(RecordType::Note),
        questions: of(RecordType::Question),
    }
}

/// The active Tasks, the most recently touched first: the dashboard's
/// in-flight lines, the first carrying the last log line — the mechanical
/// "where I stopped".
fn in_flight_tasks(live_valid: &[&Record]) -> Vec<ActiveTask> {
    let mut active: Vec<&Record> = live_valid
        .iter()
        .copied()
        .filter(|record| {
            record.record_type() == Some(RecordType::Task) && record.state() == Some("active")
        })
        .collect();
    active.sort_by(|left, right| {
        touched(right)
            .cmp(touched(left))
            .then_with(|| path_stem(left.path()).cmp(path_stem(right.path())))
    });
    active
        .iter()
        .enumerate()
        .map(|(position, record)| ActiveTask {
            id: path_stem(record.path()).to_owned(),
            title: record.file().field("title").unwrap_or_default().to_owned(),
            log: (position == 0).then(|| last_log_line(record)).flatten(),
        })
        .collect()
}

/// The last non-empty body line; the log convention makes it meaningful,
/// nothing parses it.
fn last_log_line(record: &Record) -> Option<String> {
    record
        .file()
        .body()
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::to_owned)
}

/// The Tasks waiting on a human for acceptance.
fn review_tasks(live_valid: &[&Record]) -> Vec<String> {
    let mut ids: Vec<String> = live_valid
        .iter()
        .filter(|record| {
            record.record_type() == Some(RecordType::Task) && record.state() == Some("review")
        })
        .map(|record| path_stem(record.path()).to_owned())
        .collect();
    ids.sort();
    ids
}

/// When the record last moved: `updated`, else `created` — the same proxy
/// the Debt clocks subtract from.
fn touched(record: &Record) -> &str {
    let file = record.file();
    file.field("updated")
        .or_else(|| file.field("created"))
        .unwrap_or_default()
}

fn created(record: &Record) -> &str {
    record.file().field("created").unwrap_or_default()
}

/// The order records were laid down: `created`, then id.
fn oldest_first(left: &Record, right: &Record) -> std::cmp::Ordering {
    created(left)
        .cmp(created(right))
        .then_with(|| path_stem(left.path()).cmp(path_stem(right.path())))
}

/// The standing rules: live Decisions of kind `rule`, oldest first —
/// the order they were laid down.
fn standing_rules(live_valid: &[&Record]) -> Vec<StatusRule> {
    let mut rules: Vec<&Record> = live_valid
        .iter()
        .copied()
        .filter(|record| {
            record.record_type() == Some(RecordType::Decision)
                && record.is_live()
                && record.file().field("kind") == Some("rule")
        })
        .collect();
    rules.sort_by(|left, right| oldest_first(left, right));
    rules
        .iter()
        .map(|record| StatusRule {
            id: path_stem(record.path()).to_owned(),
            title: record.file().field("title").unwrap_or_default().to_owned(),
        })
        .collect()
}

fn validate_draft(draft: &Draft) -> Result<(), NotebookError> {
    let invalid = |reason: String| Err(NotebookError::InvalidArgument { reason });

    let title = draft.title.trim();
    guard_single_line("title", title)?;
    if title.is_empty() {
        return invalid("title: must not be empty".to_owned());
    }
    for (field, value) in [
        ("by", &draft.by),
        ("via", &draft.via),
        ("kind", &draft.kind),
    ] {
        if let Some(value) = value {
            guard_single_line(field, value)?;
        }
    }

    if let Some(kind) = &draft.kind {
        match draft.record_type.kinds() {
            None => {
                return invalid(format!(
                    "kind: a {} carries no kind",
                    draft.record_type.word()
                ));
            }
            Some(kinds) if !kinds.contains(&kind.as_str()) => {
                return invalid(format!(
                    "kind: `{kind}` is not one of {} for a {}",
                    kinds.join(", "),
                    draft.record_type.word()
                ));
            }
            Some(_) => {}
        }
    }

    if let Some(priority) = draft.priority {
        if draft.record_type != RecordType::Task {
            return invalid("priority: applies only to a task".to_owned());
        }
        if priority > 4 {
            return invalid(format!("priority: {priority} is not 0–4"));
        }
    }

    if draft.supersedes.is_some()
        && !matches!(draft.record_type, RecordType::Decision | RecordType::Note)
    {
        return invalid(format!(
            "supersedes: a {} closes through its own lifecycle, not supersession",
            draft.record_type.word()
        ));
    }

    for tag in &draft.tags {
        if !grammar::is_token(tag) {
            return invalid(format!("tags: `{tag}` is not a `[a-z0-9-]+` tag"));
        }
    }
    for link in &draft.links {
        guard_single_line("link", &link.target)?;
        if !grammar::is_token(&link.kind) || link.target.trim().is_empty() {
            return invalid(format!(
                "link: `{} {}` is not `<kind> <target>`",
                link.kind, link.target
            ));
        }
    }
    Ok(())
}

fn validate_edit(record_type: RecordType, edit: &Edit) -> Result<(), NotebookError> {
    let invalid = |reason: String| Err(NotebookError::InvalidArgument { reason });

    if edit.changes_nothing() {
        return invalid(
            "edit: nothing to change — pass --title, --body, --tag, --untag, --from, --priority, or --review-by"
                .to_owned(),
        );
    }
    if let Some(title) = &edit.title {
        let title = title.trim();
        guard_single_line("title", title)?;
        if title.is_empty() {
            return invalid("title: must not be empty".to_owned());
        }
    }
    for tag in edit.add_tags.iter().chain(&edit.remove_tags) {
        if !grammar::is_token(tag) {
            return invalid(format!("tags: `{tag}` is not a `[a-z0-9-]+` tag"));
        }
    }
    if let Some(priority) = edit.priority {
        if record_type != RecordType::Task {
            return invalid("priority: applies only to a task".to_owned());
        }
        if priority > 4 {
            return invalid(format!("priority: {priority} is not 0–4"));
        }
    }
    if let Some(date) = &edit.review_by
        && let Some(why) = grammar::date_error(date)
    {
        return invalid(format!("review-by: {why}"));
    }
    Ok(())
}

/// Splice every requested correction into the file, answering the keys
/// that actually moved.
fn spliced(file: &mut RecordFile, edit: &Edit) -> Vec<&'static str> {
    let mut changed = Vec::new();
    let fields = [
        (
            "title",
            edit.title.as_deref().map(str::trim).map(str::to_owned),
        ),
        ("from", edit.from.clone()),
        (
            "priority",
            edit.priority.map(|priority| priority.to_string()),
        ),
        ("review-by", edit.review_by.clone()),
    ];
    for (key, value) in fields {
        if let Some(value) = value
            && file.set_field(key, &value)
        {
            changed.push(key);
        }
    }
    if retagged(file, edit) {
        changed.push("tags");
    }
    if let Some(body) = &edit.body {
        let body = edited_body(body);
        if body != file.body() {
            file.set_body(&body);
            changed.push("body");
        }
    }
    changed
}

/// Splice the edit's tag additions and removals into the `tags` field;
/// answers whether the set changed. An added tag already present, or a
/// removed one already absent, changes nothing — the replay contract at
/// set granularity.
fn retagged(file: &mut RecordFile, edit: &Edit) -> bool {
    if edit.add_tags.is_empty() && edit.remove_tags.is_empty() {
        return false;
    }
    let mut tags: Vec<String> = file
        .field("tags")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    tags.retain(|tag| !edit.remove_tags.contains(tag));
    for tag in &edit.add_tags {
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }
    if tags.is_empty() {
        file.remove_field("tags")
    } else {
        file.set_field("tags", &tags.join(", "))
    }
}

/// The canonical rendered body: a blank line after the fence, the content,
/// a final newline — and empty content is no body at all. `create` and
/// `edit` both write through it, so a fresh record and an edited one read
/// alike.
fn edited_body(content: &str) -> String {
    let content = content.strip_suffix('\n').unwrap_or(content);
    if content.is_empty() {
        String::new()
    } else {
        format!("\n{content}\n")
    }
}

/// Render a draft as a canonical record file. The splice machinery places
/// every field, so the canonical order has one home: the grammar's field
/// table.
fn render_draft(draft: &Draft, id: &str, today: &str) -> String {
    let mut file = RecordFile::parse("---\n---\n");
    file.set_field("id", id);
    file.set_field("type", draft.record_type.word());
    file.set_field("state", draft.record_type.initial_state());
    file.set_field("title", draft.title.trim());
    for (key, value) in [
        ("kind", &draft.kind),
        ("by", &draft.by),
        ("via", &draft.via),
        ("from", &draft.from),
        ("supersedes", &draft.supersedes),
    ] {
        if let Some(value) = value {
            file.set_field(key, value);
        }
    }
    if !draft.tags.is_empty() {
        file.set_field("tags", &draft.tags.join(", "));
    }
    for link in &draft.links {
        file.append_field("link", &format!("{} {}", link.kind, link.target.trim()));
    }
    if let Some(priority) = draft.priority {
        file.set_field("priority", &priority.to_string());
    }
    file.set_field("created", today);
    file.set_field("updated", today);
    file.set_body(&edited_body(&draft.body));
    file.render()
}

/// How much of a title an id carries. An id must stay recognisable at a
/// glance and must fit the id grammar's own length limit with room for a
/// collision suffix; the rest of the title is a `view` away.
const SLUG_CAP: usize = 40;

/// The slug an id takes from a title: ASCII alphanumerics lowercased, every
/// other run a single hyphen, cut to [`SLUG_CAP`] at a word boundary.
///
/// An id is read far more often than it is minted, and a mid-word cut costs
/// its reader more than the characters it saves. A first word longer than
/// the cap offers no boundary to cut at, so it is cut short.
fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_end_matches('-');
    if slug.len() <= SLUG_CAP {
        return slug.to_owned();
    }
    // One past the cap, so a boundary sitting exactly on it still counts.
    let within_cap = &slug[..=SLUG_CAP];
    match within_cap.rfind('-') {
        Some(boundary) => slug[..boundary].to_owned(),
        None => slug[..SLUG_CAP].to_owned(),
    }
}

fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn base36_pair(n: u64) -> String {
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let n = (n % 1296) as usize;
    let pair = [DIGITS[n / 36], DIGITS[n % 36]];
    String::from_utf8_lossy(&pair).into_owned()
}

/// The finding a reference into nothing deserves — every surface names one
/// condition with one code: a routed Question pointing at nothing has lost
/// what closed it, any other dangling reference is a `dangling-ref`.
fn dangling_finding(record: &Record, key: &str, target: &str, line: Option<usize>) -> Finding {
    let routed =
        record.record_type() == Some(RecordType::Question) && record.state() == Some("routed");
    if key == "routed-to" && routed {
        Finding::located(
            line,
            FindingCode::BrokenRouting,
            format!("routed-to: `{target}` does not exist — the routing thread is lost"),
        )
    } else {
        Finding::located(
            line,
            FindingCode::DanglingRef,
            format!("{key}: `{target}` names no record"),
        )
    }
}

/// Every edge pointing at `target` from somewhere else, in file order.
///
/// An envelope key and a body citation both count, and an invalid record's
/// edges count too: what makes an edge a blocker is that removing the
/// target would leave it pointing at nothing, and a file the tool refuses
/// to mutate is the worst place to leave that. The target's own edges are
/// not blockers — they leave with it.
fn inbound_edges(records: &[Record], target: &str) -> Vec<Blocker> {
    let mut blockers = Vec::new();
    for record in records {
        let carrier = path_stem(record.path());
        if carrier == target {
            continue;
        }
        let mut held_by = |through| {
            blockers.push(Blocker {
                carrier: carrier.to_owned(),
                through,
            });
        };
        for key in REF_KEYS {
            if record.file().field_values(key).any(|value| value == target) {
                held_by(key);
            }
        }
        if record
            .file()
            .field_values("link")
            .filter_map(linked_record)
            .any(|linked| linked == target)
        {
            held_by("link");
        }
        if mention::mentions(record.file().body()).contains(&target) {
            held_by("body");
        }
    }
    blockers
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

fn error_findings(record: &Record) -> Vec<Finding> {
    record
        .findings()
        .iter()
        .filter(|finding| finding.code.severity() == Severity::Error)
        .cloned()
        .collect()
}

fn check_refs(record: &Record, by_stem: &BTreeMap<&str, &Record>, out: &mut Vec<FileFinding>) {
    let mut dangling = |key, target: &str, line| {
        if grammar::id_error(target).is_some() || by_stem.contains_key(target) {
            return;
        }
        out.push(FileFinding {
            path: record.path().to_owned(),
            finding: dangling_finding(record, key, target, line),
        });
    };
    for key in REF_KEYS {
        for (target, line) in record.file().field_entries(key) {
            dangling(key, target, line);
        }
    }
    // A proof carried as a Note is a reference like any other: the whole
    // point of ingesting the report was that the notebook could reach it.
    for (link, line) in record.file().field_entries("link") {
        if let Some(target) = linked_record(link) {
            dangling("link", target, line);
        }
    }
}

/// The supersession pair, verified from both ends, reported against both
/// files: a forward pointer without its back-pointer, a back-pointer without
/// its claim, and a record marked replaced that still reads live.
fn check_supersession_pair(
    record: &Record,
    by_stem: &BTreeMap<&str, &Record>,
    out: &mut Vec<FileFinding>,
) {
    let stem = path_stem(record.path());
    let mut pair_finding = |path: &str, finding: Finding| {
        out.push(FileFinding {
            path: path.to_owned(),
            finding,
        });
    };

    if let Some((target, line)) = record.file().field_entry("supersedes")
        && let Some(victim) = by_stem.get(target)
        && victim.superseded_by() != Some(stem)
    {
        pair_finding(
            record.path(),
            Finding::located(
                line,
                FindingCode::BrokenSupersession,
                format!("supersedes: `{target}` does not point back with `superseded-by`"),
            ),
        );
        pair_finding(
            victim.path(),
            Finding::for_file(
                FindingCode::BrokenSupersession,
                format!("`{stem}` claims to supersede this record, which does not point back"),
            ),
        );
    }

    if let Some((superseder, line)) = record.file().field_entry("superseded-by") {
        if let Some(claimant) = by_stem.get(superseder)
            && claimant.supersedes() != Some(stem)
        {
            pair_finding(
                record.path(),
                Finding::located(
                    line,
                    FindingCode::BrokenSupersession,
                    format!("superseded-by: `{superseder}` does not claim `supersedes: {stem}`"),
                ),
            );
            pair_finding(
                claimant.path(),
                Finding::for_file(
                    FindingCode::BrokenSupersession,
                    format!(
                        "`{stem}` names this record as its superseder, which does not claim it"
                    ),
                ),
            );
        }
        if let Some(record_type) = record.record_type()
            && let Some((state, state_line)) = record.file().field_entry("state")
            && record_type.live_states().contains(&state)
        {
            pair_finding(
                record.path(),
                Finding::located(
                    state_line,
                    FindingCode::BrokenSupersession,
                    format!(
                        "state: `{state}` on a record marked `superseded-by` — a replaced record must not read live"
                    ),
                ),
            );
        }
    }
}

/// Cycles among `blocked-by` edges, named on every member file at its own
/// edge line. The tool refuses them at write, so a cycle is hand-edited
/// corruption — without this pass it would only sit there emptying `ready`.
fn check_dep_cycles(
    records: &[Record],
    by_stem: &BTreeMap<&str, &Record>,
    out: &mut Vec<FileFinding>,
) {
    for cycle in task_graph(records).cycles() {
        let walk = cycle
            .iter()
            .chain(cycle.first())
            .cloned()
            .collect::<Vec<String>>()
            .join(" → ");
        for (position, member) in cycle.iter().enumerate() {
            let next = &cycle[(position + 1) % cycle.len()];
            let Some(record) = by_stem.get(member.as_str()) else {
                continue;
            };
            let line = record
                .file()
                .field_entries("blocked-by")
                .find(|(target, _)| target == next)
                .and_then(|(_, line)| line);
            out.push(FileFinding {
                path: record.path().to_owned(),
                finding: Finding::located(
                    line,
                    FindingCode::DepCycle,
                    format!("blocked-by: `{next}` closes the cycle {walk}"),
                ),
            });
        }
    }
}

/// Two files claiming one id, live and archive alike: ids are never reused,
/// and each file is told who else claims it.
fn check_duplicate_ids(records: &[Record], out: &mut Vec<FileFinding>) {
    let mut claims: BTreeMap<&str, Vec<&Record>> = BTreeMap::new();
    for record in records {
        if let Some(id) = record.id()
            && grammar::id_error(id).is_none()
        {
            claims.entry(id).or_default().push(record);
        }
    }
    for (id, claimants) in claims {
        if claimants.len() < 2 {
            continue;
        }
        for record in &claimants {
            let others: Vec<&str> = claimants
                .iter()
                .map(|other| other.path())
                .filter(|path| *path != record.path())
                .collect();
            out.push(FileFinding {
                path: record.path().to_owned(),
                finding: Finding::for_file(
                    FindingCode::DuplicateId,
                    format!("id `{id}` is also claimed by {}", others.join(", ")),
                ),
            });
        }
    }
}
