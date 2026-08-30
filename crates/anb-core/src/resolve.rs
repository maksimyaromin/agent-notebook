//! Resolution: what an id names, where its file sits, and whether a
//! reference points at a record the notebook still holds.
//!
//! Every rule here reads a name and a place, never a record's bytes —
//! which is what lets the archive answer from its directory listing
//! instead of from every file it holds.

use crate::grammar;
use crate::record::{Record, RecordType};
use std::collections::{BTreeMap, BTreeSet};

/// What a reference may resolve to. An id resolves when a file bearing it
/// sits at that id's canonical path — a property of the name and the place,
/// never of the bytes — so the archive answers from its listing and stays
/// closed to every query that has no use for what it says.
pub(crate) struct Resolver<'a> {
    read: BTreeMap<&'a str, &'a Record>,
    archived: &'a BTreeSet<String>,
}

impl<'a> Resolver<'a> {
    /// The resolver of one query's records and the ids its archive
    /// listing named. Only a record sitting at its own canonical path
    /// resolves — a file elsewhere carries a name and claims nothing.
    pub(crate) fn of(records: &'a [Record], archived: &'a BTreeSet<String>) -> Resolver<'a> {
        Resolver {
            read: records
                .iter()
                .filter_map(|record| Some((resolvable_id(record.path())?, record)))
                .collect(),
            archived,
        }
    }

    /// Whether the notebook still holds `id`, read or filed.
    pub(crate) fn resolves(&self, id: &str) -> bool {
        self.read.contains_key(id) || self.archived.contains(id)
    }

    /// The record behind `id`, when this query read it. A filed record the
    /// query left closed answers `None`: it resolves, and says nothing.
    pub(crate) fn read(&self, id: &str) -> Option<&'a Record> {
        self.read.get(id).copied()
    }

    /// Whether `id` sits in the archive — an answer the listing carries, so
    /// it holds whether or not the file was opened.
    pub(crate) fn archived(&self, id: &str) -> bool {
        self.archived.contains(id)
    }
}

/// The id a path resolves as, if any: the stem of a file sitting at that
/// stem's own canonical path. A file in the wrong directory carries a name
/// but claims no id, so it resolves nothing anywhere.
pub(crate) fn resolvable_id(path: &str) -> Option<&str> {
    let stem = path_stem(path);
    let home = type_of(stem)?.directory();
    let filed = path.strip_prefix("archive/").unwrap_or(path);
    let (directory, filename) = filed.rsplit_once('/')?;
    (directory == home && is_record_file(filename)).then_some(stem)
}

/// The ids among `targets` the archive holds, each once, in the order the
/// records name them.
pub(crate) fn archived_among<'a>(
    targets: impl Iterator<Item = &'a str>,
    archived: &Resolver<'_>,
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    targets
        .filter(|target| archived.archived(target))
        .filter(|target| seen.insert(*target))
        .map(str::to_owned)
        .collect()
}

pub(crate) fn archive_of(directory: &str) -> String {
    format!("archive/{directory}")
}

/// The paths an id could be held at, its live home before its archived
/// one. A name the id grammar rejects yields none — it can hold no record,
/// so a caller asking where it lives is answered "nowhere" without a
/// probe.
pub(crate) fn canonical_paths(id: &str) -> impl Iterator<Item = String> + '_ {
    type_of(id).into_iter().flat_map(move |record_type| {
        [false, true].map(move |archived| record_path(id, record_type, archived))
    })
}

/// The type an id names: the word before its dot. A malformed id names no
/// type, so nothing derived from one — a path, a directory — is ever built
/// from a name the grammar rejects.
pub(crate) fn type_of(id: &str) -> Option<RecordType> {
    if grammar::id_error(id).is_some() {
        return None;
    }
    let (word, _) = id.split_once('.')?;
    RecordType::from_word(word)
}

pub(crate) fn record_path(id: &str, record_type: RecordType, archived: bool) -> String {
    let dir = record_type.directory();
    if archived {
        format!("archive/{dir}/{id}.md")
    } else {
        format!("{dir}/{id}.md")
    }
}

/// The format is byte-exact: `.MD` is not a record file.
#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "the lint's fix folds case, and this format does not: `.MD` names no record"
)]
pub(crate) fn is_record_file(path: &str) -> bool {
    path.ends_with(".md")
}

pub(crate) fn is_archived(path: &str) -> bool {
    path.starts_with("archive/")
}

/// The id a canonical record path carries: the filename minus `.md`. The
/// path↔id rule has this one home; a host never re-derives it.
#[must_use]
pub fn path_stem(path: &str) -> &str {
    let filename = path.rsplit_once('/').map_or(path, |(_, name)| name);
    filename.strip_suffix(".md").unwrap_or(filename)
}
