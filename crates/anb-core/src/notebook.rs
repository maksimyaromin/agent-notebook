//! The Notebook: the records under one root, read and mutated through
//! Storage.
//!
//! Write-time invariants live here, and they bind every author equally: a
//! declared supersession writes the back-pointer and flips the victim, a
//! Question closes only by routing or an explicit reasoned drop, a close
//! carries its proof. A verb that moves a record already in the notebook is
//! idempotent — a replayed call answers `already: true` and leaves every
//! byte of every file unchanged. Creating and expunging are not replays of
//! anything: a second `add` of the same title mints a second record, and a
//! second `expunge` names an id the notebook no longer holds.
//!
//! The mutation gate holds a record's own error findings and its dangling
//! references against it; findings that need a second record — a broken
//! supersession pair, a multi-file dependency cycle — are `check`'s alone
//! and freeze nothing. The one way past the gate is a repair: `unblock`
//! and `edit` read a record over its error findings and are judged on the
//! bytes they produce, so a corrupted line cannot freeze the verb that
//! clears it and a half-repair is refused like any other invalid write.
//!
//! The verbs are here, and with them every read and write that touches
//! Storage. The children hold what a verb needs and Storage does not
//! reach: `write` the guards a request passes and the bytes it becomes,
//! `query` the folds over records already read, `error` the refusals.
//! `check` is the exception, and earns it — verifying the notebook is
//! reading all of it, so the verb lives beside the rules it applies.

mod check;
mod error;
mod query;
mod write;

pub use error::NotebookError;

use crate::config::{CONFIG_PATH, Config};
use crate::debt::{self, DebtSources};
use crate::encode;
use crate::finding::Finding;
use crate::grammar::{self, RecordFile, Residence};
use crate::graph;
use crate::mention;
use crate::record::{
    REF_KEYS, Record, RecordType, TaskAction, TaskState, Transition, dangling_finding,
    not_utf8_finding,
};
use crate::reply::{
    Archived, CitedProof, Closed, Commented, Counts, Created, Dropped, Edged, Edited, Expunged,
    Held, ListedRecord, Overview, ReadyTask, Transitioned, TypeSection, View,
};
use crate::request::{Draft, Edit, Proof};
use crate::resolve::{
    Resolver, archive_of, archived_among, canonical_paths, is_archived, is_record_file, path_stem,
    record_path, resolvable_id,
};
use crate::status::{self, Budget, Status, StatusInputs};
use crate::storage::{Storage, StorageError};
use std::collections::BTreeSet;

/// A report landed in the notebook, and what its body cited.
struct IngestedReport {
    id: String,
    dangling_mentions: Vec<String>,
}

pub struct Notebook<'a> {
    storage: &'a mut dyn Storage,
}

impl<'a> Notebook<'a> {
    pub fn new(storage: &'a mut dyn Storage) -> Self {
        Notebook { storage }
    }

    /// Read one record by id, live or archived.
    ///
    /// # Errors
    /// [`NotebookError::UnknownId`], [`NotebookError::InvalidArgument`] on a
    /// malformed id, or a storage failure.
    pub fn record(&self, id: &str) -> Result<Record, NotebookError> {
        write::parsed_type(id)?;
        match self.holder_path(id)? {
            Some(path) => {
                let text = self.storage.read(&path)?;
                Ok(Record::parse(&path, &text))
            }
            None => Err(NotebookError::UnknownId { id: id.to_owned() }),
        }
    }

    /// The dispatch queue: open, unblocked, unheld Tasks, the most
    /// urgent first. Invalid records are excluded, as from every derived
    /// query; `check` names them.
    ///
    /// # Errors
    /// A storage failure.
    pub fn ready(&self) -> Result<Vec<ReadyTask>, NotebookError> {
        let corpus = self.live_corpus()?;
        Ok(query::ready_rows(&corpus.records, &corpus.resolver()))
    }

    /// [`Notebook::ready`] narrowed to one hub's scope: the queue for
    /// "what is next inside this epic".
    ///
    /// # Errors
    /// [`NotebookError::UnknownId`] when the hub names no record,
    /// [`NotebookError::InvalidArgument`] on a malformed id, or a storage
    /// failure.
    pub fn ready_for(&self, hub: &str) -> Result<Vec<ReadyTask>, NotebookError> {
        let (corpus, scope) = self.scoped(hub)?;
        Ok(query::ready_rows(&corpus.records, &corpus.resolver())
            .into_iter()
            .filter(|row| scope.contains(&row.id))
            .collect())
    }

    /// [`Notebook::list`] narrowed to one hub's scope, the hub itself
    /// included: what a reader opening an epic wants to see.
    ///
    /// # Errors
    /// See [`Notebook::ready_for`].
    pub fn list_for(&self, hub: &str) -> Result<Vec<ListedRecord>, NotebookError> {
        let (corpus, scope) = self.scoped(hub)?;
        let resolvable = corpus.resolver();
        Ok(corpus
            .records
            .iter()
            .filter(|record| scope.contains(path_stem(record.path())))
            .map(|record| query::listed_row(record, &resolvable))
            .collect())
    }

