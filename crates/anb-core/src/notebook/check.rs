//! Check: the verification that reads the whole notebook and names what
//! it finds, where, and what erases it — the cross-record rules no single
//! record can carry, beside each record's own findings.

use super::{Notebook, NotebookError, user_scope};
use crate::config::{CONFIG_PATH, Config};
use crate::encode;
use crate::finding::{Finding, FindingCode, Severity};
use crate::grammar;
use crate::graph;
use crate::record::{
    REF_KEYS, Record, RecordType, dangling_finding, linked_record, not_utf8_finding,
};
use crate::reply::{FileFinding, Repair};
use crate::request::CLEARABLE;
use crate::resolve::{Resolver, canonical_paths, is_archived, path_stem};
use crate::storage::StorageError;
use std::collections::{BTreeMap, BTreeSet};

impl Notebook<'_> {
    /// Verify the whole notebook: every record's own findings plus the
    /// cross-record rules — duplicate ids, dangling references, supersession
    /// pairs — each carrying the move that erases it where
    /// the notebook has one. Errors first, then by file and line.
    ///
    /// # Errors
    /// A storage failure.
    pub fn check(&self) -> Result<Vec<FileFinding>, NotebookError> {
        let corpus = self.whole_corpus()?;
        let records = &corpus.records;
        let resolvable = corpus.resolver();
        let user_corpus = user_scope(self.user);
        let behind = user_corpus.resolver();
        // Live before archived, so a finding on a name two files claim
        // points at the one a reader can still edit.
        let mut by_stem: BTreeMap<&str, &Record> = BTreeMap::new();
        for record in records {
            by_stem.entry(path_stem(record.path())).or_insert(record);
        }

        let mut located = Vec::new();
        for record in records {
            for finding in record.findings() {
                located.push(FileFinding::on(record.path(), finding.clone()));
            }
            check_refs(record, &resolvable, &behind, &mut located);
            check_supersession_pair(record, &by_stem, &mut located);
        }
        check_duplicate_ids(records, &mut located);
        check_dep_cycles(records, &by_stem, &mut located);
        check_origin_cycles(records, &by_stem, &mut located);
        self.check_config(&mut located)?;
        name_repairs(&mut located, records);

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
                out.push(FileFinding::on(CONFIG_PATH, not_utf8_finding()));
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        for finding in Config::parse(&text).findings() {
            out.push(FileFinding::on(CONFIG_PATH, finding.clone()));
        }
        Ok(())
    }
}

/// Name the move that erases each finding, where the notebook has one.
///
/// A repair is only ever named on a file a verb can reach: every
/// correcting verb resolves an id to the one live path its type dictates,
/// and the archived path is reached only to be moved back or deleted — so
/// a record in the wrong directory or under a filename that is no id has
/// no move at all, whatever is wrong inside it.
fn name_repairs(located: &mut [FileFinding], records: &[Record]) {
    let by_path: BTreeMap<&str, &Record> = records
        .iter()
        .filter(|record| reachable_by_id(record.path()))
        .map(|record| (record.path(), record))
        .collect();
    for finding in located {
        finding.repair = by_path
            .get(finding.path.as_str())
            .and_then(|record| repair_of(&finding.finding, record));
    }
}

/// Whether a verb naming this file's stem would arrive at this very file.
fn reachable_by_id(path: &str) -> bool {
    canonical_paths(path_stem(path)).any(|canonical| canonical == path)
}

/// A record on the wrong side of the residence axis is moved home by
/// `archive` or `restore`; every other repair is the verb that erases the
/// line the finding sits on. On an archived file the only move worth
/// naming is `restore`, and it erases only the residence disagreement — a
/// finding deeper inside becomes repairable when the record is back, so
/// naming its eraser here would name a command that refuses to run.
fn repair_of(finding: &Finding, record: &Record) -> Option<Repair> {
    if is_archived(record.path()) {
        return (finding.code == FindingCode::ArchivedLiveRecord).then_some(Repair::Restore);
    }
    match finding.code {
        FindingCode::UnarchivedSettledRecord => Some(Repair::Archive),
        _ => eraser_of(record, finding.line?),
    }
}

