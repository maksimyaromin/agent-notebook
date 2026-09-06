//! The Notebook: the records under one root, read and mutated through
//! Storage.
//!
//! Write-time invariants live here, and they bind every author equally: a
//! declared supersession writes the back-pointer and flips the victim, a
//! Question closes into what resolved it or with a stated reason, a close
//! carries its proof. A verb that moves a record already in the notebook is
//! idempotent — a replayed call answers `already: true` and leaves every
//! byte of every file unchanged. Creating and expunging are not replays of
//! anything: a second `add` of the same title mints a second record, and a
//! second `delete` names an id the notebook no longer holds.
//!
//! The mutation gate holds a record's own error findings and its dangling
//! references against it; findings that need a second record — a broken
//! supersession pair, a multi-file dependency cycle — are `check`'s alone
//! and freeze nothing. The one way past the gate is a repair: a verb that
//! erases a line reads its record over the error findings and is judged on
//! the bytes it produces, so a corrupted line cannot freeze the verb that
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
mod gate;
mod query;
mod write;

pub use error::NotebookError;

use crate::config::{CONFIG_PATH, Config};
use crate::debt::{self, DebtSources};
use crate::encode;
use crate::finding::Finding;
use crate::grammar::{self, RecordFile};
use crate::graph;
use crate::mention;
use crate::record::{
    REF_KEYS, Record, RecordType, TaskAction, TaskState, Transition, dangling_finding,
    not_utf8_finding,
};
use crate::reply::{
    Archived, CitedProof, Closed, Commented, Counts, Created, Deleted, Edged, Edited, Focus, Graph,
    GraphSlice, Held, ListedRecord, Overview, ReadyTask, Restored, Transitioned, TypeSection, View,
};
use crate::request::{Draft, Edit, Link, Proof};
use crate::resolve::{
    Resolver, archive_of, archived_among, canonical_paths, is_archived, is_record_file, path_stem,
    record_path, resolvable_id,
};
use crate::status::{self, Budget, Status, StatusInputs};
use crate::storage::{Storage, StorageError};
use gate::LoadedLive;
use std::collections::BTreeSet;

/// A report landed in the notebook, and what its body cited.
struct IngestedReport {
    id: String,
    dangling_mentions: Vec<String>,
}

pub struct Notebook<'a> {
    storage: &'a mut dyn Storage,
    /// The user's notebook standing behind this one, read and never
    /// written — the shared reference keeps that a fact of the type. An id
    /// it holds is no dangling citation and no dangling `link` target, and
    /// its standing rules are what a project rule shadows. `None` names no
    /// such root; a root that cannot be read counts as one, since a second
    /// notebook is consulted for a hint and never fails a write or the gate.
    user: Option<&'a dyn Storage>,
}

impl<'a> Notebook<'a> {
    pub fn new(storage: &'a mut dyn Storage) -> Self {
        Notebook {
            storage,
            user: None,
        }
    }

