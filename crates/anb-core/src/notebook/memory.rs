//! Select maintained knowledge without confusing authorship with audience.

use super::{NotebookError, guard_filter, query, read_archived_record, read_live_corpus};
use crate::debt;
use crate::record::RecordType;
use crate::request::Filter;
use crate::resolve::path_stem;
use crate::{Knowledge, Memory, Storage};

/// Read active Notes and Decisions, including their evidence and bodies.
/// `text` uses the same case-insensitive search as `list`. `focus` ranks
/// knowledge within two relationships of a record before unrelated knowledge;
/// it does not hide standing rules elsewhere in the notebook. An archived
/// focus supplies relationships without admitting archived knowledge.
///
/// Invalid records are reported separately, never recalled as established facts.
/// Reads do not update usage counters, timestamps or file representations.
///
/// # Errors
/// A storage failure, or an unknown focus record.
pub fn recall(
    storage: &dyn Storage,
    text: Option<&str>,
    focus: Option<&str>,
) -> Result<Knowledge, NotebookError> {
    let filter = Filter {
        types: vec![RecordType::Decision, RecordType::Note],
        text: text.map(str::to_owned),
        ..Filter::default()
    };
    guard_filter(&filter)?;
    let corpus = read_live_corpus(storage)?;
    let resolvable = corpus.resolver();
    if let Some(id) = focus
        && !resolvable.resolves(id)
    {
        return Err(NotebookError::UnknownId { id: id.to_owned() });
    }
    let archived_focus = match focus {
        Some(id) if resolvable.read(id).is_none() => Some(
            read_archived_record(storage, id)?
                .ok_or_else(|| NotebookError::UnknownId { id: id.to_owned() })?,
        ),
        _ => None,
    };
    let mut knowledge = Knowledge::default();
    let mut all = corpus.records.iter().collect::<Vec<_>>();
    if let Some(record) = &archived_focus {
        if debt::is_excluded(record, &resolvable) {
            knowledge.invalid.push(record.path().to_owned());
        } else {
            all.push(record);
        }
    }
    let nearby = focus.map(|id| query::neighbourhood(&all, id, 2));
    let admission = query::Admission::of(&filter, None);
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
        let file = record.file();
        let id = path_stem(record.path());
        knowledge.records.push(Memory {
            id: id.to_owned(),
            path: record.path().to_owned(),
            record_type,
            kind: file.field("kind").map(str::to_owned),
            title: file.field("title").unwrap_or_default().to_owned(),
            body: file.body().to_owned(),
            by: file.field("by").map(str::to_owned),
            links: file.field_values("link").map(str::to_owned).collect(),
            related: nearby.as_ref().is_some_and(|ids| ids.contains(id)),
        });
    }
    knowledge.records.sort_by(|left, right| {
        (
            !left.related,
            left.record_type != RecordType::Decision,
            &left.id,
        )
            .cmp(&(
                !right.related,
                right.record_type != RecordType::Decision,
                &right.id,
            ))
    });
    Ok(knowledge)
}
