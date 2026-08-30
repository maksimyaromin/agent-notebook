//! The notebook at the Storage seam: strings in, exact strings and
//! returned models out. Write-time invariants and the queries derived over
//! them are specified together, because a write is only ever observable
//! through what a later read answers. Expected file bytes derive from the
//! format's canonical form, never from running the code.

mod archive;
mod budget;
mod check;
mod creation;
mod dependencies;
mod edit;
mod lifecycle;
mod queries;
mod status;

use anb_core::{
    Blocker, Budget, CitedProof, DebtSignal, Draft, Edit, FindingCode, Link, MemoryStorage,
    Notebook, NotebookError, Proof, RecordType, Storage, StorageError, Transitioned,
};

const TODAY: &str = "2026-08-27";

/// A Status asked with nothing to settle against the world outside the
/// notebook: the host finds every cited proof still there.
fn no_lost_proofs(_cited: &[CitedProof]) -> Vec<CitedProof> {
    Vec::new()
}

fn task_file(state: &str, extra_lines: &[&str]) -> String {
    record_file("task.demo", "task", state, extra_lines, "")
}

fn record_file(id: &str, type_word: &str, state: &str, extra_lines: &[&str], body: &str) -> String {
    let mut text =
        format!("---\nid: {id}\ntype: {type_word}\nstate: {state}\ntitle: A demo record\n");
    for line in extra_lines {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str("created: 2026-08-24\nupdated: 2026-08-25\n---\n");
    text.push_str(body);
    text
}

fn storage_with(files: &[(&str, &str)]) -> MemoryStorage {
    MemoryStorage::from_files(files.iter().map(|(path, text)| (*path, *text)))
}

fn moved(id: &str, from: &'static str, to: &'static str) -> Transitioned {
    Transitioned {
        id: id.to_owned(),
        from,
        to,
        already: false,
    }
}
