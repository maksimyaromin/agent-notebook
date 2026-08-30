//! Debt: the computed signs of decay a Status surfaces.
//!
//! Every clock is leading — computed from live open records' envelope dates
//! at read time, keyed on Origin; nothing stores a score. The mention-borne
//! signals — a dangling Mention, an undeclared Decision pair — are hints
//! for a reader, never Check findings: the body is opaque prose and a
//! citation in it is a hint, not an invalidity.

use crate::date;
use crate::grammar;
use crate::mention;
use crate::record::{REF_KEYS, Record, RecordType};
use crate::reply::{Cited, CitedProof};
use crate::resolve::{Resolver, path_stem};
use std::collections::BTreeSet;

/// The Debt clocks, in days, each behind its `debt-*` config key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebtThresholds {
    pub task_stale: u32,
    pub question_age: u32,
    pub question_age_task_born: u32,
    pub hold_quiet: u32,
    pub review_wait: u32,
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
    HoldQuiet { id: String, days: u32 },
    /// A review Task waiting on a human.
    ReviewWait { id: String, days: u32 },
    /// A `review-by` date that has arrived, on any record type.
    ReviewDue { id: String, date: String },
    /// A body citation of an id no record carries.
    DanglingMention { id: String, target: String },
    /// Two live Decisions where one cites the other with no declared edge.
    UndeclaredPair { first: Cited, second: Cited },
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
            DebtSignal::HoldQuiet { .. } => "hold-quiet",
            DebtSignal::ReviewWait { .. } => "review-wait",
            DebtSignal::ReviewDue { .. } => "review-due",
            DebtSignal::DanglingMention { .. } => "dangling-mention",
            DebtSignal::UndeclaredPair { .. } => "undeclared-pair",
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
            | DebtSignal::HoldQuiet { id, days }
            | DebtSignal::ReviewWait { id, days } => format!("{code}: {id} ({days}d)"),
            DebtSignal::OriginClosed { id, origin } => format!("{code}: {id} ({origin} closed)"),
            DebtSignal::ReviewDue { id, date } => format!("{code}: {id} ({date})"),
            DebtSignal::DanglingMention { id, target } => format!("{code}: {id} -> {target}"),
            DebtSignal::UndeclaredPair { first, second } => format!(
                "{code}: {} ({}) <-> {} ({})",
                first.id,
                first.author(),
                second.id,
                second.author()
            ),
            DebtSignal::Invalid { path, errors } => {
                let unit = if *errors == 1 { "error" } else { "errors" };
                format!("{code}: {path} ({errors} {unit})")
            }
            DebtSignal::LostProof { id, proof } => format!("{code}: {id} -> {proof}"),
        }
    }
}

/// Everything the Debt pass reads: the live records and what a reference
/// among them may resolve to — existence checks run against the resolver,
/// never against storage.
pub(crate) struct DebtSources<'a> {
    /// The live records. Every clock and every hint is about what a reader
    /// can act on today, and no verb reaches into the archive, so no signal
    /// here opens a filed record — the resolver answers for them by name.
    pub records: &'a [Record],
    pub resolvable: &'a Resolver<'a>,
    pub today_day: i64,
    /// The cited proofs the world no longer holds, as the host found them.
    pub lost_proofs: &'a [CitedProof],
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
    let valid: Vec<&Record> = sources
        .records
        .iter()
        .filter(|record| !is_excluded(record, sources.resolvable))
        .collect();

    let mut classes = SignalClasses::default();
    for record in &valid {
        collect_clock_signals(record, sources, thresholds, &mut classes);
        collect_dangling_mentions(record, sources.resolvable, &mut classes);
    }
    classes.pairs = undeclared_pairs(&valid, sources.resolvable);
    classes.lost_proofs = lost_proofs(sources);
    // A corrupt live file is a hint the reader can act on today; a corrupt
    // filed one is `check`'s to name.
    for record in sources.records {
        let errors = excluding_errors(record, sources.resolvable);
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
    review_waits: Vec<DebtSignal>,
    review_due: Vec<DebtSignal>,
    dangling: Vec<DebtSignal>,
    pairs: Vec<DebtSignal>,
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
            self.review_waits,
            self.review_due,
            self.dangling,
            self.pairs,
            self.lost_proofs,
            self.invalid,
        ] {
            class.sort_by_key(signal_rank);
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
/// order by their stable names — except pairs, which arrive already ranked
/// by their older member's `created` and keep that order through the
/// stable sort.
fn signal_rank(signal: &DebtSignal) -> (i64, String) {
    match signal {
        DebtSignal::TaskStale { id, days }
        | DebtSignal::QuestionAge { id, days }
        | DebtSignal::HoldQuiet { id, days }
        | DebtSignal::ReviewWait { id, days } => (-i64::from(*days), id.clone()),
        DebtSignal::OriginClosed { id, .. } => (0, id.clone()),
        DebtSignal::ReviewDue { id, date } => (0, format!("{date} {id}")),
        DebtSignal::DanglingMention { id, target } => (0, format!("{id} {target}")),
        DebtSignal::UndeclaredPair { .. } => (0, String::new()),
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
        if quiet_days >= thresholds.hold_quiet {
            classes.quiet_holds.push(DebtSignal::HoldQuiet {
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
        Some("review") if quiet_days >= thresholds.review_wait => {
            classes.review_waits.push(DebtSignal::ReviewWait {
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

fn collect_dangling_mentions(
    record: &Record,
    resolvable: &Resolver<'_>,
    classes: &mut SignalClasses,
) {
    for target in mention::mentions(record.file().body()) {
        if !resolvable.resolves(target) {
            classes.dangling.push(DebtSignal::DanglingMention {
                id: path_stem(record.path()).to_owned(),
                target: target.to_owned(),
            });
        }
    }
}

/// The undeclared-conflict heuristic: a live Decision citing another live
/// Decision with no declared relationship in either envelope, both valid — an
/// invalid record is out of every derived query, half a pair included.
/// Ranked by the older member's `created`, oldest first. High precision by
/// construction — a typed id in prose is a deliberate reference.
fn undeclared_pairs(valid: &[&Record], resolvable: &Resolver<'_>) -> Vec<DebtSignal> {
    let mut found: Vec<(String, DebtSignal)> = Vec::new();
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
            let pair = DebtSignal::UndeclaredPair {
                first: Cited::of(first),
                second: Cited::of(second),
            };
            if !found.iter().any(|(_, existing)| *existing == pair) {
                let older_created = [first, second]
                    .into_iter()
                    .filter_map(|member| member.file().field("created"))
                    .min()
                    .unwrap_or_default()
                    .to_owned();
                found.push((older_created, pair));
            }
        }
    }
    found.sort_by(|(left_key, left), (right_key, right)| {
        (left_key, pair_ids(left)).cmp(&(right_key, pair_ids(right)))
    });
    found.into_iter().map(|(_, pair)| pair).collect()
}

fn pair_ids(pair: &DebtSignal) -> (String, String) {
    match pair {
        DebtSignal::UndeclaredPair { first, second } => (first.id.clone(), second.id.clone()),
        _ => (String::new(), String::new()),
    }
}

/// Any envelope edge counts as declared: the pair heuristic hunts only
/// relationships that exist nowhere but in prose.
fn declares_edge(record: &Record, target: &str) -> bool {
    let names_target = |key| record.file().field_values(key).any(|value| value == target);
    ["supersedes", "superseded-by", "from"]
        .into_iter()
        .any(names_target)
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
