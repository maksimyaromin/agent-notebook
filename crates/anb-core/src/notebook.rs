//! The Notebook: the records under one root, read and mutated through
//! Storage.
//!
//! Write-time invariants live here, and they bind every author equally: a
//! declared supersession writes the back-pointer and flips the victim, a
//! Question closes into what resolved it or with a stated reason, and a
//! completed Task records its outcome. A verb that moves a record already
//! in the notebook is idempotent: a replayed call answers `already: true` and leaves every
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
mod memory;
mod query;
mod transfer;
mod write;

pub use error::NotebookError;
pub use memory::recall;

use crate::config::{CONFIG_PATH, Config};
use crate::debt::{self, DebtSignal, DebtSources};
use crate::encode;
use crate::finding::Finding;
use crate::grammar::{self, RecordFile};
use crate::graph;
use crate::mention;
use crate::record::{
    REF_KEYS, Record, RecordType, TaskAction, TaskState, Transition, dangling_finding,
    linked_record, not_utf8_finding,
};
use crate::reply::{
    Archived, CitedProof, Closed, Commented, Created, Deleted, Edged, Edited, Graph, Held,
    ListedRecord, ReadyTask, Restored, Transitioned, View,
};
use crate::request::{Draft, Edit, Filter, Focus, GraphSlice, Link};
use crate::resolve::{
    Resolver, archive_of, archived_among, canonical_paths, is_archived, is_record_file, path_stem,
    record_path, resolvable_id,
};
use crate::status::{self, Budget, Status, StatusInputs};
use crate::storage::{Storage, StorageError};
use gate::LoadedLive;
use std::collections::BTreeSet;

pub struct Notebook<'a> {
    storage: &'a mut dyn Storage,
    /// The accountable identity this notebook is worked under: what a new
    /// record's `by` and a log entry's signature carry unless a request
    /// names another, who `start` records as having taken the Task, and
    /// whose work the dashboard opens on. `None` is a host that knows
    /// nobody, and the notebook then signs nothing and takes nothing.
    identity: Option<&'a str>,
}

impl<'a> Notebook<'a> {
    pub fn new(storage: &'a mut dyn Storage) -> Self {
        Notebook {
            storage,
            identity: None,
        }
    }

    /// The same notebook worked under `identity`.
    #[must_use]
    pub fn with_identity(self, identity: Option<&'a str>) -> Self {
        Notebook { identity, ..self }
    }

    /// Read maintained knowledge with the same selection rules as [`recall`].
    ///
    /// # Errors
    /// A storage failure, or an unknown focus record.
    pub fn recall(
        &self,
        text: Option<&str>,
        focus: Option<&str>,
    ) -> Result<crate::Knowledge, NotebookError> {
        recall(self.storage, text, focus)
    }

    /// The identity as a record may carry it: one non-empty line. A host
    /// hands it in from an environment it does not judge, so the guard
    /// stands here, once, where every write reads it.
    fn guarded_identity(&self) -> Result<Option<&'a str>, NotebookError> {
        write::guarded_name("by", self.identity)
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
    /// urgent first, narrowed by `filter`. The queue is drawn from live
    /// open Tasks, so a filter naming another type or the archive admits
    /// nothing more. Invalid records are excluded, as from every derived
    /// query; `check` names them.
    ///
    /// # Errors
    /// [`NotebookError::UnknownId`] when the hub names no record,
    /// [`NotebookError::InvalidArgument`] on a malformed id, a kind no type
    /// allows, a malformed tag or an empty text, or a storage failure.
    pub fn ready(&self, filter: &Filter) -> Result<Vec<ReadyTask>, NotebookError> {
        let (corpus, admission) = self.narrowed(filter)?;
        Ok(query::ready_rows(
            &corpus.records,
            &corpus.resolver(),
            |record| admission.admits(record),
        ))
    }

    /// The listing, in type-major file order, narrowed by `filter`; a
    /// hub's scope includes the hub itself, so a reader opening an epic
    /// sees it beside its work, and the archive joins when asked for. An
    /// invalid record is a row of state `invalid` — excluded from mutation
    /// and derived queries, never from sight; `check` names its findings.
    ///
    /// # Errors
    /// See [`Notebook::ready`].
    pub fn list(&self, filter: &Filter) -> Result<Vec<ListedRecord>, NotebookError> {
        let (corpus, admission) = self.narrowed(filter)?;
        let resolvable = corpus.resolver();
        Ok(corpus
            .records
            .iter()
            .filter(|record| admission.admits(record))
            .map(|record| query::listed_row(record, &resolvable))
            .collect())
    }

