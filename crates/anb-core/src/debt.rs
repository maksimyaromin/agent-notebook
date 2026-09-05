//! Debt: the computed signs of decay a Status surfaces.
//!
//! Every clock is leading — computed from live open records' envelope dates
//! at read time, keyed on Origin; nothing stores a score. The mention-borne
//! signals are hints for a reader, never Check findings: the body is opaque
//! prose and a citation in it is a hint, not an invalidity, in whichever
//! notebook the id it names turns up.

use crate::date;
use crate::encode::quoted_if_delimited;
use crate::grammar;
use crate::mention;
use crate::record::{REF_KEYS, Record, RecordType, linked_record};
use crate::reply::{Cited, CitedProof};
use crate::resolve::{Resolver, path_stem};
use std::collections::BTreeSet;

/// The Debt clocks, in days, each behind its `debt-*` config key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebtThresholds {
    pub task_stale: u32,
    pub question_age: u32,
    pub question_age_task_born: u32,
    pub hold_stale: u32,
    pub review_stale: u32,
}

/// One sign of decay: a clock past its threshold, a mention-borne hint, or
/// an invalid file; each renders as one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebtSignal {
    /// An active Task whose `updated` stopped moving.
    TaskStale { id: String, days: u32 },
    /// An open Question past its origin-keyed age threshold.
    QuestionAge { id: String, days: u32 },
    /// A task-born open Question whose origin Task is closed: the context
    /// that makes it cheap to route is being archived, so it surfaces now.
    OriginClosed { id: String, origin: String },
    /// A held Task whose `updated` stopped moving.
    HoldStale { id: String, days: u32 },
    /// A review Task waiting on a human.
    ReviewStale { id: String, days: u32 },
    /// A `review-by` date that has arrived, on any record type.
    ReviewDue { id: String, date: String },
    /// A body citation of an id no record carries.
    DanglingMention { id: String, target: String },
    /// Two live Decisions where one cites the other with no declared edge.
    MayConflict { first: Cited, second: Cited },
    /// A live project Decision and a standing Decision of the user's
    /// notebook, paired by a citation. Where the two disagree the project's
    /// governs this repository — that is what a scope is — so the pair is
    /// named and nothing is resolved.
    Shadow { project: Cited, user: Cited },
    /// A record excluded from every derived query by its error findings.
    Invalid { path: String, errors: usize },
    /// A proof naming something the world no longer holds — a commit this
    /// repository does not have, a report file that is gone. Git and the
    /// working tree outrank the record: the proof is what is wrong.
    LostProof { id: String, proof: String },
}

impl DebtSignal {
    /// The stable kebab-case code, shared by the plain and json surfaces.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            DebtSignal::TaskStale { .. } => "task-stale",
            DebtSignal::QuestionAge { .. } => "question-age",
            DebtSignal::OriginClosed { .. } => "origin-closed",
            DebtSignal::HoldStale { .. } => "hold-stale",
            DebtSignal::ReviewStale { .. } => "review-stale",
            DebtSignal::ReviewDue { .. } => "review-due",
            DebtSignal::DanglingMention { .. } => "dangling-mention",
            DebtSignal::MayConflict { .. } => "may-conflict",
            DebtSignal::Shadow { .. } => "shadow",
            DebtSignal::Invalid { .. } => "invalid",
            DebtSignal::LostProof { .. } => "lost-proof",
        }
    }

    /// The line under the `debt` header, code first: one shape per signal.
    #[must_use]
    pub fn line(&self) -> String {
        let code = self.code();
        match self {
            DebtSignal::TaskStale { id, days }
            | DebtSignal::QuestionAge { id, days }
            | DebtSignal::HoldStale { id, days }
            | DebtSignal::ReviewStale { id, days } => format!("{code}: {id} ({days}d)"),
            DebtSignal::OriginClosed { id, origin } => format!("{code}: {id} ({origin} closed)"),
            DebtSignal::ReviewDue { id, date } => format!("{code}: {id} ({date})"),
            DebtSignal::DanglingMention { id, target } => {
                format!("{code}: {id} -> {}", quoted_if_delimited(target))
            }
            DebtSignal::MayConflict { first, second } => format!(
                "{code}: {} ({}) <-> {} ({})",
                first.id,
                first.author(),
                second.id,
                second.author()
            ),
            DebtSignal::Shadow { project, user } => format!(
                "{code}: {} ({}) <-> global {} ({})",
                project.id,
                project.author(),
                user.id,
                user.author()
            ),
            DebtSignal::Invalid { path, errors } => {
                let unit = if *errors == 1 { "error" } else { "errors" };
                format!("{code}: {} ({errors} {unit})", quoted_if_delimited(path))
            }
            DebtSignal::LostProof { id, proof } => {
                format!("{code}: {id} -> {}", quoted_if_delimited(proof))
            }
        }
    }
}