    /// The same notebook with the user's own standing behind it.
    #[must_use]
    pub fn with_user(self, user: Option<&'a dyn Storage>) -> Self {
        Notebook { user, ..self }
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
        let behind = user_scope(self.user);
        let sources = DebtSources {
            records,
            resolvable: &resolvable,
            today_day,
            lost_proofs: &lost,
            user_records: &behind.records,
            user_archived: &behind.archived,
        };
        let queue = query::ready_rows(records, &resolvable);
        let inputs = StatusInputs {
            counts: query::live_counts(records),
            active: query::active_tasks(&live_valid),
            review: query::review_tasks(&live_valid),
            held: query::held_tasks(&live_valid),
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
                    .filter(|record| query::lives_in(record, record_type))
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

    /// The notebook as a graph: every record the slice reaches, each with the
    /// edges it takes part in, and the record itself when asked for.
    ///
    /// Every slice reads the whole notebook. Narrowing changes what is
    /// shown, never what is read: a record's state, an epic's count, and
    /// the queue are all settled against the whole notebook first, so a hub
    /// still counts the closed children a slice leaves out.
    ///
    /// One id is one node. Where an interrupted archive move left the same
    /// id in both homes, the live file answers for it, as it does on every
    /// other edge the notebook walks.
    ///
    /// The nodes arrive in one order for one notebook: live records before
    /// archived ones, by file name within each. Whatever derives an
    /// arrangement from that order gets the same one twice from an
    /// unchanged notebook.
    ///
    /// # Errors
    /// [`NotebookError::UnknownId`] when the slice names no record,
    /// [`NotebookError::InvalidArgument`] on a malformed id or on a focus
    /// the slice itself leaves out, or a storage failure.
    pub fn graph(&self, slice: &GraphSlice) -> Result<Graph, NotebookError> {
        if let Some(hub) = &slice.hub {
            write::parsed_type(hub)?;
            if self.holder_path(hub)?.is_none() {
                return Err(NotebookError::UnknownId { id: hub.clone() });
            }
        }
        let live = self.live_corpus()?;
        let filed = self.filed_records()?;
        let resolvable = live.resolver();
        let queue = query::ready_rows(&live.records, &resolvable);
        let epics = query::epic_rows(&live.records, &filed, &resolvable, &queue);
        let scope = slice.hub.as_deref().map(|hub| {
            query::MembershipIndex::of(&live.records, &filed)
                .scope_of(hub)
                .into_iter()
                .map(str::to_owned)
                .collect::<BTreeSet<String>>()
        });

        let mut claimed = BTreeSet::new();
        let drawable: Vec<&Record> = live
            .records
            .iter()
            .chain(&filed)
            .filter(|record| claimed.insert(path_stem(record.path())))
            .filter(|record| slice.archive || !is_archived(record.path()))
            .collect();
        let near = match &slice.focus {
            Some(focus) => Some(query::neighbourhood(
                &drawable,
                self.focusable(focus, slice.archive)?,
                focus.depth,
            )),
            None => None,
        };

        // The types narrow what is drawn, never what the neighbourhood was
        // walked over: a reader asking which Decisions stand around a Task
        // is asking about that Task's surroundings, and a walk that could
        // not step through a Task would answer that nothing does.
        let asked_for = |record: &Record| {
            slice.types.is_empty()
                || record
                    .record_type()
                    .is_some_and(|record_type| slice.types.contains(&record_type))
        };
        let shown = |record: &Record| {
            let id = path_stem(record.path());
            asked_for(record)
                && scope.as_ref().is_none_or(|scope| scope.contains(id))
                && (!slice.ready_only || queue.iter().any(|row| row.id == id))
                && near.as_ref().is_none_or(|near| near.contains(id))
        };
        let nodes = drawable
            .into_iter()
            .filter(|record| shown(record))
            .map(|record| query::graph_node(record, &resolvable, &epics, &queue))
            .collect();
        Ok(Graph {
            slice: slice.clone(),
            nodes,
        })
    }

    /// The id a focus walk starts from, once it is a record this slice
    /// keeps. An id nothing is filed under, and one filed away while the
    /// archive is left out, both answer with an empty graph — which reads
    /// exactly like a notebook holding nothing — so each is replaced by the
    /// correction that fills it. A focus narrowed away by another flag is
    /// still an empty answer; the others say what they leave out in the
    /// slice they carry back.
    fn focusable<'f>(&self, focus: &'f Focus, archive: bool) -> Result<&'f str, NotebookError> {
        let refused = |reason: String| Err(NotebookError::InvalidArgument { reason });
        write::parsed_type(&focus.id)?;
        let Some(path) = self.holder_path(&focus.id)? else {
            return Err(NotebookError::UnknownId {
                id: focus.id.clone(),
            });
        };
        if is_archived(&path) && !archive {
            return refused(format!(
                "graph: `{}` is archived; add --archive to focus on it",
                focus.id
            ));
        }
        Ok(&focus.id)
    }

    /// Every archived record, read whole: the tiles a map draws for
    /// finished work, and the lineage a live record's Origin reaches back
    /// through.
    fn filed_records(&self) -> Result<Vec<Record>, NotebookError> {
        let mut filed = Vec::new();
        for record_type in RecordType::ALL {
            filed.extend(read_records_in(
                self.storage,
                &archive_of(record_type.directory()),
            )?);
        }
        Ok(filed)
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
        self.create_minting(draft, today, || write::title_id(draft))
    }