    /// The records a filter reads and what it admits of them. A filter
    /// narrows what is *shown*, never what is *read*: a row is blocked,
    /// excluded and resolved against every record, so narrowing a query
    /// cannot change any row's verdict — only which rows reach the reader.
    fn narrowed(&self, filter: &Filter) -> Result<(Corpus, query::Admission), NotebookError> {
        guard_filter(filter)?;
        let corpus = if filter.archive {
            self.whole_corpus()?
        } else {
            self.live_corpus()?
        };
        let Some(hub) = &filter.hub else {
            return Ok((corpus, query::Admission::of(filter, None)));
        };
        self.guard_hub(hub)?;
        // A whole corpus already holds the archive; a live one borrows the
        // filed kin an epic's lineage runs through.
        let kin = if filter.archive {
            Vec::new()
        } else {
            self.archived_kin(&corpus)?
        };
        let scope = query::MembershipIndex::of(&corpus.records, &kin)
            .scope_of(hub)
            .into_iter()
            .map(str::to_owned)
            .collect();
        Ok((corpus, query::Admission::of(filter, Some(scope))))
    }

    /// A hub a filter names must be a record the notebook holds: an empty
    /// answer reads exactly like an epic with nothing in it.
    fn guard_hub(&self, hub: &str) -> Result<(), NotebookError> {
        write::parsed_type(hub)?;
        if self.holder_path(hub)?.is_none() {
            return Err(NotebookError::UnknownId { id: hub.to_owned() });
        }
        Ok(())
    }

    /// The archived records the live ones still name as a blocker, an
    /// Origin or a link target, and the ones those name in turn.
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