/// Everything the Debt pass reads: the live records and what a reference
/// among them may resolve to — existence checks run against the resolver,
/// never against storage.
pub(crate) struct DebtSources<'a> {
    /// The live records. Every clock and every hint is about the work
    /// still in play, and a filed record is not that, so no signal here
    /// opens one — the resolver answers for them by name.
    pub records: &'a [Record],
    pub resolvable: &'a Resolver<'a>,
    pub today_day: i64,
    /// The cited proofs the world no longer holds, as the host found them.
    pub lost_proofs: &'a [CitedProof],
    /// The user's notebook behind this one: its live records and the ids
    /// its archive holds, both empty when a project is read alone.
    pub user_records: &'a [Record],
    pub user_archived: &'a BTreeSet<String>,
}

/// A record's error findings plus a reference into nothing: the exclusion
/// rule of the derived queries, answered from the resolver.
pub(crate) fn is_excluded(record: &Record, resolvable: &Resolver<'_>) -> bool {
    excluding_errors(record, resolvable) > 0
}

fn excluding_errors(record: &Record, resolvable: &Resolver<'_>) -> usize {
    let own = record
        .findings()
        .iter()
        .filter(|finding| finding.is_error())
        .count();
    let dangling = reference_targets(record)
        .filter(|target| !resolvable.resolves(target))
        .count();
    own + dangling
}

fn reference_targets(record: &Record) -> impl Iterator<Item = &str> {
    REF_KEYS.into_iter().flat_map(|key| {
        record
            .file()
            .field_values(key)
            .filter(|target| grammar::id_error(target).is_none())
    })
}

/// Every Debt signal of the notebook, in the clock table's order, oldest
/// first within a class.
pub(crate) fn signals(sources: &DebtSources<'_>, thresholds: &DebtThresholds) -> Vec<DebtSignal> {
    // Answering this walks every reference through the resolver, and both
    // the exclusion below and the invalid signal further down ask it.
    let error_counts: Vec<usize> = sources
        .records
        .iter()
        .map(|record| excluding_errors(record, sources.resolvable))
        .collect();
    let valid: Vec<&Record> = sources
        .records
        .iter()
        .zip(&error_counts)
        .filter(|(_, errors)| **errors == 0)
        .map(|(record, _)| record)
        .collect();

    let behind = Resolver::of(sources.user_records, sources.user_archived);
    let mut classes = SignalClasses::default();
    for record in &valid {
        collect_clock_signals(record, sources, thresholds, &mut classes);
        collect_dangling_mentions(record, sources.resolvable, &behind, &mut classes);
    }
    classes.pairs = may_conflict_pairs(&valid, sources.resolvable);
    classes.shadows = shadows(&valid, sources.resolvable, &behind);
    classes.lost_proofs = lost_proofs(sources);
    // A corrupt live file is a hint the reader can act on today; a corrupt
    // filed one is `check`'s to name.
    for (record, errors) in sources.records.iter().zip(error_counts) {
        if errors > 0 {
            classes.invalid.push(DebtSignal::Invalid {
                path: record.path().to_owned(),
                errors,
            });
        }
    }
    classes.into_ordered()
}

/// The signals in class order; each class sorts before the merge.
#[derive(Default)]
struct SignalClasses {
    stale: Vec<DebtSignal>,
    aging: Vec<DebtSignal>,
    origin_closed: Vec<DebtSignal>,
    quiet_holds: Vec<DebtSignal>,
    review_stales: Vec<DebtSignal>,
    review_due: Vec<DebtSignal>,
    dangling: Vec<DebtSignal>,
    pairs: Vec<DebtSignal>,
    shadows: Vec<DebtSignal>,
    lost_proofs: Vec<DebtSignal>,
    invalid: Vec<DebtSignal>,
}