/// The verb that erases `line`: a dependency edge and a hold are erased by
/// their own verbs, an optional field the record can lose by `edit`. A line
/// no verb writes has none.
fn eraser_of(record: &Record, line: usize) -> Option<Repair> {
    let file = record.file();
    let sits_on = |key: &str| file.field_entries(key).any(|(_, at)| at == Some(line));
    // `unblock` and `unhold` refuse a record that is not a Task, so a line
    // a record should not carry at all is not theirs to erase.
    if record.record_type() == Some(RecordType::Task) {
        if let Some((target, _)) = file
            .field_entries("blocked-by")
            .find(|(_, at)| *at == Some(line))
        {
            return Some(Repair::Unblock(target.to_owned()));
        }
        if sits_on("hold") || sits_on("hold-until") {
            return Some(Repair::Unhold);
        }
    }
    CLEARABLE
        .into_iter()
        .find(|key| sits_on(key))
        .map(Repair::Clear)
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

/// The envelope edges are this notebook's own structure — an origin the
/// clocks key on, a supersession the tool writes both halves of, a block the
/// queue follows — so each must be answered here. A `link` points outward by
/// nature, at a pull request, a commit, a path or a record, so a record the
/// user's notebook holds is within its reach.
fn check_refs(
    record: &Record,
    resolvable: &Resolver<'_>,
    behind: &Resolver<'_>,
    out: &mut Vec<FileFinding>,
) {
    let mut dangling = |key, target: &str, line, reaches_the_user: bool| {
        if grammar::id_error(target).is_some()
            || resolvable.resolves(target)
            || (reaches_the_user && behind.resolves(target))
        {
            return;
        }
        out.push(FileFinding::on(
            record.path(),
            dangling_finding(key, target, line),
        ));
    };
    for key in REF_KEYS {
        for (target, line) in record.file().field_entries(key) {
            dangling(key, target, line, false);
        }
    }
    // A proof carried as a Note is a reference like any other: the whole
    // point of ingesting the report was that a reader could reach it.
    for (link, line) in record.file().field_entries("link") {
        if let Some(target) = linked_record(link) {
            dangling("link", target, line, true);
        }
    }
}

/// The supersession pair, verified from both ends and reported against
/// both files. Three independent rules, each of which can fire alone.
fn check_supersession_pair(
    record: &Record,
    by_stem: &BTreeMap<&str, &Record>,
    out: &mut Vec<FileFinding>,
) {
    check_claim_answered(record, by_stem, out);
    check_superseder_claims_back(record, by_stem, out);
    check_replaced_record_is_settled(record, out);
}

/// A record claiming `supersedes` whose victim does not point back. Both
/// halves of a supersession are written by one command, so a claim
/// standing alone is a hand edit or an interrupted one.
fn check_claim_answered(
    record: &Record,
    by_stem: &BTreeMap<&str, &Record>,
    out: &mut Vec<FileFinding>,
) {
    let stem = path_stem(record.path());
    let Some((target, line)) = record.file().field_entry("supersedes") else {
        return;
    };
    let Some(victim) = by_stem.get(target) else {
        return;
    };
    if victim.superseded_by() == Some(stem) {
        return;
    }
    out.push(FileFinding::on(
        record.path(),
        Finding::located(
            line,
            FindingCode::BrokenSupersession,
            format!("supersedes: `{target}` does not point back with `superseded-by`"),
        ),
    ));
    out.push(FileFinding::on(
        victim.path(),
        Finding::for_file(
            FindingCode::BrokenSupersession,
            format!("`{stem}` claims to supersede this record, which does not point back"),
        ),
    ));
}

/// A record pointing at a superseder that does not claim it — the same
/// broken pair seen from the victim's end, which is the end a reader
/// reaches first when the claimant is the file that was hand-edited.
fn check_superseder_claims_back(
    record: &Record,
    by_stem: &BTreeMap<&str, &Record>,
    out: &mut Vec<FileFinding>,
) {
    let stem = path_stem(record.path());
    let Some((superseder, line)) = record.file().field_entry("superseded-by") else {
        return;
    };
    let Some(claimant) = by_stem.get(superseder) else {
        return;
    };
    if claimant.supersedes() == Some(stem) {
        return;
    }
    out.push(FileFinding::on(
        record.path(),
        Finding::located(
            line,
            FindingCode::BrokenSupersession,
            format!("superseded-by: `{superseder}` does not claim `supersedes: {stem}`"),
        ),
    ));
    out.push(FileFinding::on(
        claimant.path(),
        Finding::for_file(
            FindingCode::BrokenSupersession,
            format!("`{stem}` names this record as its superseder, which does not claim it"),
        ),
    ));
}

/// A replaced record that still reads live: the supersession flip and the
/// back-pointer are written together, so a live state beside one means the
/// record is being answered for by a replacement it has not stepped aside
/// for. Judged from the record alone, whether or not the superseder exists.
fn check_replaced_record_is_settled(record: &Record, out: &mut Vec<FileFinding>) {
    let Some(record_type) = record.record_type() else {
        return;
    };
    if record.file().field_entry("superseded-by").is_none() {
        return;
    }
    let Some((state, line)) = record.file().field_entry("state") else {
        return;
    };
    if !record_type.live_states().contains(&state) {
        return;
    }
    out.push(FileFinding::on(
        record.path(),
        Finding::located(
            line,
            FindingCode::BrokenSupersession,
            format!(
                "state: `{state}` on a record marked `superseded-by` — a replaced record must not read live"
            ),
        ),
    ));
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
            out.push(FileFinding::on(
                record.path(),
                Finding::for_file(
                    FindingCode::DuplicateId,
                    format!("id `{id}` is also claimed by {}", others.join(", ")),
                ),
            ));
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
        FindingCode::BlockCycle,
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
    let mut named: BTreeSet<(String, String)> = BTreeSet::new();
    for cycle in cycles {
        let mut walk = None;
        for (position, member) in cycle.iter().enumerate() {
            let next = &cycle[(position + 1) % cycle.len()];
            let Some(record) = by_stem.get(member.as_str()) else {
                continue;
            };
            if !named.insert((record.path().to_owned(), next.clone())) {
                continue;
            }
            let line = record
                .file()
                .field_entries(field)
                .find(|(target, _)| target == next)
                .and_then(|(_, line)| line);
            let walk = walk.get_or_insert_with(|| encode::id_chain(&closed_walk(&cycle)));
            out.push(FileFinding::on(
                record.path(),
                Finding::located(
                    line,
                    code,
                    format!("{field}: `{next}` closes the cycle {walk}"),
                ),
            ));
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
/// `Record::blocked_by` states: a malformed target is a finding, never an
/// edge. A record has at most one Origin, so a lineage is a path — one
/// that meets itself is the corruption `check` names.
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