    /// [`Notebook::create`] with the id minted on the base `minted` names
    /// when the draft carries none: a Note born as a Task's report is named
    /// after the Task, every other record after its title.
    fn create_minting(
        &mut self,
        draft: &Draft,
        today: &str,
        minted: impl FnOnce() -> Result<String, NotebookError>,
    ) -> Result<Created, NotebookError> {
        write::guard_today(today)?;
        write::validate_draft(draft)?;
        if let Some(origin) = &draft.from {
            self.guard_ref_exists("from", origin)?;
        }
        for link in &draft.links {
            self.guard_link_target(link)?;
        }
        let victim = self.guard_supersession(draft)?;

        let corpus = self.live_corpus()?;
        let records = &corpus.records;
        let claims = write::id_claims(records, &corpus.archived);
        let id = write::resolve_draft_id(draft, &claims, minted)?;
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
        self.closed(id, transition, Vec::new(), None)
    }

    /// `open | active | review → closed` without work or without a record:
    /// the Task or Question ends stating why. The reason lands in the
    /// envelope as `reason` and the close date is stamped, but no proof link
    /// is written — a proof would vouch for work that did not happen. The
    /// state is the same `closed`, so dependents unblock and an epic counts
    /// it like any close; the envelope carries the distinction.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on an empty or multi-line reason,
    /// plus the refusals of [`Notebook::close`].
    pub fn close_with_reason(
        &mut self,
        id: &str,
        reason: &str,
        today: &str,
    ) -> Result<Closed, NotebookError> {
        let reason = write::guarded_reason("close", reason)?;
        let transition = match write::parsed_type(id)? {
            RecordType::Question => self.close_question(id, today, |file| {
                file.set_field("reason", reason);
            })?,
            _ => self.task_transition(id, TaskAction::CloseWithReason, today, |file| {
                file.set_field("closed", today);
                file.set_field("reason", reason);
            })?,
        };
        // A replay wrote nothing, so its reason has no citations to nudge on.
        let dangling_mentions = if transition.already {
            Vec::new()
        } else {
            self.dangling_mentions(reason)?
        };
        self.closed(id, transition, dangling_mentions, None)
    }

