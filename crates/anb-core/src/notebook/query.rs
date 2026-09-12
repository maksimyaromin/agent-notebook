//! The folds a verb derives its answer from: the listing and dispatch
//! rows, the membership edges, the consequences a close or a delete must
//! not bury.
//!
//! Nothing here reaches Storage — a fold runs over a corpus the Notebook
//! handed it, which is what keeps a verb to one pass over the notebook.

use crate::debt;
use crate::grammar::{self, Residence};
use crate::graph::{TaskGraph, TaskNode};
use crate::mention;
use crate::record::{REF_KEYS, Record, RecordType, TaskState, linked_record};
use crate::reply::{
    Attribution, Blocker, CitedProof, Counts, Epic, GraphNode, ListedRecord, ReadyTask,
};
use crate::request::Filter;
use crate::resolve::{Resolver, is_archived, path_stem};
use crate::status::{ActiveTask, HeldTask, OpenQuestion, ReviewTask};
use std::collections::{BTreeMap, BTreeSet};

/// A filter settled against one notebook: what it admits, record by
/// record. Narrowing changes what is shown, never what is read, so a row
/// is blocked, excluded and resolved against every record before this is
/// asked of it.
pub(super) struct Admission {
    filter: Filter,
    /// The ids inside the hub's scope, when the filter names a hub.
    scope: Option<BTreeSet<String>>,
    /// The text to find, lowered once rather than once per record.
    needle: Option<String>,
}

impl Admission {
    pub(super) fn of(filter: &Filter, scope: Option<BTreeSet<String>>) -> Admission {
        Admission {
            filter: filter.clone(),
            scope,
            needle: filter.text.as_ref().map(|text| text.trim().to_lowercase()),
        }
    }

    /// Whether the record answers the filter: every narrowing it names
    /// holds at once.
    pub(super) fn admits(&self, record: &Record) -> bool {
        let filter = &self.filter;
        let file = record.file();
        let id = path_stem(record.path());
        (filter.types.is_empty()
            || record
                .record_type()
                .is_some_and(|record_type| filter.types.contains(&record_type)))
            && (filter.kinds.is_empty()
                || file
                    .field("kind")
                    .is_some_and(|kind| filter.kinds.iter().any(|wanted| wanted == kind)))
            && filter
                .tags
                .iter()
                .all(|wanted| tags_of(record).any(|tag| tag == wanted))
            && self.scope.as_ref().is_none_or(|scope| scope.contains(id))
            && filter.by.as_deref().is_none_or(|by| record.concerns(by))
            && (!filter.untaken || is_untaken(record))
            && filter
                .to
                .as_deref()
                .is_none_or(|to| record.addressee() == Some(to))
            && self
                .needle
                .as_deref()
                .is_none_or(|needle| holds_text(record, needle))
    }
}

/// Whether the record is a Task nobody holds: one of the pool anyone may
/// take. A record of another type is held by nobody and is not in the
/// pool either, since only work is taken.
pub(super) fn is_untaken(record: &Record) -> bool {
    record.record_type() == Some(RecordType::Task) && record.taken_by().is_none()
}

/// The tags a record carries, as the envelope lists them.
fn tags_of(record: &Record) -> impl Iterator<Item = &str> {
    record
        .file()
        .field("tags")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
}

/// Every proof these records cite that names something outside the
/// notebook — a commit, or a file — so the host can ask git and the
/// filesystem which of them are still there. A link naming another record
/// resolves inside the notebook and is `check`'s to judge, not the world's.
pub(super) fn cited_proofs(records: &[Record]) -> Vec<CitedProof> {
    records
        .iter()
        .flat_map(|record| {
            let id = path_stem(record.path());
            record.file().field_values("link").filter_map(move |link| {
                let (kind, target) = grammar::split_link(link)?;
                ["sha", "report"].contains(&kind).then(|| CitedProof {
                    record: id.to_owned(),
                    kind: kind.to_owned(),
                    target: target.to_owned(),
                })
            })
        })
        .collect()
}

/// The order records were laid down: `created`, then id.
fn oldest_first(left: &Record, right: &Record) -> std::cmp::Ordering {
    created(left)
        .cmp(created(right))
        .then_with(|| path_stem(left.path()).cmp(path_stem(right.path())))
}