    /// The whole notebook and the ids inside `hub`'s scope. A scope narrows
    /// what is *shown*, never what is *read*: a row is blocked, excluded and
    /// resolved against every record, so scoping a query cannot change any
    /// row's verdict — only which rows reach the reader.
    fn scoped(&self, hub: &str) -> Result<(Corpus, BTreeSet<String>), NotebookError> {
        write::parsed_type(hub)?;
        if self.holder_path(hub)?.is_none() {
            return Err(NotebookError::UnknownId { id: hub.to_owned() });
        }
        let corpus = self.live_corpus()?;
        let kin = self.archived_kin(&corpus)?;
        let scope = query::MembershipIndex::of(&corpus.records, &kin)
            .scope_of(hub)
            .into_iter()
            .map(str::to_owned)
            .collect();
        Ok((corpus, scope))
    }

    /// The archived records the live ones still name as a blocker or an
    /// Origin, and the ones those name in turn.
    ///
    /// An epic archives its children as they settle, so a hub read from the
    /// live records alone would forget its own progress and, in the end,
    /// that it was ever a hub; and a live record born inside an archived
    /// one belongs to the same epic through a lineage only that archived
    /// record spells out.
    fn archived_kin(&self, corpus: &Corpus) -> Result<Vec<Record>, NotebookError> {
        let archived = corpus.resolver();
        let wanted = archived_among(corpus.records.iter().flat_map(query::kin_of), &archived);
        self.kin_closure(&archived, Vec::new(), wanted)
    }

    /// The kin an epic asks for — and, when the notebook keeps no epic,
    /// only the one hop that establishes there is none.
    ///
    /// A hub names its own children on its `blocked-by` lines, so whether
    /// the notebook holds an epic at all is settled one step into the
    /// archive. Everything behind those children answers a second question
    /// — where a live record sits inside an epic — which a notebook without
    /// one never asks. So a dashboard reads history only once there is an
    /// epic to spend it on.
    fn epic_kin(&self, corpus: &Corpus) -> Result<Vec<Record>, NotebookError> {
        let archived = corpus.resolver();
        let children = archived_among(
            corpus
                .records
                .iter()
                .flat_map(|record| record.file().field_values("blocked-by")),
            &archived,
        );
        let mut declared = Vec::new();
        for id in children {
            if let Some(record) = self.archived_record(&id)? {
                declared.push(record);
            }
        }
        if !query::any_hub(&corpus.records, &declared, &archived) {
            return Ok(declared);
        }
        let wanted = archived_among(
            corpus
                .records
                .iter()
                .chain(declared.iter())
                .flat_map(query::kin_of),
            &archived,
        );
        self.kin_closure(&archived, declared, wanted)
    }