impl SignalClasses {
    fn into_ordered(self) -> Vec<DebtSignal> {
        let mut ordered = Vec::new();
        for mut class in [
            self.stale,
            self.aging,
            self.origin_closed,
            self.quiet_holds,
            self.review_stales,
            self.review_due,
            self.dangling,
            self.pairs,
            self.shadows,
            self.lost_proofs,
            self.invalid,
        ] {
            class.sort_by_cached_key(signal_rank);
            ordered.extend(class);
        }
        ordered
    }
}

/// The proofs the world no longer holds, as the host settled them.
///
/// Between the record and the world outside it, the world wins: a commit
/// that is gone was rebased or dropped, a report file that is gone was
/// moved or deleted, and either way the record is left pointing at nothing.
/// Nothing is repaired — the tool cannot know what the proof meant, and
/// inventing one would be worse than naming the gap. Any live record
/// counts, not only a closed one: a stale proof is stale while the record
/// carrying it can still be reopened and closed again on a proof that
/// holds. Once the record is archived nothing can, so the claim is history
/// like the rest of it. A record already excluded by its own errors is left
/// to `check`, which names it once and better.
fn lost_proofs(sources: &DebtSources<'_>) -> Vec<DebtSignal> {
    sources
        .lost_proofs
        .iter()
        .filter(|lost| {
            sources
                .resolvable
                .read(&lost.record)
                .is_some_and(|record| !is_excluded(record, sources.resolvable))
        })
        .map(|lost| DebtSignal::LostProof {
            id: lost.record.clone(),
            proof: lost.to_string(),
        })
        .collect()
}

/// Oldest first where the signal carries an age; the classes without one
/// order by their stable names — except the two paired classes, which
/// arrive in an order of their own and keep it through the stable sort:
/// undeclared pairs ranked by their older member's `created`, shadows in
/// the order the project records name them.
fn signal_rank(signal: &DebtSignal) -> (i64, String) {
    match signal {
        DebtSignal::TaskStale { id, days }
        | DebtSignal::QuestionAge { id, days }
        | DebtSignal::HoldStale { id, days }
        | DebtSignal::ReviewStale { id, days } => (-i64::from(*days), id.clone()),
        DebtSignal::OriginClosed { id, .. } => (0, id.clone()),
        DebtSignal::ReviewDue { id, date } => (0, format!("{date} {id}")),
        DebtSignal::DanglingMention { id, target } => (0, format!("{id} {target}")),
        DebtSignal::MayConflict { .. } | DebtSignal::Shadow { .. } => (0, String::new()),
        DebtSignal::Invalid { path, .. } => (0, path.clone()),
        DebtSignal::LostProof { id, proof } => (0, format!("{id} {proof}")),
    }
}

/// Every clock is leading: it ticks only on records whose state still
/// binds — a closed Task or a superseded Decision is history whatever
/// fields it still carries.
fn collect_clock_signals(
    record: &Record,
    sources: &DebtSources<'_>,
    thresholds: &DebtThresholds,
    classes: &mut SignalClasses,
) {
    let Some(record_type) = record.record_type() else {
        return;
    };
    if !record.is_live() {
        return;
    }
    let id = path_stem(record.path()).to_owned();
    let quiet_days = days_since_touch(record, sources.today_day);

    if let Some(date) = record.file().field("review-by")
        && date::day_number(date).is_some_and(|due| sources.today_day >= due)
    {
        classes.review_due.push(DebtSignal::ReviewDue {
            id: id.clone(),
            date: date.to_owned(),
        });
    }

    match record_type {
        RecordType::Task => {
            collect_task_clocks(record, &id, quiet_days, thresholds, classes);
        }
        RecordType::Question => {
            collect_question_clocks(record, &id, quiet_days, sources, thresholds, classes);
        }
        RecordType::Decision | RecordType::Note => {}
    }
}