    /// Read the archived records `wanted` names, and those they name in
    /// turn, beside the `kin` already read.
    ///
    /// Follow declared references into archived history without opening
    /// unrelated archived files. Live origin descendants remain connected
    /// through an archived ancestor. Archived descendants unnamed by any
    /// live record join only an explicit archive query.
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
            let Some(record) = read_archived_record(self.storage, &id)? else {
                continue;
            };
            wanted.extend(archived_among(query::kin_of(&record), archived));
            kin.push(record);
        }
        Ok(kin)
    }

    /// The session-start dashboard under `budget`, gated: one quiet line
    /// when the notebook carries no signal, the full budgeted composite
    /// otherwise. The caller resolves `budget` — a CLI flag outranks the
    /// config key, which defaults to 1500 — and `by`, the one identity the
    /// work is narrowed to, or the whole team's. Narrowed, it holds what
    /// [`Record::concerns`] that identity: what they hold, what they
    /// wrote, and what waits on them.
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
        by: Option<&str>,
        settle: impl FnOnce(&[CitedProof]) -> Vec<CitedProof>,
    ) -> Result<Status, NotebookError> {
        let today_day = write::guarded_day(today)?;
        let corpus = self.live_corpus()?;
        let records = &corpus.records;
        let resolvable = corpus.resolver();
        let debt = self.decay(&corpus, today_day, settle)?;
        let named = |record: &Record| by.is_none_or(|by| record.concerns(by));
        let scoped: Vec<&Record> = records
            .iter()
            .filter(|record| !debt::is_excluded(record, &resolvable) && named(record))
            .collect();
        let identity = self.identity;
        let inputs = StatusInputs {
            by: by.map(str::to_owned),
            counts: query::live_counts(records),
            active: query::active_tasks(&scoped, identity),
            review: query::review_tasks(&scoped, identity),
            held: query::held_tasks(&scoped, identity),
            ready: query::ready_rows(records, &resolvable, named),
            untaken: query::ready_rows(records, &resolvable, query::is_untaken).len(),
            questions: query::open_questions(&scoped, identity),
            debt: debt.len(),
        };
        Ok(status::assemble(inputs, budget))
    }

    /// The Debt of the whole notebook, in the clock table's order: the
    /// read the dashboard's count line points at. `settle` is what
    /// [`Notebook::status`] takes.
    ///
    /// # Errors
    /// See [`Notebook::status`].
    pub fn debt(
        &self,
        today: &str,
        settle: impl FnOnce(&[CitedProof]) -> Vec<CitedProof>,
    ) -> Result<Vec<DebtSignal>, NotebookError> {
        let today_day = write::guarded_day(today)?;
        self.decay(&self.live_corpus()?, today_day, settle)
    }

    /// Every sign of decay over this notebook's live records.
    fn decay(
        &self,
        corpus: &Corpus,
        today_day: i64,
        settle: impl FnOnce(&[CitedProof]) -> Vec<CitedProof>,
    ) -> Result<Vec<DebtSignal>, NotebookError> {
        let thresholds = self.config()?.debt_thresholds();
        let records = &corpus.records;
        let resolvable = corpus.resolver();
        let lost = settle(&query::cited_proofs(records));
        let sources = DebtSources {
            records,
            resolvable: &resolvable,
            today_day,
            lost_proofs: &lost,
        };
        Ok(debt::signals(&sources, &thresholds))
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
    /// [`NotebookError::InvalidArgument`] on a malformed id, a filter
    /// [`Notebook::ready`] would refuse, or a focus the slice itself
    /// leaves out, or a storage failure.
    pub fn graph(&self, slice: &GraphSlice) -> Result<Graph, NotebookError> {
        let filter = &slice.filter;
        guard_filter(filter)?;
        if let Some(hub) = &filter.hub {
            self.guard_hub(hub)?;
        }
        let live = self.live_corpus()?;
        let filed = self.filed_records()?;
        let resolvable = live.resolver();
        let queue = query::ready_rows(&live.records, &resolvable, |_| true);
        let epics = query::epic_rows(&live.records, &filed, &resolvable, &queue);
        let scope = filter.hub.as_deref().map(|hub| {
            query::MembershipIndex::of(&live.records, &filed)
                .scope_of(hub)
                .into_iter()
                .map(str::to_owned)
                .collect::<BTreeSet<String>>()
        });
        let admission = query::Admission::of(filter, scope);

        let mut claimed = BTreeSet::new();
        let drawable: Vec<&Record> = live
            .records
            .iter()
            .chain(&filed)
            .filter(|record| claimed.insert(path_stem(record.path())))
            .filter(|record| filter.archive || !is_archived(record.path()))
            .collect();
        let near = match &slice.focus {
            Some(focus) => Some(query::neighbourhood(
                &drawable,
                self.focusable(focus, filter.archive)?,
                focus.depth,
            )),
            None => None,
        };

        // The filter narrows what is drawn, never what the neighbourhood
        // was walked over: a reader asking which Decisions stand around a
        // Task is asking about that Task's surroundings, and a walk that
        // could not step through a Task would answer that nothing does.
        let shown = |record: &Record| {
            admission.admits(record)
                && near
                    .as_ref()
                    .is_none_or(|near| near.contains(path_stem(record.path())))
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
    /// order, the body, the ids its body cites, the live records whose
    /// bodies cite it, and the live records whose `link` lines name it,
    /// each under the kind its line gives the relation. A record's own id
    /// never enters its blocks. Reading never gates: an invalid record
    /// shows as it stands.
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
        let others = live
            .records
            .iter()
            .filter(|other| path_stem(other.path()) != id);
        let mentioned_by = others
            .clone()
            .filter(|other| mention::mentions(other.file().body()).contains(&id))
            .map(|other| path_stem(other.path()).to_owned())
            .collect();
        let mut linked_by: Vec<(String, String)> = others
            .flat_map(|other| {
                other
                    .linked_records()
                    .filter(|(_, target)| *target == id)
                    .map(|(kind, _)| (kind.to_owned(), path_stem(other.path()).to_owned()))
            })
            .collect();
        linked_by.sort();
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
            linked_by,
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

    /// Create a record, minting an id from the title unless one is given
    /// and signing it with the notebook's identity unless the draft names
    /// its own. A draft declaring `supersedes` performs the whole supersession: the
    /// new record is written first, then the victim gains the back-pointer
    /// and flips to its superseded state. Links and tags do not imply a
    /// conflict between records.
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
        let by = write::guarded_name("by", draft.by.as_deref().or(self.identity))?;
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
        let id = write::resolve_draft_id(draft, &claims, || write::title_id(draft))?;
        let path = record_path(&id, draft.record_type, false);
        self.storage
            .write(&path, &write::render_draft(draft, &id, by, today))?;

        if let Some(victim) = victim {
            self.flip_victim(victim, &id, today)?;
        }

        // Probed after the write, so a body citing its own record resolves.
        let dangling_mentions = self.dangling_mentions(&draft.body)?;
        Ok(Created {
            id,
            path,
            superseded: draft.supersedes.clone(),
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

    /// `open | review → active`, taking the Task: the identity is recorded
    /// as `taken-by` unless someone already holds it. A Task another
    /// identity holds is refused — handing it over is a correction made
    /// deliberately with `edit --taken-by` — and so is a replay against
    /// it, since answering `already` would tell a second person the work
    /// is theirs. A host that knows nobody takes nothing and is refused
    /// any held Task the same way. Unfinished dependencies do not prevent
    /// accepting responsibility; they gate readiness and completion.
    ///
    /// # Errors
    /// [`NotebookError::Taken`] naming who took it,
    /// [`NotebookError::InvalidTransition`], or the resolution errors of
    /// [`Notebook::close`].
    pub fn start(&mut self, id: &str, today: &str) -> Result<Transitioned, NotebookError> {
        write::guard_today(today)?;
        let identity = self.guarded_identity()?;
        let loaded = self.load_live(id, &[RecordType::Task])?;
        let resuming_review = loaded.task_state() == TaskState::Review;
        let transition = decided(&loaded, TaskAction::Start)?;
        if let Some(taken_by) = loaded.record.taken_by()
            && Some(taken_by) != identity
        {
            return Err(NotebookError::Taken {
                id: id.to_owned(),
                taken_by: taken_by.to_owned(),
            });
        }
        self.transitioned(loaded, transition, today, |file| {
            if let Some(me) = identity {
                file.set_field("taken-by", me);
            }
            if resuming_review {
                file.remove_field("to");
            }
        })
    }

    /// Start the next eligible Task, preferring this identity's work over
    /// unclaimed work. Readiness uses the complete dependency graph before
    /// applying `filter`. `excluded` names Tasks reserved by the host.
    ///
    /// # Errors
    /// The refusals of [`Notebook::ready`] and [`Notebook::start`]. An
    /// empty eligible queue returns `None` without writing a record.
    pub fn start_next(
        &mut self,
        filter: &Filter,
        excluded: &BTreeSet<String>,
        today: &str,
    ) -> Result<Option<Transitioned>, NotebookError> {
        let identity = self.guarded_identity()?;
        let ready = self.ready(filter)?;
        let eligible = |task: &&ReadyTask| !excluded.contains(&task.id);
        let own = ready
            .iter()
            .filter(eligible)
            .find(|task| identity.is_some() && task.attribution.taken_by.as_deref() == identity);
        let next = own.or_else(|| {
            ready
                .iter()
                .filter(eligible)
                .find(|task| task.attribution.taken_by.is_none())
        });
        next.map(|task| self.start(&task.id, today)).transpose()
    }

    /// `active → review`: hand the work to a human for acceptance, `to`
    /// naming whom when the caller says. Absent, the Task keeps the
    /// addressee it carries, or waits on a human in general.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on an empty or multi-line name,
    /// plus the refusals of [`Notebook::close`].
    pub fn submit(
        &mut self,
        id: &str,
        to: Option<&str>,
        today: &str,
    ) -> Result<Transitioned, NotebookError> {
        if let Some(to) = to {
            write::guard_addressee(RecordType::Task, to)?;
        }
        self.task_transition(id, TaskAction::Submit, today, |file| {
            if let Some(to) = to {
                file.set_field("to", to.trim());
            }
        })
    }

    /// Close a completed Task and append its outcome in one record write.
    /// Repeating the close leaves the original outcome unchanged; use
    /// [`Notebook::comment`] to add a correction. Replies name remaining
    /// Questions and newly unblocked Tasks. Successful completion requires
    /// every dependency to be closed; cancellation does not.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on an empty outcome or malformed
    /// signature, [`NotebookError::UnfinishedDependencies`] naming unfinished
    /// prerequisites, [`NotebookError::InvalidTransition`] naming the valid
    /// commands, [`NotebookError::WrongType`], [`NotebookError::UnknownId`],
    /// [`NotebookError::Archived`], [`NotebookError::InvalidRecord`], or a
    /// storage failure.
    pub fn close(
        &mut self,
        id: &str,
        via: Option<&str>,
        outcome: &str,
        today: &str,
    ) -> Result<Closed, NotebookError> {
        let entry = self.authored_entry("close", via, outcome, today)?;
        let dangling_mentions = self.dangling_mentions(outcome)?;
        write::guard_today(today)?;
        let loaded = self.load_live(id, &[RecordType::Task])?;
        let transition = decided(&loaded, TaskAction::Close)?;
        if matches!(transition, Transition::Move { .. }) {
            let blockers = query::unfinished_dependencies(&self.live_corpus()?.records, id);
            if !blockers.is_empty() {
                return Err(NotebookError::UnfinishedDependencies {
                    id: id.to_owned(),
                    blockers,
                });
            }
        }
        let transition = self.transitioned(loaded, transition, today, |file| {
            file.set_field("closed", today);
            file.append_body(&entry);
        })?;
        let dangling_mentions = if transition.already {
            Vec::new()
        } else {
            dangling_mentions
        };
        self.closed(id, transition, dangling_mentions, None)
    }

    /// End a Task or Question with a reason, including an unstarted Task.
    /// The reason lands in the envelope as `reason` with the close date.
    /// The record reaches `closed`, so dependents unblock and an epic
    /// counts it like any close. Unfinished dependencies do not prevent
    /// cancellation and are not changed by it.
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
            resolved_by,
            dangling_mentions,
        })
    }

    /// `closed → open`, dropping the close date and previous review recipient.
    /// Existing links and outcomes remain as history.
    ///
    /// # Errors
    /// See [`Notebook::close`].
    pub fn reopen(&mut self, id: &str, today: &str) -> Result<Transitioned, NotebookError> {
        self.task_transition(id, TaskAction::Reopen, today, |file| {
            file.remove_field("closed");
            file.remove_field("to");
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

    /// Append a dated entry to any live record, signed by the notebook's
    /// identity and optional agent tool. Continuation lines are indented
    /// under the entry so quoted log lines cannot forge another author.
    /// Repeating the final entry with the same date and signature changes
    /// no bytes. Settled records accept comments until they are archived.
    ///
    /// # Errors
    /// [`NotebookError::InvalidArgument`] on an empty entry or invalid signature,
    /// [`NotebookError::UnknownId`], [`NotebookError::Archived`],
    /// [`NotebookError::InvalidRecord`], or a storage failure.
    pub fn comment(
        &mut self,
        id: &str,
        via: Option<&str>,
        text: &str,
        today: &str,
    ) -> Result<Commented, NotebookError> {
        write::guard_today(today)?;
        let entry = self.authored_entry("comment", via, text, today)?;
        let loaded = self.load_live(id, &RecordType::ALL)?;
        // A replay carries the nudge too: the entry is the trail's tail, so
        // its citations stand in the body either way.
        let dangling_mentions = self.dangling_mentions(text)?;
        let body = loaded.record.file().body().trim_end();
        if body == entry || body.ends_with(&format!("\n{entry}")) {
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

    fn authored_entry(
        &self,
        verb: &str,
        via: Option<&str>,
        text: &str,
        today: &str,
    ) -> Result<String, NotebookError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(NotebookError::InvalidArgument {
                reason: format!("{verb}: the text must not be empty"),
            });
        }
        let by = self.guarded_identity()?;
        let via = write::guarded_name("via", via)?;
        Ok(write::log_entry(today, by, via, text))
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
        self.retire_record(id, None, today)
    }

    /// Append an attributed outcome and retire a Decision or Note in one
    /// record write. A repeated retirement leaves the outcome unchanged.
    ///
    /// # Errors
    /// The refusals of [`Notebook::comment`] and [`Notebook::retire`].
    pub fn retire_with_outcome(
        &mut self,
        id: &str,
        via: Option<&str>,
        outcome: &str,
        today: &str,
    ) -> Result<Transitioned, NotebookError> {
        let entry = self.authored_entry("retire", via, outcome, today)?;
        self.retire_record(id, Some(&entry), today)
    }

    fn retire_record(
        &mut self,
        id: &str,
        entry: Option<&str>,
        today: &str,
    ) -> Result<Transitioned, NotebookError> {
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
        if let Some(entry) = entry {
            file.append_body(entry);
        }
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Transitioned {
            id: id.to_owned(),
            from: "active",
            to: "retired",
            already: false,
        })
    }

    /// Move one settled record into the archive without rewriting its
    /// bytes. Linked records keep their state and location. The archive
    /// copy is written before the live file is removed, so an interrupted
    /// move leaves recoverable copies rather than losing the record.
    ///
    /// # Errors
    /// [`NotebookError::InvalidTransition`] on a record still live, naming
    /// the commands that settle it; [`NotebookError::DuplicateId`] on a
    /// destination whose bytes differ from the source; plus the
    /// resolution errors of [`Notebook::close`].
    pub fn archive(&mut self, id: &str) -> Result<Archived, NotebookError> {
        let record_type = write::parsed_type(id)?;
        let mut moved = Archived {
            id: id.to_owned(),
            from: record_path(id, record_type, false),
            to: record_path(id, record_type, true),
            already: false,
        };
        let loaded = match self.resolve_live(id, record_type) {
            Ok(loaded) => loaded,
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
        let source = loaded.record.file().render();
        match self.held_at(&moved.to)? {
            Holding::Absent => self.storage.write(&moved.to, &source)?,
            Holding::Bytes(standing) if source == standing => {}
            _ => {
                return Err(NotebookError::DuplicateId {
                    id: id.to_owned(),
                    holder: moved.to,
                });
            }
        }
        self.storage.remove(&moved.from)?;
        Ok(moved)
    }

    /// Move one archived record back into the working set with the same
    /// filename and bytes. Linked records keep their state and location.
    ///
    /// Invalid but readable records can return for repair. When both homes
    /// hold a file, only byte-identical copies permit the move to resume.
    /// Divergent copies stay untouched: an id cannot establish which one
    /// contains all changes made by other contributors.
    ///
    /// # Errors
    /// [`NotebookError::UnknownId`] when neither home holds the id,
    /// [`NotebookError::DuplicateId`] on a live destination taken by a
    /// file whose bytes differ, [`NotebookError::InvalidRecord`]
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
            (Holding::Bytes(source), Holding::Bytes(standing)) if source == standing => {
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

    /// A replay succeeds only if the archived copy has no error findings.
    fn replayed_archive(&self, moved: &Archived) -> Result<(), NotebookError> {
        let record = match self.storage.read(&moved.to) {
            Ok(text) => Record::parse(&moved.to, &text),
            Err(StorageError::NotUtf8 { .. }) => Record::unreadable(&moved.to),
            Err(error) => return Err(error.into()),
        };
        if !record.has_errors() {
            return Ok(());
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
    /// the standing one moves nothing, including a non-canonical spelling.
    /// A new body carries the quotation rule's
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
            if link.target.trim() == id {
                return Err(NotebookError::InvalidArgument {
                    reason: "link: a record cannot link itself".to_owned(),
                });
            }
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
        let transition = decided(&loaded, action)?;
        self.transitioned(loaded, transition, today, stamp_extras)
    }

    /// The write half of a Task move: a replay changes no byte, a move
    /// splices the state, the verb's own fields and the date.
    fn transitioned(
        &mut self,
        loaded: LoadedLive,
        transition: Transition,
        today: &str,
        stamp_extras: impl FnOnce(&mut RecordFile),
    ) -> Result<Transitioned, NotebookError> {
        let id = path_stem(&loaded.path).to_owned();
        let Transition::Move { from, to } = transition else {
            return Ok(Transitioned::replayed(&id, loaded.task_state().word()));
        };
        let mut file = loaded.record.into_file();
        file.set_field("state", to.word());
        stamp_extras(&mut file);
        file.set_field("updated", today);
        self.storage.write(&loaded.path, &file.render())?;
        Ok(Transitioned {
            id,
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
        for (link, line) in record.file().field_entries("link") {
            if let Some(target) = linked_record(link)
                && self.holder_path(target)?.is_none()
            {
                errors.push(dangling_finding("link", target, line));
            }
        }
        Ok(errors)
    }

    /// Bare ids cited in `text` that this notebook does not hold. These
    /// are informational hints; forward references do not prevent a write.
    fn dangling_mentions(&self, text: &str) -> Result<Vec<String>, NotebookError> {
        let mut dangling = Vec::new();
        for target in mention::mentions(text) {
            if self.holder_path(target)?.is_none() {
                dangling.push(target.to_owned());
            }
        }
        Ok(dangling)
    }

    /// Record-shaped links resolve in this notebook. Other targets are
    /// external references whose availability is not a record invariant.
    fn guard_link_target(&self, link: &Link) -> Result<(), NotebookError> {
        let target = link.target.trim();
        if grammar::id_error(target).is_some() || self.holder_path(target)?.is_some() {
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
        for record_type in RecordType::ALL {
            records.extend(read_records_in(self.storage, record_type.directory())?);
            records.extend(read_records_in(
                self.storage,
                &archive_of(record_type.directory()),
            )?);
        }
        let archived = records
            .iter()
            .filter(|record| is_archived(record.path()))
            .filter_map(|record| resolvable_id(record.path()))
            .map(str::to_owned)
            .collect();
        Ok(Corpus { records, archived })
    }

    /// The live records, with the archive present by name alone: a query
    /// about work in motion costs what the live notebook costs, however far
    /// history has grown behind it.
    fn live_corpus(&self) -> Result<Corpus, NotebookError> {
        read_live_corpus(self.storage)
    }
}

/// What `action` comes to for a loaded Task, decided before any byte moves.
///
/// # Errors
/// [`NotebookError::InvalidTransition`] naming the moves the state allows.
fn decided(loaded: &LoadedLive, action: TaskAction) -> Result<Transition, NotebookError> {
    loaded
        .task_state()
        .transition(action)
        .map_err(|valid| NotebookError::InvalidTransition {
            id: path_stem(&loaded.path).to_owned(),
            state: loaded.state_word().to_owned(),
            valid: valid.into_iter().map(TaskAction::word).collect(),
        })
}

/// A filter is refused before any record is read when it names what no
/// record can carry: a kind no type allows, a tag the grammar rejects, or
/// no text at all. An empty answer to any of these would read as a notebook
/// holding nothing of the kind, which is a true-sounding answer to a
/// question nobody can ask.
fn guard_filter(filter: &Filter) -> Result<(), NotebookError> {
    let invalid = |reason: String| Err(NotebookError::InvalidArgument { reason });
    for kind in &filter.kinds {
        if !RecordType::kind_words().any(|word| word == kind) {
            return invalid(format!(
                "kind: `{kind}` is not one of {}",
                RecordType::kind_words().collect::<Vec<_>>().join(", ")
            ));
        }
    }
    for tag in &filter.tags {
        if !grammar::is_token(tag) {
            return invalid(format!("tags: `{tag}` is not a `[a-z0-9-]+` tag"));
        }
    }
    if filter
        .text
        .as_deref()
        .is_some_and(|text| text.trim().is_empty())
    {
        return invalid("match: the text must not be empty".to_owned());
    }
    Ok(())
}

/// [`Notebook::live_corpus`] over any root, so a second notebook is read by
/// the same rules as the first.
fn read_live_corpus(storage: &dyn Storage) -> Result<Corpus, NotebookError> {
    let mut records = Vec::new();
    for record_type in RecordType::ALL {
        records.extend(read_records_in(storage, record_type.directory())?);
    }
    let mut archived = BTreeSet::new();
    for record_type in RecordType::ALL {
        for path in storage.list(&archive_of(record_type.directory()))? {
            if is_record_file(&path) {
                archived.extend(resolvable_id(&path).map(str::to_owned));
            }
        }
    }
    Ok(Corpus { records, archived })
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

/// Read one archived record; `None` means its canonical file is absent or
/// the id cannot name a record path.
fn read_archived_record(storage: &dyn Storage, id: &str) -> Result<Option<Record>, NotebookError> {
    let Ok(record_type) = write::parsed_type(id) else {
        return Ok(None);
    };
    let path = record_path(id, record_type, true);
    match storage.read(&path) {
        Ok(text) => Ok(Some(Record::parse(&path, &text))),
        Err(StorageError::NotUtf8 { .. }) => Ok(Some(Record::unreadable(&path))),
        Err(StorageError::NotFound { .. }) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// The records one query reads, beside the ids the archive holds. The two
/// are separate because they cost differently: a record is a file opened,
/// an archived id is a name in a listing.
pub(super) struct Corpus {
    records: Vec<Record>,
    archived: BTreeSet<String>,
}

impl Corpus {
    fn resolver(&self) -> Resolver<'_> {
        Resolver::of(&self.records, &self.archived)
    }
}

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

struct Victim {
    path: String,
    record: Record,
    dead_state: &'static str,
}
