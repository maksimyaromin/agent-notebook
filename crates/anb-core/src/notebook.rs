//! The Notebook: the records under one root, read and mutated through
//! Storage.
//!
//! Write-time invariants live here, and they bind every author equally: a
//! declared supersession writes the back-pointer and flips the victim, a
//! Question closes only by routing or an explicit reasoned drop, a close
//! carries its proof. Every mutation is idempotent — a replayed call answers
//! `already: true` and leaves every byte of every file unchanged — and a
//! record carrying an error finding is never mutated and never rewritten.

use crate::finding::{Finding, FindingCode, Severity};
use crate::grammar::{self, RecordFile};
use crate::record::{Record, RecordType, TaskAction, TaskState, Transition};
use crate::storage::{Storage, StorageError};
use std::collections::BTreeMap;

/// The envelope keys whose values point at other records.
const REF_KEYS: [&str; 5] = [
    "from",
    "supersedes",
    "superseded-by",
    "routed-to",
    "blocked-by",
];

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

/// The auditable evidence a close carries (US6). `Waived` is the explicit
/// override: the caller states there is no proof rather than omitting it.
pub enum Proof {
    Pr(String),
    Sha(String),
    Report(String),
    Waived,
}

impl Proof {
    fn link_value(&self) -> Option<String> {
        match self {
            Proof::Pr(target) => Some(format!("pr {target}")),
            Proof::Sha(target) => Some(format!("sha {target}")),
            Proof::Report(target) => Some(format!("report {target}")),
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

/// A close, with the deferred findings it must not bury: the still-open
/// Questions born from this Task (US8).
#[derive(Debug, PartialEq, Eq)]
pub struct Closed {
    pub transition: Transitioned,
    pub open_questions: Vec<String>,
}

/// A hold set or cleared; `already` marks the replay.
#[derive(Debug, PartialEq, Eq)]
pub struct Held {
    pub id: String,
    pub already: bool,
}

/// A record created, with the victim its supersession flipped, if any.
#[derive(Debug, PartialEq, Eq)]
pub struct Created {
    pub id: String,
    pub path: String,
    pub superseded: Option<String>,
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
    /// commands it does allow (US5).
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
            NotebookError::DanglingRef { field, target } => {
                write!(f, "{field}: `{target}` names no record")
            }
            NotebookError::CannotSupersede { id, reason } => {
                write!(f, "cannot supersede `{id}`: {reason}")
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
    /// pairs, routing threads. Sorted by file, then line.
    ///
    /// # Errors
    /// A storage failure.
    pub fn check(&self) -> Result<Vec<FileFinding>, NotebookError> {
        let records = self.read_records()?;
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
            check_refs(record, &by_stem, &mut located);
            check_supersession_pair(record, &by_stem, &mut located);
        }
        check_duplicate_ids(&records, &mut located);

        located.sort_by(|left, right| finding_order(left).cmp(&finding_order(right)));
        Ok(located)
    }

    /// Create a record, minting an id from the title unless one is given.
    /// A draft declaring `supersedes` performs the whole supersession: the
    /// new record is written first, then the victim gains the back-pointer
    /// and flips to its superseded state — a rule recorded as replaced can
    /// never be read as live.
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

        let id = self.resolve_draft_id(draft)?;
        let path = record_path(&id, draft.record_type, false);
        self.storage
            .write(&path, &render_draft(draft, &id, today))?;

        if let Some(victim) = victim {
            let mut file = victim.record.into_file();
            file.set_field("superseded-by", &id);
            file.set_field("state", victim.dead_state);
            file.set_field("updated", today);
            self.storage.write(&victim.path, &file.render())?;
        }

        Ok(Created {
            id,
            path,
            superseded: draft.supersedes.clone(),
        })
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
        Ok(Closed {
            transition,
            open_questions: self.open_questions_from(id)?,
        })
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
    ) -> Result<Transitioned, NotebookError> {
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
            return Ok(replayed(id, "dropped"));
        }
        if state != "open" {
            return Err(settled_question(id, state));
        }
        let mut file = loaded.record.into_file();
        file.set_field("state", "dropped");
        file.set_field("updated", today);
        file.append_body(&format!("Dropped {today}: {reason}"));
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Transitioned {
            id: id.to_owned(),
            from: "open",
            to: "dropped",
            already: false,
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
        let record_type = parsed_type(id)?;
        if !expected.contains(&record_type) {
            return Err(NotebookError::WrongType {
                id: id.to_owned(),
                expected: type_list(expected),
            });
        }
        self.resolve_live(id, record_type)
    }

    /// The one gate every write passes: the record is live and carries no
    /// error finding, so no path — a verb or a supersession flip — can
    /// rewrite an invalid record.
    fn resolve_live(&self, id: &str, record_type: RecordType) -> Result<LoadedLive, NotebookError> {
        let path = record_path(id, record_type, false);
        let text = match self.storage.read(&path) {
            Ok(text) => text,
            Err(StorageError::NotFound { .. }) => {
                return Err(match self.holder_path(id)? {
                    Some(_) => NotebookError::Archived { id: id.to_owned() },
                    None => NotebookError::UnknownId { id: id.to_owned() },
                });
            }
            Err(error) => return Err(error.into()),
        };
        let record = Record::parse(&path, &text);
        let errors = self.mutation_errors(&record)?;
        if !errors.is_empty() {
            return Err(NotebookError::InvalidRecord {
                path,
                findings: errors,
            });
        }
        Ok(LoadedLive { path, record })
    }

    /// The error findings that exclude a record from mutation: its own,
    /// plus a dangling reference probed against storage.
    fn mutation_errors(&self, record: &Record) -> Result<Vec<Finding>, NotebookError> {
        let mut errors: Vec<Finding> = record
            .findings()
            .iter()
            .filter(|finding| finding.code.severity() == Severity::Error)
            .cloned()
            .collect();
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

    /// The draft's id: the caller's, validated and free, or one minted from
    /// the title — retried with a two-character suffix on collision, since
    /// ids are never reused.
    fn resolve_draft_id(&self, draft: &Draft) -> Result<String, NotebookError> {
        let claims = self.id_claims()?;
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
    fn id_claims(&self) -> Result<BTreeMap<String, String>, NotebookError> {
        let mut claims = BTreeMap::new();
        for record in self.read_records()? {
            claims
                .entry(path_stem(record.path()).to_owned())
                .or_insert_with(|| record.path().to_owned());
            if let Some(id) = record.id() {
                claims
                    .entry(id.to_owned())
                    .or_insert_with(|| record.path().to_owned());
            }
        }
        Ok(claims)
    }

    /// Where `id` lives, if anywhere: its live path, else its archive path.
    fn holder_path(&self, id: &str) -> Result<Option<String>, NotebookError> {
        let Some(record_type) = id
            .split_once('.')
            .and_then(|(word, _)| RecordType::from_word(word))
        else {
            return Ok(None);
        };
        for archived in [false, true] {
            let path = record_path(id, record_type, archived);
            match self.storage.read(&path) {
                Ok(_) => return Ok(Some(path)),
                Err(StorageError::NotFound { .. }) => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(None)
    }

    /// The still-open Questions whose Origin is this Task.
    fn open_questions_from(&self, task_id: &str) -> Result<Vec<String>, NotebookError> {
        let mut ids = Vec::new();
        for path in self.storage.list(RecordType::Question.directory())? {
            if !is_record_file(&path) {
                continue;
            }
            let file = RecordFile::parse(&self.storage.read(&path)?);
            if file.field("from") == Some(task_id) && file.field("state") == Some("open") {
                ids.push(path_stem(&path).to_owned());
            }
        }
        Ok(ids)
    }

    /// Every record, live and archived, invalid ones included: an invalid
    /// record is a visible first-class state, never a silent drop.
    fn read_records(&self) -> Result<Vec<Record>, NotebookError> {
        let mut records = Vec::new();
        for record_type in RecordType::ALL {
            let live = record_type.directory().to_owned();
            let dirs = [live.clone(), format!("archive/{live}")];
            for dir in dirs {
                for path in self.storage.list(&dir)? {
                    if !is_record_file(&path) {
                        continue;
                    }
                    let text = self.storage.read(&path)?;
                    records.push(Record::parse(&path, &text));
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

/// A proof is a link value: one non-empty line, or an explicit waiver.
fn guard_proof(proof: &Proof) -> Result<(), NotebookError> {
    let (Proof::Pr(target) | Proof::Sha(target) | Proof::Report(target)) = proof else {
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

/// File, then line (file-level findings last), then code.
fn finding_order(located: &FileFinding) -> (&str, usize, &'static str) {
    (
        located.path.as_str(),
        located.finding.line.unwrap_or(usize::MAX),
        located.finding.code.as_str(),
    )
}

fn path_stem(path: &str) -> &str {
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

    let body = draft.body.strip_suffix('\n').unwrap_or(&draft.body);
    if !body.is_empty() {
        file.append_body("");
        for line in body.split('\n') {
            file.append_body(line);
        }
    }
    file.render()
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.truncate(40);
    slug.trim_end_matches('-').to_owned()
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

fn check_refs(record: &Record, by_stem: &BTreeMap<&str, &Record>, out: &mut Vec<FileFinding>) {
    for key in REF_KEYS {
        for (target, line) in record.file().field_entries(key) {
            if grammar::id_error(target).is_some() || by_stem.contains_key(target) {
                continue;
            }
            out.push(FileFinding {
                path: record.path().to_owned(),
                finding: dangling_finding(record, key, target, line),
            });
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