/// A held Task's silence reads as its hold gone quiet, never as staleness
/// or review wait: the hold already explains why nothing moves, so no
/// second clock fires beside it.
fn collect_task_clocks(
    record: &Record,
    id: &str,
    quiet_days: u32,
    thresholds: &DebtThresholds,
    classes: &mut SignalClasses,
) {
    if record.hold().is_some() {
        if quiet_days >= thresholds.hold_stale {
            classes.quiet_holds.push(DebtSignal::HoldStale {
                id: id.to_owned(),
                days: quiet_days,
            });
        }
        return;
    }
    match record.state() {
        Some("active") if quiet_days >= thresholds.task_stale => {
            classes.stale.push(DebtSignal::TaskStale {
                id: id.to_owned(),
                days: quiet_days,
            });
        }
        Some("review") if quiet_days >= thresholds.review_stale => {
            classes.review_stales.push(DebtSignal::ReviewStale {
                id: id.to_owned(),
                days: quiet_days,
            });
        }
        _ => {}
    }
}

/// Origin-keyed aging: a task-born Question ages faster than a free-standing
/// one, and surfaces immediately once its origin Task closes — routing is
/// cheap while the origin's context is alive, worthless once it is archived.
fn collect_question_clocks(
    record: &Record,
    id: &str,
    quiet_days: u32,
    sources: &DebtSources<'_>,
    thresholds: &DebtThresholds,
    classes: &mut SignalClasses,
) {
    let origin_task = record
        .origin()
        .filter(|origin| origin.starts_with("task.") && sources.resolvable.resolves(origin));
    if let Some(origin) = origin_task
        && origin_settled(origin, sources.resolvable)
    {
        classes.origin_closed.push(DebtSignal::OriginClosed {
            id: id.to_owned(),
            origin: origin.to_owned(),
        });
        return;
    }
    let threshold = if origin_task.is_some() {
        thresholds.question_age_task_born
    } else {
        thresholds.question_age
    };
    if quiet_days >= threshold {
        classes.aging.push(DebtSignal::QuestionAge {
            id: id.to_owned(),
            days: quiet_days,
        });
    }
}

/// Whether the Task an origin names has stopped carrying context: it does
/// at its close, and archiving it is a close by residence — which the
/// archive's listing says, so no filed record is opened to ask.
fn origin_settled(origin: &str, resolvable: &Resolver<'_>) -> bool {
    resolvable.archived(origin)
        || resolvable
            .read(origin)
            .is_some_and(|task| task.state() == Some("closed"))
}

/// A citation names nothing only when neither notebook holds it. A reader
/// standing in a project reaches the user's notebook too, so an id it
/// carries is one they can open.
fn collect_dangling_mentions(
    record: &Record,
    resolvable: &Resolver<'_>,
    behind: &Resolver<'_>,
    classes: &mut SignalClasses,
) {
    for target in mention::mentions(record.file().body()) {
        if !resolvable.resolves(target) && !behind.resolves(target) {
            classes.dangling.push(DebtSignal::DanglingMention {
                id: path_stem(record.path()).to_owned(),
                target: target.to_owned(),
            });
        }
    }
}

/// The cross-scope pairs, in the order the project records name them.
///
/// A project Decision reaches one of the user's through its prose or through
/// a `link`: the envelope edges must be answered by this notebook, which
/// `check` enforces, so a citation and a link are the edges that survive. A
/// typed id in either is deliberate, the construction [`may_conflict_pairs`]
/// rests on.
///
/// A citation this notebook answers with a live record of its own is about
/// that record and belongs to the pair below; only what this notebook has
/// no live answer for reaches across. So a notebook read behind itself
/// pairs with nobody, and a rule this project retired stops hiding the
/// user's, which still stands.
fn shadows(valid: &[&Record], resolvable: &Resolver<'_>, behind: &Resolver<'_>) -> Vec<DebtSignal> {
    let mut found = Vec::new();
    for record in valid
        .iter()
        .copied()
        .filter(|record| record.record_type() == Some(RecordType::Decision) && record.is_live())
    {
        for target in cited_ids(record) {
            if resolvable.read(target).is_some() {
                continue;
            }
            let Some(rule) = behind.read(target).filter(|rule| is_standing(rule, behind)) else {
                continue;
            };
            found.push(DebtSignal::Shadow {
                project: Cited::of(record),
                user: Cited::of(rule),
            });
        }
    }
    found
}

