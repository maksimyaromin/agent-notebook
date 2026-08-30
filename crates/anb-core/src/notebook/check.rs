//! Check: the verification that reads the whole notebook and names every
//! file, line, and reason it finds — the cross-record rules no single
//! record can carry, beside each record's own findings.

use super::{Notebook, NotebookError};
use crate::config::{CONFIG_PATH, Config};
use crate::encode;
use crate::finding::{Finding, FindingCode, Severity};
use crate::grammar;
use crate::graph;
use crate::record::{
    REF_KEYS, Record, RecordType, dangling_finding, linked_record, not_utf8_finding,
};
use crate::reply::FileFinding;
use crate::resolve::{Resolver, path_stem};
use crate::storage::{Storage, StorageError};
use std::collections::BTreeMap;

impl<S: Storage> Notebook<'_, S> {
    /// Verify the whole notebook: every record's own findings plus the
    /// cross-record rules — duplicate ids, dangling references, supersession
    /// pairs, routing threads. Errors first, then by file and line.
    ///
    /// # Errors
    /// A storage failure.
    pub fn check(&self) -> Result<Vec<FileFinding>, NotebookError> {
        let corpus = self.whole_corpus()?;
        let records = &corpus.records;
        let resolvable = corpus.resolver();
        // Live before archived, so a finding on a name two files claim
        // points at the one a reader can still edit.
        let mut by_stem: BTreeMap<&str, &Record> = BTreeMap::new();
        for record in records {
            by_stem.entry(path_stem(record.path())).or_insert(record);
        }

        let mut located = Vec::new();
        for record in records {
            for finding in record.findings() {
                located.push(FileFinding {
                    path: record.path().to_owned(),
                    finding: finding.clone(),
                });
            }
            check_refs(record, &resolvable, &mut located);
            check_supersession_pair(record, &by_stem, &mut located);
        }
        check_duplicate_ids(records, &mut located);
        check_dep_cycles(records, &by_stem, &mut located);
        check_origin_cycles(records, &by_stem, &mut located);
        self.check_config(&mut located)?;

        located.sort_by(|left, right| finding_order(left).cmp(&finding_order(right)));
        Ok(located)
    }

    /// The config file's findings, against its own path; the same closed
    /// finding set covers records and config alike.
    fn check_config(&self, out: &mut Vec<FileFinding>) -> Result<(), NotebookError> {
        let text = match self.storage.read(CONFIG_PATH) {
            Ok(text) => text,
            Err(StorageError::NotFound { .. }) => return Ok(()),
            Err(StorageError::NotUtf8 { .. }) => {
                out.push(FileFinding {
                    path: CONFIG_PATH.to_owned(),
                    finding: not_utf8_finding(),
                });
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        for finding in Config::parse(&text).findings() {
            out.push(FileFinding {
                path: CONFIG_PATH.to_owned(),
                finding: finding.clone(),
            });
        }
        Ok(())
    }
}

/// Errors first, then file, then line (file-level findings last), then code.
/// Severity outranks the file because the reply prints a bounded head: a
/// warning the tool's own happy path produces must never push an error out
/// of a truncated report.
fn finding_order(located: &FileFinding) -> (u8, &str, usize, &'static str) {
    let severity = match located.finding.code.severity() {
        Severity::Error => 0,
        Severity::Warning => 1,
    };
    (
        severity,
        located.path.as_str(),
        located.finding.line.unwrap_or(usize::MAX),
        located.finding.code.as_str(),
    )
}

fn check_refs(record: &Record, resolvable: &Resolver<'_>, out: &mut Vec<FileFinding>) {
    let mut dangling = |key, target: &str, line| {
        if grammar::id_error(target).is_some() || resolvable.resolves(target) {
            return;
        }
        out.push(FileFinding {
            path: record.path().to_owned(),
            finding: dangling_finding(record, key, target, line),
        });
    };
    for key in REF_KEYS {
        for (target, line) in record.file().field_entries(key) {
            dangling(key, target, line);
        }
    }
    // A proof carried as a Note is a reference like any other: the whole
    // point of ingesting the report was that the notebook could reach it.
    for (link, line) in record.file().field_entries("link") {
        if let Some(target) = linked_record(link) {
            dangling("link", target, line);
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

/// Cycles among `blocked-by` edges. The tool refuses them at write, so a
/// cycle is hand-edited corruption — without this pass it would only sit
/// there emptying `ready`.
fn check_dep_cycles(
    records: &[Record],
    by_stem: &BTreeMap<&str, &Record>,
    out: &mut Vec<FileFinding>,
) {
    // A Task waiting on itself is the one cycle a single file carries, and
    // that file names it alone; naming it again here would double it.
    let across_files = graph::cycles(&task_edges(records))
        .into_iter()
        .filter(|cycle| cycle.len() > 1)
        .collect();
    cycle_findings(
        across_files,
        by_stem,
        "blocked-by",
        FindingCode::DepCycle,
        out,
    );
}

/// Cycles among `from` edges. `edit` refuses to write one, so a lineage
/// that closes on itself is hand-edited corruption. A record whose `from`
/// names itself is such a cycle with one member, and is named here rather
/// than by its own file: an Origin has no verb that erases it, so a
/// finding the mutation gate holds would freeze the record out of `edit`,
/// the only verb that can repoint it.
fn check_origin_cycles(
    records: &[Record],
    by_stem: &BTreeMap<&str, &Record>,
    out: &mut Vec<FileFinding>,
) {
    cycle_findings(
        graph::cycles(&origin_edges(records)),
        by_stem,
        "from",
        FindingCode::OriginCycle,
        out,
    );
}

/// Each cycle named on every member file, at the line carrying the edge
/// that closes it, with the whole walk for a reader deciding which edge to
/// erase.
fn cycle_findings(
    cycles: Vec<Vec<String>>,
    by_stem: &BTreeMap<&str, &Record>,
    field: &'static str,
    code: FindingCode,
    out: &mut Vec<FileFinding>,
) {
    for cycle in cycles {
        let walk = encode::id_chain(&closed_walk(&cycle));
        for (position, member) in cycle.iter().enumerate() {
            let next = &cycle[(position + 1) % cycle.len()];
            let Some(record) = by_stem.get(member.as_str()) else {
                continue;
            };
            let line = record
                .file()
                .field_entries(field)
                .find(|(target, _)| target == next)
                .and_then(|(_, line)| line);
            out.push(FileFinding {
                path: record.path().to_owned(),
                finding: Finding::located(
                    line,
                    code,
                    format!("{field}: `{next}` closes the cycle {walk}"),
                ),
            });
        }
    }
}

/// A cycle walked so its ends meet, for a reader who must see where it
/// closes.
fn closed_walk(cycle: &[String]) -> Vec<String> {
    cycle.iter().chain(cycle.first()).cloned().collect()
}

/// The `blocked-by` edges the Tasks handed in draw, keyed by file stem —
/// the name graph findings report against. Archived Tasks enter too, and
/// a closed one keeps its edges: a cycle through history is still a cycle,
/// and a closed Task can be reopened into one.
fn task_edges(records: &[Record]) -> BTreeMap<&str, Vec<&str>> {
    let mut edges = BTreeMap::new();
    for record in records {
        if record.record_type() != Some(RecordType::Task) {
            continue;
        }
        edges
            .entry(path_stem(record.path()))
            .or_insert_with(|| record.blocked_by().collect());
    }
    edges
}

/// The `from` edge every record draws, keyed by file stem, under the rule
/// [`blocked_by`] states: a malformed target is a finding, never an edge.
/// A record has at most one Origin, so a lineage is a path — one that
/// meets itself is the corruption `check` names.
fn origin_edges(records: &[Record]) -> BTreeMap<&str, Vec<&str>> {
    let mut edges = BTreeMap::new();
    for record in records {
        edges.entry(path_stem(record.path())).or_insert_with(|| {
            record
                .origin()
                .filter(|origin| grammar::id_error(origin).is_none())
                .into_iter()
                .collect()
        });
    }
    edges
}
