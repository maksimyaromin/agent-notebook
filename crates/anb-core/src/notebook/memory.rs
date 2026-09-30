//! Select maintained knowledge without confusing authorship with audience.

use super::{NotebookError, guard_filter, query, read_live_corpus};
use crate::debt;
use crate::record::{Record, RecordType};
use crate::request::Filter;
use crate::{Knowledge, Memory, Storage};

/// The Decision kinds a session opens with: standing rules, and the agreed
/// exceptions to them, since a rule read without its exception misleads.
const STANDING_KINDS: [&str; 2] = ["rule", "drift"];

/// Read the live Notes and Decisions a session needs before its work is
/// known, or the ones a phrase finds.
///
/// Without `text`, the standing Decisions: every live `rule` and `drift`.
/// Design choices and Notes are reached from the work that cites them, so
/// `other` counts them instead. With `text`, every live Note and Decision
/// the match of [`Filter::text`] admits. Either way Decisions come first,
/// rules and then drifts leading them, and ids order the rest, so an
/// unchanged notebook recalls the same way twice.
///
/// Invalid records are reported separately, never recalled as established facts.
/// Reads do not update usage counters, timestamps or file representations.
///
/// # Errors
/// A storage failure, or an empty `text`.
pub fn recall(storage: &dyn Storage, text: Option<&str>) -> Result<Knowledge, NotebookError> {
    let filter = Filter {
        types: vec![RecordType::Decision, RecordType::Note],
        text: text.map(str::to_owned),
        ..Filter::default()
    };
    guard_filter(&filter)?;
    let corpus = read_live_corpus(storage)?;
    let resolvable = corpus.resolver();
    let admission = query::Admission::of(&filter, None);
    let mut knowledge = Knowledge::default();
    let mut other = 0;
    for record in &corpus.records {
        if debt::is_excluded(record, &resolvable) {
            knowledge.invalid.push(record.path().to_owned());
            continue;
        }
        if !admission.admits(record) || !record.is_live() {
            continue;
        }
        let Some(record_type) = record.record_type() else {
            continue;
        };
        if text.is_none() && !is_standing(record) {
            other += 1;
            continue;
        }
        knowledge.records.push(Memory::of(record, record_type));
    }
    let rank = |memory: &Memory| {
        let kind = memory.kind.as_deref().unwrap_or_default();
        (
            memory.record_type != RecordType::Decision,
            STANDING_KINDS
                .iter()
                .position(|standing| *standing == kind)
                .unwrap_or(STANDING_KINDS.len()),
            memory.id.clone(),
        )
    };
    knowledge.records.sort_by_key(rank);
    knowledge.other = text.is_none().then_some(other);
    Ok(knowledge)
}

/// Whether the record binds every session whatever its work: a live
/// Decision of a standing kind.
fn is_standing(record: &Record) -> bool {
    record.record_type() == Some(RecordType::Decision)
        && record
            .file()
            .field("kind")
            .is_some_and(|kind| STANDING_KINDS.contains(&kind))
}