fn created(record: &Record) -> &str {
    record.file().field("created").unwrap_or_default()
}

/// The records this one names as a blocker, as its Origin or as a link
/// target: the edges a walk follows, from the end that carries them.
pub(super) fn kin_of(record: &Record) -> impl Iterator<Item = &str> {
    record
        .file()
        .field_values("blocked-by")
        .chain(record.origin())
        .chain(record.linked_records().map(|(_, target)| target))
}

pub(super) fn edge_exists(record: &Record, target: &str) -> bool {
    record
        .file()
        .field_values("blocked-by")
        .any(|value| value == target)
}

/// The same edges `check`'s `task_edges` draws, each node carrying whether its
/// Task has closed — the flag the ready gate and the unblock consequences
/// read, and the reason this owns its ids rather than borrowing them.
fn task_graph(records: &[Record]) -> TaskGraph {
    let mut nodes = BTreeMap::new();
    for record in records {
        if record.record_type() != Some(RecordType::Task) {
            continue;
        }
        nodes
            .entry(path_stem(record.path()).to_owned())
            .or_insert_with(|| TaskNode {
                closed: record.state() == Some("closed"),
                blocked_by: record.blocked_by().map(str::to_owned).collect(),
            });
    }
    TaskGraph::new(nodes)
}

pub(super) fn unfinished_dependencies(records: &[Record], id: &str) -> Vec<String> {
    task_graph(records)
        .unfinished(id)
        .map(str::to_owned)
        .collect()
}

pub(super) fn listed_row(record: &Record, resolvable: &Resolver<'_>) -> ListedRecord {
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
        attribution: Attribution::of(record),
        title: file.field("title").map(str::to_owned),
    }
}

/// Focus follows graph relationships in either direction. Distance is the
/// shortest path, so records with a shared origin are two edges apart.
/// Lifecycle pointers are not graph edges and do not enter this walk.
pub(super) fn neighbourhood<'a>(
    records: &[&'a Record],
    from: &str,
    depth: usize,
) -> BTreeSet<&'a str> {
    let drawn: BTreeSet<&str> = records
        .iter()
        .map(|record| path_stem(record.path()))
        .collect();
    let Some(root) = drawn.get(from).copied() else {
        return BTreeSet::new();
    };
    let mut neighbours: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for &record in records {
        let id = path_stem(record.path());
        let targets = kin_of(record).chain(mention::mentions(record.file().body()));
        for target in targets.filter(|target| drawn.contains(target)) {
            neighbours.entry(id).or_default().push(target);
            neighbours.entry(target).or_default().push(id);
        }
    }
    let mut visited = BTreeSet::from([root]);
    let mut frontier = vec![root];
    for _ in 0..depth {
        if frontier.is_empty() {
            break;
        }
        frontier = frontier
            .iter()
            .filter_map(|at| neighbours.get(at))
            .flatten()
            .copied()
            .filter(|target| visited.insert(target))
            .collect();
    }
    visited
}

/// One Task as the map draws it: the tile's word, the edges it declares,
/// and the record a reader opens on it.
pub(super) fn graph_node(
    record: &Record,
    resolvable: &Resolver<'_>,
    epics: &[Epic],
    queue: &[ReadyTask],
) -> GraphNode {
    let row = listed_row(record, resolvable);
    let file = record.file();
    GraphNode {
        kind: record.record_type(),
        priority: file.field("priority").and_then(|value| value.parse().ok()),
        created: file.field("created").unwrap_or_default().to_owned(),
        // Only a Task still in play queues, so only there is the answer
        // either yes or no. Saying `no` of a Decision would answer a
        // question nobody can ask of it.
        ready: (record.record_type() == Some(RecordType::Task) && !is_archived(record.path()))
            .then(|| queue.iter().any(|waiting| waiting.id == row.id)),
        epic: epics.iter().find(|epic| epic.id == row.id).cloned(),
        archived: is_archived(record.path()),
        blocked_by: file.field_values("blocked-by").map(str::to_owned).collect(),
        origin: record.origin().map(str::to_owned),
        links: record
            .linked_records()
            .map(|(kind, target)| (kind.to_owned(), target.to_owned()))
            .collect(),
        fields: file
            .fields()
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect(),
        body: file.body().to_owned(),
        mentions: mention::mentions(file.body())
            .into_iter()
            .filter(|target| *target != row.id)
            .map(str::to_owned)
            .collect(),
        id: row.id,
        state: row.state,
        title: row.title,
    }
}