    /// Read the archived records `wanted` names, and those they name in
    /// turn, beside the `kin` already read.
    ///
    /// The walk leaves the live notebook along declared edges, so it costs
    /// the history still connected to today rather than the archive's size
    /// — how much that is, is how much of the archive the live records
    /// still point at. And it follows edges the way they are written: an
    /// archived record that only *carries* an Origin into the walk, one
    /// born inside a member and named by nothing live, is never reached,
    /// and a live record waiting on it alone falls outside the scope.
    /// Finding it would mean opening the archive to read Origins backwards,
    /// which is the cost this avoids.
    fn kin_closure(
        &self,
        archived: &Resolver<'_>,
        mut kin: Vec<Record>,
        mut wanted: Vec<String>,
    ) -> Result<Vec<Record>, NotebookError> {
        let mut seen: BTreeSet<String> = kin
            .iter()
            .map(|record| path_stem(record.path()).to_owned())
            .collect();
        while let Some(id) = wanted.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            let Some(record) = self.archived_record(&id)? else {
                continue;
            };
            wanted.extend(archived_among(query::kin_of(&record), archived));
            kin.push(record);
        }
        Ok(kin)
    }

    /// One archived record by id; `None` when the archive holds no such
    /// file, or the id names no type at all.
    fn archived_record(&self, id: &str) -> Result<Option<Record>, NotebookError> {
        let Ok(record_type) = write::parsed_type(id) else {
            return Ok(None);
        };
        let path = record_path(id, record_type, true);
        match self.storage.read(&path) {
            Ok(text) => Ok(Some(Record::parse(&path, &text))),
            Err(StorageError::NotUtf8 { .. }) => Ok(Some(Record::unreadable(&path))),
            Err(StorageError::NotFound { .. }) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// The session-start dashboard under `budget`, gated: one quiet line
    /// when the notebook carries no signal, the full budgeted composite
    /// otherwise. The caller resolves `budget` — a CLI flag outranks the
    /// config key, which defaults to 1500.
    ///
    /// `settle` is the host's answer to "which of these proofs does the
    /// world still hold": it is handed the proofs the live records cite and
    /// returns the lost ones. It is asked from the records already read, so
    /// a Status costs the notebook one pass and not two.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on a malformed `today`, or a
    /// storage failure.
    pub fn status(
        &self,
        today: &str,
        budget: Budget,
        settle: impl FnOnce(&[CitedProof]) -> Vec<CitedProof>,
    ) -> Result<Status, NotebookError> {
        let today_day = write::guarded_day(today)?;
        let thresholds = self.config()?.debt_thresholds();
        let corpus = self.live_corpus()?;
        let records = &corpus.records;
        let resolvable = corpus.resolver();
        let live_valid: Vec<&Record> = records
            .iter()
            .filter(|record| !debt::is_excluded(record, &resolvable))
            .collect();

        let lost = settle(&query::cited_proofs(records));
        let sources = DebtSources {
            records,
            resolvable: &resolvable,
            today_day,
            lost_proofs: &lost,
        };
        let queue = query::ready_rows(records, &resolvable);
        let inputs = StatusInputs {
            counts: query::live_counts(records),
            in_flight: query::in_flight_tasks(&live_valid),
            review: query::review_tasks(&live_valid),
            rules: query::standing_rules(&live_valid),
            epics: query::epic_rows(records, &self.epic_kin(&corpus)?, &resolvable, &queue),
            ready: queue,
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
        let corpus = self.live_corpus()?;
        let resolvable = corpus.resolver();
        Ok(corpus
            .records
            .iter()
            .map(|record| query::listed_row(record, &resolvable))
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
        let corpus = self.whole_corpus()?;
        let resolvable = corpus.resolver();
        Ok(corpus
            .records
            .iter()
            .filter(|record| query::matches_query(record, &needle))
            .map(|record| query::listed_row(record, &resolvable))
            .collect())
    }

    /// The whole notebook as one page: every live record grouped by type,
    /// with the archive reduced to counts. The model is complete; what a
    /// reader is shown of it is the reply's to bound.
    ///
    /// # Errors
    /// A storage failure.
    pub fn overview(&self) -> Result<Overview, NotebookError> {
        let corpus = self.live_corpus()?;
        let records = &corpus.records;
        let resolvable = corpus.resolver();
        let sections = RecordType::ALL
            .into_iter()
            .map(|record_type| TypeSection {
                record_type,
                rows: records
                    .iter()
                    .filter(|record| query::sits_in(record, record_type, Residence::Live))
                    .map(|record| query::listed_row(record, &resolvable))
                    .collect(),
            })
            .collect();
        let queue = query::ready_rows(records, &resolvable);
        Ok(Overview {
            live: query::live_counts(records),
            epics: query::epic_rows(records, &self.epic_kin(&corpus)?, &resolvable, &queue),
            sections,
            archived: corpus.archive,
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
        let live = self.live_corpus()?;
        let mentions = mention::mentions(record.file().body())
            .into_iter()
            .filter(|target| *target != id)
            .map(str::to_owned)
            .collect();
        let mentioned_by = live
            .records
            .iter()
            .filter(|other| path_stem(other.path()) != id)
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
        write::guard_today(today)?;
        write::validate_draft(draft)?;
        if let Some(origin) = &draft.from {
            self.guard_ref_exists("from", origin)?;
        }
        let victim = self.guard_supersession(draft)?;

        let corpus = self.live_corpus()?;
        let records = &corpus.records;
        let id = write::resolve_draft_id(draft, &write::id_claims(records, &corpus.archived))?;
        let may_conflict = query::conflict_candidates(draft, records, &corpus.resolver());
        let path = record_path(&id, draft.record_type, false);
        self.storage
            .write(&path, &write::render_draft(draft, &id, today))?;

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
        write::guard_proof(proof)?;
        let transition = self.task_transition(id, TaskAction::Close, today, |file| {
            file.set_field("closed", today);
            if let Some(link) = proof.link_value() {
                file.append_field("link", &link);
            }
        })?;
        let corpus = self.live_corpus()?;
        let records = &corpus.records;
        let resolvable = corpus.resolver();
        Ok(Closed {
            transition,
            open_questions: query::open_questions_from(records, &resolvable, id),
            unblocked: query::unblocked_by_close(records, &resolvable, id),
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
    /// its own interrupted run left behind, recognised by the link the
    /// Task already carries.
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
        let title = write::report_note_title(task.record.file().field("title").unwrap_or(id));
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
        let wanted = write::edited_body(report);
        Ok(self
            .records_in(RecordType::Note.directory())?
            .into_iter()
            .find(|note| note.origin() == Some(origin) && note.file().body() == wanted)
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
        write::guard_today(today)?;
        let reason = reason.trim();
        write::guard_single_line("hold", reason)?;
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
    /// It repairs as well as resumes, because it erases those two lines by
    /// name: a hand-broken hold has no other way out of the notebook.
    ///
    /// # Errors
    /// The resolution errors of [`Notebook::close`].
    pub fn unhold(&mut self, id: &str, today: &str) -> Result<Held, NotebookError> {
        write::guard_today(today)?;
        let repairing = self.load_live_repairing(id, &[RecordType::Task])?;
        let held = repairing.record().hold().is_some() || repairing.record().hold_until().is_some();
        let resumed = self.commit_repair(repairing, today, |file| {
            held.then(|| {
                file.remove_field("hold");
                file.remove_field("hold-until");
            })
        })?;
        Ok(Held {
            id: id.to_owned(),
            already: resumed.is_none(),
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
        write::guard_today(today)?;
        let text = text.trim();
        write::guard_single_line("comment", text)?;
        if text.is_empty() {
            return Err(NotebookError::InvalidArgument {
                reason: "comment: the text must not be empty".to_owned(),
            });
        }
        let author = author.map(str::trim).filter(|name| !name.is_empty());
        if let Some(author) = author {
            write::guard_single_line("author", author)?;
        }

        let entry = format!("- {today} {}: {text}", author.unwrap_or("-"));
        let loaded = self.load_live(id, &[RecordType::Task])?;
        // A replay carries the nudge too: the entry is the trail's tail, so
        // its citations stand in the body either way.
        let dangling_mentions = self.dangling_mentions(text)?;
        if query::last_log_line(&loaded.record).as_deref() == Some(entry.as_str()) {
            return Ok(Commented {
                id: id.to_owned(),
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
        write::guard_today(today)?;
        if write::parsed_type(on)? != RecordType::Task {
            return Err(NotebookError::WrongType {
                id: on.to_owned(),
                expected: "a task".to_owned(),
            });
        }
        self.guard_ref_exists("blocked-by", on)?;
        let loaded = self.load_live(id, &[RecordType::Task])?;
        if query::edge_exists(&loaded.record, on) {
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
    /// and a corrupted edge does not freeze the verb that erases it, as
    /// long as erasing it leaves the record clean. An edge hand-edited into
    /// duplicates is erased whole, so the replay stays true.
    ///
    /// # Errors
    /// The resolution errors of [`Notebook::close`].
    pub fn unblock(&mut self, id: &str, on: &str, today: &str) -> Result<Edged, NotebookError> {
        write::guard_today(today)?;
        write::parsed_type(on)?;
        let repairing = self.load_live_repairing(id, &[RecordType::Task])?;
        let stands = query::edge_exists(repairing.record(), on);
        let erased = self.commit_repair(repairing, today, |file| {
            stands.then(|| file.remove_field_value("blocked-by", on))
        })?;
        Ok(Edged {
            id: id.to_owned(),
            on: on.to_owned(),
            already: erased.is_none(),
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
        write::guard_today(today)?;
        let target_type = write::parsed_type(to)?;
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
            return Ok(Transitioned::replayed(id, "routed"));
        }
        if state != "open" {
            return Err(error::settled_question(id, state));
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
        write::guard_today(today)?;
        let reason = reason.trim();
        write::guard_single_line("drop reason", reason)?;
        if reason.is_empty() {
            return Err(NotebookError::InvalidArgument {
                reason: "drop: the reason must not be empty".to_owned(),
            });
        }

        let loaded = self.load_live(id, &[RecordType::Question])?;
        let state = loaded.state_word();
        if state == "dropped" {
            return Ok(Dropped {
                transition: Transitioned::replayed(id, "dropped"),
                dangling_mentions: Vec::new(),
            });
        }
        if state != "open" {
            return Err(error::settled_question(id, state));
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
        write::guard_today(today)?;
        let loaded = self.load_live(id, &[RecordType::Decision, RecordType::Note])?;
        let state = loaded.state_word();
        if state == "retired" {
            return Ok(Transitioned::replayed(id, "retired"));
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

    /// Move a settled record into the archive, and carry its reports with
    /// it: same filename, same bytes, so `git log --follow` keeps its
    /// history and the round-trip contract holds. Each archive copy lands
    /// before its live file goes — a failure between the two leaves a loud
    /// `duplicate-id`, never a lost record.
    ///
    /// The reports are the record's own history, so filing one without the
    /// other is a split no reader can act on; carried along, the live
    /// notebook stays the size of the work still open. Everything else a
    /// record spawned stays where it is — an open Question born inside a
    /// closed Task is a debt the dashboard raises, not history.
    ///
    /// # Errors
    /// [`NotebookError::InvalidTransition`] on a record still live, naming
    /// the commands that settle it, [`NotebookError::InvalidRecord`] or
    /// [`NotebookError::DuplicateId`] on a report that cannot be filed —
    /// answered before anything moves — plus the resolution errors of
    /// [`Notebook::close`].
    pub fn archive(&mut self, id: &str, today: &str) -> Result<Archived, NotebookError> {
        write::guard_today(today)?;
        let record_type = write::parsed_type(id)?;
        let mut moved = Archived {
            id: id.to_owned(),
            from: record_path(id, record_type, false),
            to: record_path(id, record_type, true),
            carried: Vec::new(),
            already: false,
        };
        let loaded = match self.resolve_live(id, record_type) {
            Ok(loaded) => loaded,
            // The record is already where it belongs, and whatever
            // travelled with it travelled then: a second call is a
            // question, not a move.
            Err(NotebookError::Archived { .. }) => {
                moved.already = true;
                self.replayed_archive(&moved)?;
                return Ok(moved);
            }
            Err(error) => return Err(error),
        };
        if loaded.record.is_live() {
            return Err(NotebookError::InvalidTransition {
                id: id.to_owned(),
                state: loaded.state_word().to_owned(),
                valid: error::settling_commands(record_type, loaded.state_word()),
            });
        }
        let reports = self.carriable_reports(&loaded.record)?;
        self.guard_archive_destination_free(id, &moved.to, loaded.record.file())?;
        // The reports move first, so a run interrupted inside the cascade
        // leaves the record live and the next call finishes it.
        for report in reports {
            moved.carried.push(self.file_report(report, today)?);
        }
        self.storage
            .write(&moved.to, &loaded.record.file().render())?;
        self.storage.remove(&moved.from)?;
        Ok(moved)
    }

    /// The reports `filed` carries that this move may take with it, each
    /// read and judged before a single byte moves — the whole cascade is
    /// decided first, so a report the notebook cannot move refuses the
    /// archive instead of interrupting it.
    ///
    /// A report is a Note this record links and that was born inside it:
    /// the record's own output, which is what makes it that record's
    /// history rather than knowledge outliving it. A link naming anything
    /// else — a Note born elsewhere, one already filed, a target that is no
    /// record id at all — is left alone, because a link is evidence and
    /// evidence is not a licence to move another record.
    fn carriable_reports(&self, filed: &Record) -> Result<Vec<Report>, NotebookError> {
        let origin = path_stem(filed.path());
        let mut reports = Vec::new();
        for id in query::note_links(filed) {
            let path = record_path(&id, RecordType::Note, false);
            let Ok(text) = self.storage.read(&path) else {
                continue;
            };
            let record = Record::parse(&path, &text);
            if record.origin() != Some(origin) {
                continue;
            }
            if record.has_errors() {
                return Err(NotebookError::InvalidRecord {
                    path,
                    findings: record.error_findings(),
                });
            }
            let destination = record_path(&id, RecordType::Note, true);
            self.guard_report_destination(&destination, origin)?;
            reports.push(Report {
                path,
                record,
                destination,
            });
        }
        Ok(reports)
    }

    /// Refuse the report's place in the archive when another record holds
    /// it. Free is a place this move may take, and so is one holding this
    /// origin's own report from a run that crashed mid-cascade.
    ///
    /// A report is retired as it is filed and stamped with the day it
    /// moved, so it can never equal the live bytes the way an unchanged
    /// record does: the interrupted move is recognised by what stands
    /// there — this origin's report, retired, sound — and never by what it
    /// says. What it says is the live copy's to decide, which is why the
    /// move writes it again instead of trusting what it found.
    fn guard_report_destination(
        &self,
        destination: &str,
        origin: &str,
    ) -> Result<(), NotebookError> {
        let text = match self.storage.read(destination) {
            Ok(text) => text,
            Err(StorageError::NotFound { .. }) => return Ok(()),
            // Bytes no parse can read stand for a record that is not this
            // report, which is the refusal below.
            Err(StorageError::NotUtf8 { .. }) => String::new(),
            Err(error) => return Err(error.into()),
        };
        let standing = Record::parse(destination, &text);
        if standing.origin() == Some(origin)
            && standing.state() == Some("retired")
            && !standing.has_errors()
        {
            return Ok(());
        }
        Err(NotebookError::DuplicateId {
            id: path_stem(destination).to_owned(),
            holder: destination.to_owned(),
        })
    }

    /// Retire a report and file it in one move: a Note is current knowledge
    /// until something ends it, and what ends this one is the record it
    /// reports on becoming history. Only the Note this call already judged
    /// moves — a report of its own is not followed, so the cascade is one
    /// record deep.
    fn file_report(&mut self, report: Report, today: &str) -> Result<String, NotebookError> {
        let id = path_stem(&report.path).to_owned();
        let mut file = report.record.into_file();
        file.set_field("state", "retired");
        file.set_field("updated", today);
        self.storage.write(&report.destination, &file.render())?;
        self.storage.remove(&report.path)?;
        Ok(id)
    }

    /// Judge the copy already in the archive before calling the move done.
    /// Only a clean one proves this very move happened; on any error finding
    /// — the same gate every write passes — answering `already` would report
    /// a corruption as a success.
    fn replayed_archive(&self, moved: &Archived) -> Result<Record, NotebookError> {
        let record = match self.storage.read(&moved.to) {
            Ok(text) => Record::parse(&moved.to, &text),
            Err(StorageError::NotUtf8 { .. }) => Record::unreadable(&moved.to),
            Err(error) => return Err(error.into()),
        };
        if !record.has_errors() {
            return Ok(record);
        }
        Err(NotebookError::InvalidRecord {
            path: moved.to.clone(),
            findings: record.error_findings(),
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
        write::parsed_type(id)?;
        let paths = self.holder_paths(id)?;
        if paths.is_empty() {
            return Err(NotebookError::UnknownId { id: id.to_owned() });
        }
        let blockers = query::inbound_edges(&self.whole_corpus()?.records, id);
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
        write::guard_today(today)?;
        let record_type = write::parsed_type(id)?;
        let cleared = write::validate_edit(record_type, edit)?;
        if let Some(origin) = &edit.from {
            if origin == id {
                return Err(NotebookError::InvalidArgument {
                    reason: "from: a record cannot be its own origin".to_owned(),
                });
            }
            self.guard_ref_exists("from", origin)?;
            self.guard_lineage_stays_open(id, origin)?;
        }

        let repairing = self.resolve_live_repairing(id, record_type)?;
        let dangling_mentions = match &edit.body {
            Some(body) => self.dangling_mentions(body)?,
            None => Vec::new(),
        };
        let changed = self.commit_repair(repairing, today, |file| {
            let changed = write::spliced(file, edit, &cleared);
            (!changed.is_empty()).then_some(changed)
        })?;
        Ok(Edited {
            id: id.to_owned(),
            changed: changed.unwrap_or_default(),
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
        write::guard_today(today)?;
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
            return Ok(Transitioned::replayed(id, state.word()));
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
        self.resolve_live(id, commanded_type(id, expected)?)
    }

    /// [`Notebook::load_live`] for a repair: the record is read over its
    /// error findings, and the only way on from there is
    /// [`Notebook::commit_repair`].
    fn load_live_repairing(
        &self,
        id: &str,
        expected: &[RecordType],
    ) -> Result<Repairing, NotebookError> {
        self.resolve_live_repairing(id, commanded_type(id, expected)?)
    }

    /// The one gate every write passes: the record is live and carries no
    /// error finding, so no path — a verb or a supersession flip — can
    /// rewrite an invalid record.
    fn resolve_live(&self, id: &str, record_type: RecordType) -> Result<LoadedLive, NotebookError> {
        let (loaded, errors) = self.read_live(id, record_type)?;
        if errors.is_empty() {
            return Ok(loaded);
        }
        Err(NotebookError::InvalidRecord {
            path: loaded.path,
            findings: errors,
        })
    }

    /// [`Notebook::resolve_live`] for a verb that repairs: it is let past
    /// the record's error findings and judged on the bytes it produces
    /// instead. A file with no envelope is not let past, because splicing
    /// one is not defined.
    fn resolve_live_repairing(
        &self,
        id: &str,
        record_type: RecordType,
    ) -> Result<Repairing, NotebookError> {
        let (loaded, errors) = self.read_live(id, record_type)?;
        if !errors.is_empty() && !loaded.record.file().has_envelope() {
            return Err(NotebookError::InvalidRecord {
                path: loaded.path,
                findings: errors,
            });
        }
        Ok(Repairing {
            loaded,
            carried: errors,
        })
    }

    /// The live record `id` names, beside the error findings that exclude
    /// it from mutation — read once here, because the two gates above
    /// differ only in what they do with them.
    fn read_live(
        &self,
        id: &str,
        record_type: RecordType,
    ) -> Result<(LoadedLive, Vec<Finding>), NotebookError> {
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
        let errors = self.exclusion_errors(&record)?;
        Ok((LoadedLive { path, record }, errors))
    }

    /// A repairing verb's whole write, and the only way a [`Repairing`]
    /// reaches storage: the splice runs on the record that was read, the
    /// bytes it produced are judged, and a splice that moved nothing is
    /// written nowhere. `splice` answers `None` when the record already
    /// reads as asked — the replay — and otherwise whatever the verb needs
    /// to report.
    ///
    /// The obligation cannot be forgotten because it is not the caller's:
    /// [`Repairing`] hands out no bytes to write.
    fn commit_repair<T>(
        &mut self,
        repairing: Repairing,
        today: &str,
        splice: impl FnOnce(&mut RecordFile) -> Option<T>,
    ) -> Result<Option<T>, NotebookError> {
        let Repairing {
            loaded: LoadedLive { path, record },
            carried,
        } = repairing;
        let mut file = record.into_file();
        let moved = splice(&mut file);
        if moved.is_some() {
            file.set_field("updated", today);
        }
        if !carried.is_empty() {
            self.guard_repaired(&path, &carried, &file)?;
        }
        if moved.is_some() {
            self.storage.write(&path, &file.render())?;
        }
        Ok(moved)
    }

    /// The repair's other half: a verb let past a record's error findings
    /// must leave it better than it found it — fewer of them, and none it
    /// did not already carry — or the call is refused and nothing is
    /// written. The proof is the bytes the splice produced: a line that
    /// carries a finding is not always the line a splice rewrites, and a
    /// splice that reaches the right line may still leave a duplicate of it
    /// standing.
    ///
    /// Progress, not perfection, because a record broken several ways is
    /// repaired one verb at a time, and each verb erases only what it
    /// names. A call that erases nothing is refused all the same, which is
    /// why this runs before the replay answers: a record that came in
    /// invalid must not be told `already` while it still is.
    fn guard_repaired(
        &self,
        path: &str,
        before: &[Finding],
        file: &RecordFile,
    ) -> Result<(), NotebookError> {
        let repaired = Record::parse(path, &file.render());
        let after = self.exclusion_errors(&repaired)?;
        let already_carried = |finding: &Finding| {
            before
                .iter()
                .any(|had| had.code == finding.code && had.message == finding.message)
        };
        if after.len() < before.len() && after.iter().all(already_carried) {
            return Ok(());
        }
        Err(NotebookError::InvalidRecord {
            path: path.to_owned(),
            findings: after,
        })
    }

    /// The cycle the edge `id → on` would close, walked in full: `on`
    /// already waits on `id`, or is `id` itself.
    fn would_cycle(&self, id: &str, on: &str) -> Result<Option<Vec<String>>, NotebookError> {
        if id == on {
            return Ok(Some(vec![id.to_owned(), on.to_owned()]));
        }
        let Some(waited_on) = graph::chain(on, id, |at| self.blockers_of(at))? else {
            return Ok(None);
        };
        let mut cycle = vec![id.to_owned()];
        cycle.extend(waited_on);
        Ok(Some(cycle))
    }

    /// Refuse an Origin that would close a lineage loop: `origin` was
    /// already born inside `id`, however many births ago. Origin is the edge
    /// Debt keys its clocks on and the edge an epic's scope follows, so a
    /// record standing inside its own lineage is a record with no birth —
    /// and `edit` is the one verb that can write it, since a record minted
    /// with `--from` does not exist yet to be anyone's origin.
    fn guard_lineage_stays_open(&self, id: &str, origin: &str) -> Result<(), NotebookError> {
        let Some(lineage) = graph::chain(origin, id, |at| self.origin_of(at))? else {
            return Ok(());
        };
        Err(NotebookError::InvalidArgument {
            reason: format!(
                "from: `{origin}` is already born inside `{id}` \u{2014} {}",
                encode::id_chain(&lineage)
            ),
        })
    }

    /// The record `at` was born from, read from wherever `at`'s file sits —
    /// the archive included, since a lineage runs back through history. A
    /// name no record bears was born of nothing; the walk takes at most one
    /// step per record, because a record has at most one Origin.
    fn origin_of(&self, at: &str) -> Result<Vec<String>, NotebookError> {
        for path in canonical_paths(at) {
            let text = match self.storage.read(&path) {
                Ok(text) => text,
                Err(StorageError::NotFound { .. } | StorageError::NotUtf8 { .. }) => continue,
                Err(error) => return Err(error.into()),
            };
            return Ok(Record::parse(&path, &text)
                .origin()
                .filter(|origin| grammar::id_error(origin).is_none())
                .map(str::to_owned)
                .into_iter()
                .collect());
        }
        Ok(Vec::new())
    }

    /// What the Task filed under the name `at` waits on, read from its live
    /// home before its archived one — a cycle through history is still a
    /// cycle. The Task directories are the whole graph, whatever a file
    /// there is named: a record whose name says one type and whose bytes
    /// say Task carries real edges, and `check` names the mismatch
    /// separately. A name no Task file there bears waits on nothing.
    fn blockers_of(&self, at: &str) -> Result<Vec<String>, NotebookError> {
        for archived in [false, true] {
            let path = record_path(at, RecordType::Task, archived);
            let text = match self.storage.read(&path) {
                Ok(text) => text,
                Err(StorageError::NotFound { .. } | StorageError::NotUtf8 { .. }) => continue,
                Err(error) => return Err(error.into()),
            };
            let record = Record::parse(&path, &text);
            if record.record_type() != Some(RecordType::Task) {
                continue;
            }
            return Ok(record.blocked_by().map(str::to_owned).collect());
        }
        Ok(Vec::new())
    }

    /// The error findings that exclude a record from mutation: its own,
    /// plus a dangling reference probed against storage — the same rule the
    /// derived queries answer from the resolvable map, adapted here to a
    /// single record so the gate never has to read the whole notebook.
    fn exclusion_errors(&self, record: &Record) -> Result<Vec<Finding>, NotebookError> {
        let mut errors = record.error_findings();
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
        let victim_type = write::parsed_type(target)?;
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
        let LoadedLive { path, record, .. } = loaded;
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
    /// The archive is probed only when the live home is empty — one probe
    /// answers the question the caller asked.
    fn holder_path(&self, id: &str) -> Result<Option<String>, NotebookError> {
        for path in canonical_paths(id) {
            if self.storage.exists(&path)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    /// Every canonical path holding `id`, live before archived. Two is the
    /// `duplicate-id` corruption an interrupted archive move leaves, and
    /// only expunge, which must leave nothing behind, needs them all.
    fn holder_paths(&self, id: &str) -> Result<Vec<String>, NotebookError> {
        let mut holders = Vec::new();
        for path in canonical_paths(id) {
            if self.storage.exists(&path)? {
                holders.push(path);
            }
        }
        Ok(holders)
    }

    /// The whole notebook read: history's bytes among the rest. The corpus
    /// of the verbs that must judge what an archived file says — verify it,
    /// search it, or refuse an id it already claims.
    fn whole_corpus(&self) -> Result<Corpus, NotebookError> {
        let mut records = Vec::new();
        let mut held = [0; RecordType::ALL.len()];
        for (record_type, tally) in RecordType::ALL.into_iter().zip(&mut held) {
            records.extend(self.records_in(record_type.directory())?);
            let filed = self.records_in(&archive_of(record_type.directory()))?;
            *tally = filed.len();
            records.extend(filed);
        }
        let archived = records
            .iter()
            .filter(|record| is_archived(record.path()))
            .filter_map(|record| resolvable_id(record.path()))
            .map(str::to_owned)
            .collect();
        Ok(Corpus {
            records,
            archived,
            archive: Counts::per_type(held),
        })
    }

    /// The live records, with the archive present by name alone: a query
    /// about work in motion costs what the live notebook costs, however far
    /// history has grown behind it.
    fn live_corpus(&self) -> Result<Corpus, NotebookError> {
        let mut records = Vec::new();
        for record_type in RecordType::ALL {
            records.extend(self.records_in(record_type.directory())?);
        }
        self.corpus(records)
    }

    fn corpus(&self, records: Vec<Record>) -> Result<Corpus, NotebookError> {
        let mut archived = BTreeSet::new();
        let mut held = [0; RecordType::ALL.len()];
        for (record_type, tally) in RecordType::ALL.into_iter().zip(&mut held) {
            for path in self.storage.list(&archive_of(record_type.directory()))? {
                if !is_record_file(&path) {
                    continue;
                }
                *tally += 1;
                archived.extend(resolvable_id(&path).map(str::to_owned));
            }
        }
        Ok(Corpus {
            records,
            archived,
            archive: Counts::per_type(held),
        })
    }

    /// One directory's records, invalid ones included: an invalid record is
    /// a visible first-class state, never a silent drop.
    fn records_in(&self, dir: &str) -> Result<Vec<Record>, NotebookError> {
        let mut records = Vec::new();
        for path in self.storage.list(dir)? {
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
        Ok(records)
    }
}

/// The records one query reads, beside the ids the archive holds. The two
/// are separate because they cost differently: a record is a file opened,
/// an archived id is a name in a listing.
struct Corpus {
    records: Vec<Record>,
    archived: BTreeSet<String>,
    archive: Counts,
}

impl Corpus {
    fn resolver(&self) -> Resolver<'_> {
        Resolver::of(&self.records, &self.archived)
    }
}

/// A live record cleared for mutation, so it carries no error finding.
struct LoadedLive {
    path: String,
    record: Record,
}

impl LoadedLive {
    /// A record with no state carries an error finding, and one of those
    /// is what [`Notebook::resolve_live`] refuses, so a value of this type
    /// always has one.
    fn state_word(&self) -> &str {
        self.record.state().expect("a clean record carries a state")
    }
}

/// A live record read over its error findings, for the verbs that erase
/// them. It is the whole repair protocol: the findings it came in with
/// travel with it, and [`Notebook::commit_repair`] is the only way to the
/// bytes — so a verb cannot write an invalid record by forgetting a step.
struct Repairing {
    loaded: LoadedLive,
    carried: Vec<Finding>,
}

impl Repairing {
    fn record(&self) -> &Record {
        &self.loaded.record
    }
}

/// The type `id` names, refused when the command does not accept it.
fn commanded_type(id: &str, expected: &[RecordType]) -> Result<RecordType, NotebookError> {
    let record_type = write::parsed_type(id)?;
    if expected.contains(&record_type) {
        return Ok(record_type);
    }
    Err(NotebookError::WrongType {
        id: id.to_owned(),
        expected: error::type_list(expected),
    })
}

/// A report an archive move carries, judged before the first byte moves.
struct Report {
    path: String,
    record: Record,
    destination: String,
}

struct Victim {
    path: String,
    record: Record,
    dead_state: &'static str,
}