    /// The reply every way of closing shares: the move, plus the
    /// consequences a close must not bury — the still-open Questions born
    /// from the Task and the Tasks it was the last live blocker of.
    fn closed(
        &self,
        id: &str,
        transition: Transitioned,
        dangling_mentions: Vec<String>,
        resolved_by: Option<String>,
    ) -> Result<Closed, NotebookError> {
        let corpus = self.live_corpus()?;
        let records = &corpus.records;
        let resolvable = corpus.resolver();
        Ok(Closed {
            transition,
            open_questions: query::open_questions_from(records, &resolvable, id),
            unblocked: query::unblocked_by_close(records, &resolvable, id),
            report_note: None,
            resolved_by,
            dangling_mentions,
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
                reason: "note: the report is empty; a close carries a proof or waives one"
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
        let created = self.create_minting(&draft, today, || Ok(write::report_note_id(origin)))?;
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
        Ok(read_records_in(self.storage, RecordType::Note.directory())?
            .into_iter()
            .find(|note| note.origin() == Some(origin) && note.file().body() == wanted)
            .map(|note| path_stem(note.path()).to_owned()))
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
        let author = write::guarded_author(author)?;

        let entry = write::log_entry(today, author, text);
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

    /// Close a Question into the Decision or Task that resolved it:
    /// `open → closed`, writing `resolved-by` in the same move so the record
    /// can never lose the thread of what settled it.
    ///
    /// # Errors
    /// [`NotebookError::WrongType`] when `resolved_by` is not a decision or
    /// a task, [`NotebookError::DanglingRef`] when it does not exist,
    /// [`NotebookError::InvalidTransition`] from a settled state, plus the
    /// resolution errors of [`Notebook::close`].
    pub fn resolve_question(
        &mut self,
        id: &str,
        resolved_by: &str,
        today: &str,
    ) -> Result<Closed, NotebookError> {
        let target_type = write::parsed_type(resolved_by)?;
        if !matches!(target_type, RecordType::Decision | RecordType::Task) {
            return Err(NotebookError::WrongType {
                id: resolved_by.to_owned(),
                expected: "a decision or a task".to_owned(),
            });
        }
        self.guard_ref_exists("resolved-by", resolved_by)?;
        let transition = self.close_question(id, today, |file| {
            file.set_field("resolved-by", resolved_by);
        })?;
        // A replay wrote nothing, so the reply names the resolver the record
        // holds, not the one this call carried.
        let resolved_by = if transition.already {
            self.load_live(id, &[RecordType::Question])?
                .record
                .resolved_by()
                .map(str::to_owned)
        } else {
            Some(resolved_by.to_owned())
        };
        self.closed(id, transition, Vec::new(), resolved_by)
    }

    /// The shared shape of a Question's close: `open → closed` with the
    /// close date stamped and the verb's own outcome field written; a
    /// Question already closed answers with a replay and changes no byte.
    fn close_question(
        &mut self,
        id: &str,
        today: &str,
        stamp_outcome: impl FnOnce(&mut RecordFile),
    ) -> Result<Transitioned, NotebookError> {
        write::guard_today(today)?;
        let loaded = self.load_live(id, &[RecordType::Question])?;
        let state = loaded.state_word();
        if state == "closed" {
            return Ok(Transitioned::replayed(id, "closed"));
        }
        if state != "open" {
            return Err(error::settled_question(id, state));
        }
        let mut file = loaded.record.into_file();
        file.set_field("state", "closed");
        file.set_field("closed", today);
        stamp_outcome(&mut file);
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Transitioned {
            id: id.to_owned(),
            from: "open",
            to: "closed",
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
    /// the commands that settle it; [`NotebookError::InvalidRecord`] or
    /// [`NotebookError::DuplicateId`] on a report that cannot be filed, and
    /// [`NotebookError::DuplicateId`] on a destination held by anything but
    /// this record's own copy — both answered before anything moves; plus
    /// the resolution errors of [`Notebook::close`].
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
        // An archive move rewrites nothing, so a resume finds the record
        // under its own name, answering for its own id.
        self.guard_destination_free(&moved.to, |standing| standing.id() == Some(id))?;
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

    /// Move an archived record back into the working set: the move
    /// `archive` makes, made back — same filename, same bytes, the record
    /// alone. The reports the archive move carried are retired history and
    /// stay history.
    ///
    /// No verb that corrects a record resolves an archived id — `delete`
    /// reaches the archive only to delete — so this move is how a finding
    /// on an archived record becomes repairable at all. The bytes travel
    /// unjudged past one bar, readability: a broken record must be able to
    /// come back to where the repairing verbs are, but this verb promises
    /// residence and cannot vouch it over bytes no parse can read — on
    /// either side of the move, and on the replay, whose `already`
    /// otherwise stands on the live file's existence alone, findings and
    /// all.
    ///
    /// A live file under this id keeps its bytes whatever happens: the
    /// live directory is the only editable home, so what stands there is
    /// the record's current truth. When the archive also holds the id —
    /// the leftover of a move interrupted in either direction — this call
    /// finishes the move by removing that leftover, provided both files
    /// answer for this record — the resume test `leftover_is_own` carries
    /// the two tiers and their reasons; anything else holds the
    /// destination, and the move refuses rather than guess.
    ///
    /// # Errors
    /// [`NotebookError::UnknownId`] when neither home holds the id,
    /// [`NotebookError::DuplicateId`] on a live destination taken by a
    /// file this move cannot call its own, [`NotebookError::InvalidRecord`]
    /// on bytes that cannot cross the seam, or a storage failure.
    pub fn restore(&mut self, id: &str) -> Result<Restored, NotebookError> {
        let record_type = write::parsed_type(id)?;
        let moved = Restored {
            id: id.to_owned(),
            from: record_path(id, record_type, true),
            to: record_path(id, record_type, false),
            already: false,
        };
        match (self.held_at(&moved.from)?, self.held_at(&moved.to)?) {
            (Holding::Unreadable, _) => Err(unreadable_record(&moved.from)),
            (Holding::Absent, Holding::Absent) => {
                Err(NotebookError::UnknownId { id: id.to_owned() })
            }
            (Holding::Absent, Holding::Unreadable) => Err(unreadable_record(&moved.to)),
            (Holding::Absent, Holding::Bytes(_)) => Ok(Restored {
                already: true,
                ..moved
            }),
            (Holding::Bytes(source), Holding::Absent) => {
                self.storage.write(&moved.to, &source)?;
                self.storage.remove(&moved.from)?;
                Ok(moved)
            }
            (Holding::Bytes(source), Holding::Bytes(standing))
                if leftover_is_own(&moved, &source, &standing) =>
            {
                self.storage.remove(&moved.from)?;
                Ok(moved)
            }
            (Holding::Bytes(_), _) => Err(NotebookError::DuplicateId {
                id: id.to_owned(),
                holder: moved.to,
            }),
        }
    }

    /// What one home holds for a move that judges names and bytes, never
    /// states: the file's text, nothing, or bytes no parse can read.
    fn held_at(&self, path: &str) -> Result<Holding, NotebookError> {
        match self.storage.read(path) {
            Ok(text) => Ok(Holding::Bytes(text)),
            Err(StorageError::NotFound { .. }) => Ok(Holding::Absent),
            Err(StorageError::NotUtf8 { .. }) => Ok(Holding::Unreadable),
            Err(error) => Err(error.into()),
        }
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
            // A record that links itself is a hand edit, and carrying it as
            // its own report would file it twice: once here and once by the
            // move that called this, whose second write would then fail on
            // a source it had already removed.
            if id == origin {
                continue;
            }
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
            // A report is retired as it is filed, so a resume finds this
            // origin's own Note already wearing the state the move gave it.
            self.guard_destination_free(&destination, |standing| {
                standing.origin() == Some(origin) && standing.state() == Some("retired")
            })?;
            reports.push(Report {
                path,
                record,
                destination,
            });
        }
        Ok(reports)
    }

    /// Refuse a place in the archive another record holds, and admit one
    /// holding this very move's own copy, left where a run crashed between
    /// its write and its remove. `is_this_moves_own` names what makes the
    /// standing record that copy.
    ///
    /// Identity is the whole admission test, and it settles only whether
    /// the move may go on — never whether it must write. The file being
    /// moved from is the authoritative one: it can carry a correction made
    /// since the interrupted run, and a move that trusted what it found
    /// would delete that correction unread. Bytes cannot serve as the test
    /// either, in both directions: filing a report restamps it, so a sound
    /// resume never matches, and a record corrected since its interrupted
    /// move stops matching a twin that is its own.
    fn guard_destination_free(
        &self,
        destination: &str,
        is_this_moves_own: impl Fn(&Record) -> bool,
    ) -> Result<(), NotebookError> {
        let standing = match self.storage.read(destination) {
            Ok(text) => Record::parse(destination, &text),
            Err(StorageError::NotFound { .. }) => return Ok(()),
            // Bytes no parse can read stand for a record that is not this
            // one, which is the refusal below.
            Err(StorageError::NotUtf8 { .. }) => Record::unreadable(destination),
            Err(error) => return Err(error.into()),
        };
        if !standing.has_errors() && is_this_moves_own(&standing) {
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
    /// a notebook pointing at nothing. Alone among the verbs it does not
    /// replay — with the record gone, nothing tells a delete already
    /// done from an id that never existed.
    ///
    /// # Errors
    /// [`NotebookError::StillReferenced`] naming every blocker,
    /// [`NotebookError::UnknownId`], [`NotebookError::InvalidArgument`] on
    /// a malformed id, or a storage failure.
    pub fn delete(&mut self, id: &str) -> Result<Deleted, NotebookError> {
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
        Ok(Deleted {
            id: id.to_owned(),
            paths,
        })
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
        for link in &edit.add_links {
            self.guard_link_target(link)?;
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
                    errors.push(dangling_finding(key, target, line));
                }
            }
        }
        Ok(errors)
    }

    /// The write-time half of the quotation rule: the bare ids `text` cites
    /// that neither this notebook nor the user's holds, live or archived. A
    /// nudge for the reply, never a gate — a forward reference is legal and
    /// the text lands as given.
    fn dangling_mentions(&self, text: &str) -> Result<Vec<String>, NotebookError> {
        let mut dangling = Vec::new();
        for target in mention::mentions(text) {
            if self.holder_path(target)?.is_none() && !self.user_holds(target) {
                dangling.push(target.to_owned());
            }
        }
        Ok(dangling)
    }

    /// Whether the user's notebook holds `id` at a canonical path. A root
    /// that cannot answer holds nothing: the hint is dropped, the write goes
    /// on.
    fn user_holds(&self, id: &str) -> bool {
        let Some(user) = self.user else {
            return false;
        };
        canonical_paths(id).any(|path| user.exists(&path).unwrap_or(false))
    }

    /// A link whose target is shaped like an id names a record, and a
    /// record it names must exist: here, or in the user's notebook, which a
    /// link reaches as `check` reads it. Any other target points outside
    /// the notebook and is taken as given.
    fn guard_link_target(&self, link: &Link) -> Result<(), NotebookError> {
        let target = link.target.trim();
        if grammar::id_error(target).is_some()
            || self.holder_path(target)?.is_some()
            || self.user_holds(target)
        {
            return Ok(());
        }
        Err(NotebookError::DanglingRef {
            field: "link",
            target: target.to_owned(),
        })
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
    /// only delete, which must leave nothing behind, needs them all.
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
        let mut held = RecordType::ALL.map(|record_type| (record_type, 0));
        for (record_type, tally) in &mut held {
            records.extend(read_records_in(self.storage, record_type.directory())?);
            let filed = read_records_in(self.storage, &archive_of(record_type.directory()))?;
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
        read_live_corpus(self.storage)
    }
}

/// [`Notebook::live_corpus`] over any root, so a second notebook is read by
/// the same rules as the first.
fn read_live_corpus(storage: &dyn Storage) -> Result<Corpus, NotebookError> {
    let mut records = Vec::new();
    for record_type in RecordType::ALL {
        records.extend(read_records_in(storage, record_type.directory())?);
    }
    let mut archived = BTreeSet::new();
    let mut held = RecordType::ALL.map(|record_type| (record_type, 0));
    for (record_type, tally) in &mut held {
        for path in storage.list(&archive_of(record_type.directory()))? {
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

/// The user's notebook standing behind a project's, read by the same rules
/// as any other: its live records, and the ids its archive holds.
///
/// A second root is read for a hint on somebody else's dashboard, a nudge
/// in somebody else's reply, or the reach of a `link` in somebody else's
/// gate, so a root that cannot be read leaves the hint out instead of
/// taking those down. What is wrong with that notebook is what a `check`
/// against it reports.
pub(super) fn user_scope(user: Option<&dyn Storage>) -> Corpus {
    user.and_then(|storage| read_live_corpus(storage).ok())
        .unwrap_or_else(Corpus::empty)
}

/// One directory's records, invalid ones included: an invalid record is a
/// visible first-class state, never a silent drop.
fn read_records_in(storage: &dyn Storage, dir: &str) -> Result<Vec<Record>, NotebookError> {
    let mut records = Vec::new();
    for path in storage.list(dir)? {
        if !is_record_file(&path) {
            continue;
        }
        match storage.read(&path) {
            Ok(text) => records.push(Record::parse(&path, &text)),
            // The adapter's duty ends at naming the encoding; the
            // file stays a visible invalid record, not an abort.
            Err(StorageError::NotUtf8 { .. }) => records.push(Record::unreadable(&path)),
            // A listing is a snapshot. Between it and this read another
            // process may have filed or deleted the record, and a
            // reader that answered `not found` for the whole notebook
            // would be reporting someone else's completed work as its
            // own failure.
            Err(StorageError::NotFound { .. }) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(records)
}

/// The records one query reads, beside the ids the archive holds. The two
/// are separate because they cost differently: a record is a file opened,
/// an archived id is a name in a listing.
pub(super) struct Corpus {
    records: Vec<Record>,
    archived: BTreeSet<String>,
    archive: Counts,
}

impl Corpus {
    /// The notebook a caller was handed no root for.
    fn empty() -> Corpus {
        Corpus {
            records: Vec::new(),
            archived: BTreeSet::new(),
            archive: Counts::default(),
        }
    }

    fn resolver(&self) -> Resolver<'_> {
        Resolver::of(&self.records, &self.archived)
    }
}

/// A report an archive move carries, judged before the first byte moves.
/// What one home holds, as [`Notebook::held_at`] reads it.
enum Holding {
    Absent,
    Bytes(String),
    Unreadable,
}

/// The refusal for bytes that cannot cross the seam, named as the record
/// they occupy.
fn unreadable_record(path: &str) -> NotebookError {
    NotebookError::InvalidRecord {
        path: path.to_owned(),
        findings: vec![not_utf8_finding()],
    }
}

/// The resume test for a restore that finds its id in both homes: the two
/// files are the same record when they are byte-identical — an untouched
/// interrupted copy, however broken — or when each answers for the id on
/// its own. The standing file answers by parsing clean, because a
/// canonical live path admits no clean record but the id's own and `add`
/// refuses an id the archive claims; the leftover answers by declaring the
/// id in its bytes, because bytes under this filename that declare another
/// record are somebody's only copy, and removing them unread would destroy
/// it.
fn leftover_is_own(moved: &Restored, source: &str, standing: &str) -> bool {
    source == standing
        || (!Record::parse(&moved.to, standing).has_errors()
            && Record::parse(&moved.from, source).id() == Some(&moved.id))
}

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