/// Whether the record holds the text: in its id, its title, its tags, its
/// people, or its body, case folded.
fn holds_text(record: &Record, needle: &str) -> bool {
    let file = record.file();
    [
        Some(path_stem(record.path())),
        file.field("title"),
        file.field("tags"),
        file.field("by"),
        file.field("via"),
        file.field("taken-by"),
        Some(file.body()),
    ]
    .into_iter()
    .flatten()
    .any(|surface| surface.to_lowercase().contains(needle))
}

/// Whether the record's file sits in `record_type`'s live directory —
/// residence, the axis a reader browsing the tree sees.
pub(super) fn lives_in(record: &Record, record_type: RecordType) -> bool {
    grammar::residence(record.path(), record_type.word()) == Some(Residence::Live)
}

/// The dispatch queue's rows: live, open, valid, unblocked, unheld Tasks
/// in ready order, the ones `admitted` keeps. Blocked is judged against
/// every record before the narrowing is asked, so a filter cannot free a
/// Task.
pub(super) fn ready_rows(
    records: &[Record],
    resolvable: &Resolver<'_>,
    admitted: impl Fn(&Record) -> bool,
) -> Vec<ReadyTask> {
    let graph = task_graph(records);
    let keep = |record: &Record| {
        record.hold().is_none() && !graph.is_blocked(path_stem(record.path())) && admitted(record)
    };
    open_rows(records, resolvable, keep)
}