/// The ids a record names as a reader would follow them, each once and in
/// the order the record names them: bare ids in its prose, then the records
/// its `link` lines point at. One id named both ways is one edge, so it is
/// one pair.
fn cited_ids(record: &Record) -> Vec<&str> {
    let mut seen = BTreeSet::new();
    mention::mentions(record.file().body())
        .into_iter()
        .chain(
            record
                .file()
                .field_entries("link")
                .filter_map(|(link, _)| linked_record(link)),
        )
        .filter(|id| seen.insert(*id))
        .collect()
}

/// Whether a record of the notebook behind this one is a rule still
/// binding. An invalid record is out of every derived query, and a pair
/// drawn from one would name an id its own envelope disowns.
fn is_standing(record: &Record, behind: &Resolver<'_>) -> bool {
    record.record_type() == Some(RecordType::Decision)
        && record.is_live()
        && !is_excluded(record, behind)
}

/// The undeclared-conflict heuristic: a live Decision citing another live
/// Decision with no declared relationship in either envelope, both valid — an
/// invalid record is out of every derived query, half a pair included.
/// Ranked by the older member's `created`, oldest first. High precision by
/// construction — a typed id in prose is a deliberate reference.
fn may_conflict_pairs(valid: &[&Record], resolvable: &Resolver<'_>) -> Vec<DebtSignal> {
    let mut found: Vec<RankedPair> = Vec::new();
    let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
    let live_decision =
        |record: &Record| record.record_type() == Some(RecordType::Decision) && record.is_live();
    let valid_paths: BTreeSet<&str> = valid.iter().map(|record| record.path()).collect();
    for record in valid.iter().copied().filter(|record| live_decision(record)) {
        let citer = path_stem(record.path());
        for target in mention::mentions(record.file().body()) {
            let Some(other) = resolvable.read(target) else {
                continue;
            };
            if target == citer || !live_decision(other) || !valid_paths.contains(other.path()) {
                continue;
            }
            if declares_edge(record, target) || declares_edge(other, citer) {
                continue;
            }
            let (first, second) = if citer <= target {
                (record, other)
            } else {
                (other, record)
            };
            if !seen.insert((path_stem(first.path()), path_stem(second.path()))) {
                continue;
            }
            let (one, other) = (Cited::of(first), Cited::of(second));
            found.push(RankedPair {
                older_created: [first, second]
                    .into_iter()
                    .filter_map(|member| member.file().field("created"))
                    .min()
                    .unwrap_or_default()
                    .to_owned(),
                first: one,
                second: other,
            });
        }
    }
    found.sort_by(|left, right| left.rank().cmp(&right.rank()));
    found
        .into_iter()
        .map(|pair| DebtSignal::MayConflict {
            first: pair.first,
            second: pair.second,
        })
        .collect()
}

/// An undeclared pair with what ranks it: the older member's `created`,
/// read where both members are still at hand.
struct RankedPair {
    older_created: String,
    first: Cited,
    second: Cited,
}

impl RankedPair {
    fn rank(&self) -> (&str, &str, &str) {
        (&self.older_created, &self.first.id, &self.second.id)
    }
}

/// Any reference the envelope draws counts as declared: the pair heuristic
/// hunts only relationships that exist nowhere but in prose.
fn declares_edge(record: &Record, target: &str) -> bool {
    let names_target = |key| record.file().field_values(key).any(|value| value == target);
    REF_KEYS.into_iter().any(names_target)
        || record
            .file()
            .field_values("link")
            .any(|link| link.split_whitespace().any(|word| word == target))
}

/// Days since the record was last touched: `updated`, else `created` — the
/// honest proxy for "last confirmed true". A future date reads as today:
/// clocks never run backwards.
fn days_since_touch(record: &Record, today_day: i64) -> u32 {
    let touched = record
        .file()
        .field("updated")
        .or_else(|| record.file().field("created"))
        .and_then(date::day_number);
    match touched {
        Some(day) => u32::try_from(today_day - day).unwrap_or(0),
        None => 0,
    }
}