/// The open Tasks a close of `id` released, in ready order: their last live
/// blocker was that Task. A held one is named too — the hold gates
/// `ready`, not the fact.
pub(super) fn unblocked_by_close(
    records: &[Record],
    resolvable: &Resolver<'_>,
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

fn ready_row(record: &Record) -> ReadyTask {
    let file = record.file();
    ReadyTask {
        id: path_stem(record.path()).to_owned(),
        priority: file.field("priority").and_then(|value| value.parse().ok()),
        created: file.field("created").unwrap_or_default().to_owned(),
        attribution: Attribution::of(record),
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

/// The live, open, valid Tasks passing `keep`, in ready order.
fn open_rows(
    records: &[Record],
    resolvable: &Resolver<'_>,
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

/// Origin descendants indexed for membership, with dependency edges and
/// closed states retained for epic detection and progress. A membership
/// walk visits indexed descendants instead of rescanning the corpus.
pub(super) struct MembershipIndex<'a> {
    /// What each record waits on, by id.
    waits_on: BTreeMap<&'a str, Vec<&'a str>>,
    /// What was born inside each record, by the origin's id.
    born_inside: BTreeMap<&'a str, Vec<&'a str>>,
    /// The ids of the Tasks that have closed.
    closed: BTreeSet<&'a str>,
}

impl<'a> MembershipIndex<'a> {
    pub(super) fn of(live: &'a [Record], archived: &'a [Record]) -> Self {
        let mut index = MembershipIndex {
            waits_on: BTreeMap::new(),
            born_inside: BTreeMap::new(),
            closed: BTreeSet::new(),
        };
        // Live before archived, so the duplicate-id corruption an
        // interrupted archive move leaves reads as the live file's record
        // and not as the union of two: the first file to claim an id
        // answers for it on every edge at once. Waiting on nothing, being
        // born nowhere and standing open are answers like any other, so a
        // claimed id closes the slot whether or not it filled one.
        for record in live.iter().chain(archived) {
            let id = path_stem(record.path());
            if index.waits_on.contains_key(id) {
                continue;
            }
            index
                .waits_on
                .insert(id, record.file().field_values("blocked-by").collect());
            if let Some(origin) = record.origin() {
                index.born_inside.entry(origin).or_default().push(id);
            }
            if record.state() == Some(TaskState::Closed.word()) {
                index.closed.insert(id);
            }
        }
        index
    }

    /// An epic is a hub Task, distinguished by its edges: it is blocked by
    /// at least one record that also carries it as Origin — one edge
    /// written from each end. The pairing is the discriminator. A Task
    /// blocked by a plain dependency did not give birth to it, and a Task
    /// that spawned a Question does not wait on it, so neither becomes an
    /// epic by accident.
    ///
    /// A hub that no longer binds is not an epic in flight: its acceptance
    /// close has happened, and a dashboard that still asked for it would be
    /// asking for work already done.
    fn is_hub(&self, record: &Record, resolvable: &Resolver<'_>) -> bool {
        if record.record_type() != Some(RecordType::Task)
            || !record.is_live()
            || is_archived(record.path())
            || debt::is_excluded(record, resolvable)
        {
            return false;
        }
        let hub = path_stem(record.path());
        let children = self.born_inside.get(hub).map_or(&[][..], Vec::as_slice);
        self.waits_on
            .get(hub)
            .is_some_and(|waits| waits.iter().any(|target| children.contains(target)))
    }

    /// The subject and its transitive origin descendants. A prerequisite
    /// constrains readiness but does not become part of the subject;
    /// contextual links belong to neighbourhood queries instead.
    pub(super) fn scope_of(&self, hub: &'a str) -> BTreeSet<&'a str> {
        let mut scope = BTreeSet::from([hub]);
        let mut frontier = vec![hub];
        while let Some(id) = frontier.pop() {
            for &reached in self.born_inside.get(id).into_iter().flatten() {
                if scope.insert(reached) {
                    frontier.push(reached);
                }
            }
        }
        scope
    }
}

/// The hubs and where each stands, in notebook order. Progress counts the
/// hub's own `blocked-by` children, which are its statement of what it
/// waits on, while `next` selects among its origin descendants, not external
/// prerequisites. A hub never nominates itself: one that
/// reaches `ready` is asking for its acceptance close, not for work.
pub(super) fn epic_rows(
    records: &[Record],
    archived: &[Record],
    resolvable: &Resolver<'_>,
    queue: &[ReadyTask],
) -> Vec<Epic> {
    let index = MembershipIndex::of(records, archived);
    records
        .iter()
        .filter(|record| index.is_hub(record, resolvable))
        .map(|hub| {
            let id = path_stem(hub.path());
            let children: Vec<&str> = hub.file().field_values("blocked-by").collect();
            let scope = index.scope_of(id);
            Epic {
                id: id.to_owned(),
                closed: children
                    .iter()
                    .filter(|child| index.closed.contains(*child))
                    .count(),
                total: children.len(),
                next: queue
                    .iter()
                    .find(|row| row.id != id && scope.contains(row.id.as_str()))
                    .map(|row| row.id.clone()),
            }
        })
        .collect()
}

/// The still-open, valid Questions whose Origin is this Task; an invalid
/// one is `check`'s to name, as everywhere.
pub(super) fn open_questions_from(
    records: &[Record],
    resolvable: &Resolver<'_>,
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

/// The live records per type, counted by the directory each file sits in
/// — the rule the type sections of the same reply are grouped by, so a
/// record filed under the wrong type is counted where it is shown.
pub(super) fn live_counts(records: &[Record]) -> Counts {
    Counts::per_type(RecordType::ALL.map(|record_type| {
        let held = records
            .iter()
            .filter(|record| lives_in(record, record_type))
            .count();
        (record_type, held)
    }))
}

/// The live Tasks on hold, the reader's own first and then by id: each
/// with the reason it waits for and the day it resumes on, when one was
/// set. A closed Task still carrying its hold line is settled work, not
/// something that waits.
pub(super) fn held_tasks(live_valid: &[&Record], identity: Option<&str>) -> Vec<HeldTask> {
    let held = live_valid.iter().copied().filter(|record| {
        record.record_type() == Some(RecordType::Task)
            && record.is_live()
            && record.hold().is_some()
    });
    own_first(held, identity, |left, right| {
        path_stem(left.path()).cmp(path_stem(right.path()))
    })
    .into_iter()
    .map(|(record, attribution)| HeldTask {
        id: path_stem(record.path()).to_owned(),
        reason: record.hold().unwrap_or_default().to_owned(),
        until: record.hold_until().map(str::to_owned),
        attribution,
    })
    .collect()
}

/// The records with the reader's own leading — what they hold, what they
/// wrote, and what waits on them — and `within` ordering each tier, so a
/// notebook several people work in opens on the reader's work wherever
/// the section. A host that knows nobody has nothing to lead with, and
/// the order is `within` alone.
fn own_first<'a>(
    records: impl Iterator<Item = &'a Record>,
    identity: Option<&str>,
    within: impl Fn(&Record, &Record) -> std::cmp::Ordering,
) -> Vec<(&'a Record, Attribution)> {
    let mut ranked: Vec<&Record> = records.collect();
    let others_first = |record: &Record| !identity.is_some_and(|me| record.concerns(me));
    ranked.sort_by(|left, right| {
        others_first(left)
            .cmp(&others_first(right))
            .then_with(|| within(left, right))
    });
    ranked
        .into_iter()
        .map(|record| (record, Attribution::of(record)))
        .collect()
}

/// Active Tasks retain their own continuations. Recency orders a work list;
/// it does not select which parallel session the caller means to resume.
pub(super) fn active_tasks(live_valid: &[&Record], identity: Option<&str>) -> Vec<ActiveTask> {
    let active = live_valid.iter().copied().filter(|record| {
        record.record_type() == Some(RecordType::Task)
            && record.state() == Some("active")
            && record.hold().is_none()
    });
    own_first(active, identity, |left, right| {
        touched(right)
            .cmp(touched(left))
            .then_with(|| path_stem(left.path()).cmp(path_stem(right.path())))
    })
    .into_iter()
    .map(|(record, attribution)| ActiveTask {
        id: path_stem(record.path()).to_owned(),
        title: record.file().field("title").unwrap_or_default().to_owned(),
        attribution,
        log: last_log_line(record),
    })
    .collect()
}

/// The latest dated log entry, including its indented Markdown continuation.
/// Older handoffs without a dated log fall back to their last non-empty line.
pub(super) fn last_log_line(record: &Record) -> Option<String> {
    let body = record.file().body();
    let lines = body.lines().collect::<Vec<_>>();
    let last_entry = lines.iter().rposition(|line| {
        line.strip_prefix("- ")
            .and_then(|text| text.get(..10))
            .is_some_and(|day| crate::date::day_number(day).is_some())
    });
    if let Some(start) = last_entry {
        let end = lines[start + 1..]
            .iter()
            .position(|line| !line.is_empty() && !line.starts_with("  "))
            .map_or(lines.len(), |relative| start + 1 + relative);
        return Some(lines[start..end].join("\n").trim_end().to_owned());
    }
    lines
        .into_iter()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::to_owned)
}

/// The Tasks waiting for acceptance, the reader's own first — held by
/// them or waiting on them — and then by id.
pub(super) fn review_tasks(live_valid: &[&Record], identity: Option<&str>) -> Vec<ReviewTask> {
    let waiting = live_valid.iter().copied().filter(|record| {
        record.record_type() == Some(RecordType::Task) && record.state() == Some("review")
    });
    own_first(waiting, identity, |left, right| {
        path_stem(left.path()).cmp(path_stem(right.path()))
    })
    .into_iter()
    .map(|(record, attribution)| ReviewTask {
        id: path_stem(record.path()).to_owned(),
        attribution,
    })
    .collect()
}

/// The open Questions, the reader's own first and then oldest first: the
/// doubts a session is about to work past.
pub(super) fn open_questions(live_valid: &[&Record], identity: Option<&str>) -> Vec<OpenQuestion> {
    let open = live_valid.iter().copied().filter(|record| {
        record.record_type() == Some(RecordType::Question) && record.state() == Some("open")
    });
    own_first(open, identity, oldest_first)
        .into_iter()
        .map(|(record, attribution)| OpenQuestion {
            id: path_stem(record.path()).to_owned(),
            created: created(record).to_owned(),
            attribution,
            title: record.file().field("title").unwrap_or_default().to_owned(),
        })
        .collect()
}

/// When the record last moved: `updated`, else `created` — the same proxy
/// the Debt clocks subtract from.
fn touched(record: &Record) -> &str {
    let file = record.file();
    file.field("updated")
        .or_else(|| file.field("created"))
        .unwrap_or_default()
}

/// Every edge pointing at `target` from somewhere else, in file order.
///
/// An envelope key and a body citation both count, and an invalid record's
/// edges count too: what makes an edge a blocker is that removing the
/// target would leave it pointing at nothing, and a file the tool refuses
/// to mutate is the worst place to leave that. The target's own edges are
/// not blockers — they leave with it.
pub(super) fn inbound_edges(records: &[Record], target: &str) -> Vec<Blocker> {
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
