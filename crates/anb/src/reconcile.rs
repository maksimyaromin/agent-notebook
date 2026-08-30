//! Settling a notebook's proofs against the world outside it.
//!
//! A proof is the one part of a record that claims something the notebook
//! cannot see: a commit belongs to git, a report to the filesystem. Between
//! the record and the world, the world wins — so this asks, and reports
//! what is no longer there. It never repairs: which commit a lost proof
//! meant is not something anything here can know.

use crate::fs_storage::project_anchor;
use crate::git;
use anb_core::CitedProof;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The proofs the notebook at `root` cites that the world no longer holds.
///
/// A proof whose kind nothing here can settle is left alone: silence means
/// "not known to be lost", never "verified". A report path is read from the
/// project rather than from the notebook directory or the caller's working
/// directory — a path in a record outlives the shell that typed it, and the
/// project is the only base a later reader shares.
#[must_use]
pub fn lost_proofs(root: &Path, cited: &[CitedProof]) -> Vec<CitedProof> {
    let gone = absent_commits(root, cited);
    let project = project_anchor(&ask_in(root)).to_owned();
    cited
        .iter()
        .filter(|proof| match proof.kind.as_str() {
            "sha" => gone.contains(&proof.target),
            "report" => !project.join(&proof.target).exists(),
            _ => false,
        })
        .cloned()
        .collect()
}

/// The cited commits this repository does not have.
///
/// One `git cat-file --batch-check`, which answers one line per line asked,
/// in order. Anything short of an answer — no git, no repository, a
/// notebook living outside one, a git that ran out of time — reports
/// "nothing known missing" rather than inventing a divergence. git runs in
/// the notebook's own directory, since that is the repository whose history
/// these proofs are about.
fn absent_commits(root: &Path, cited: &[CitedProof]) -> BTreeSet<String> {
    let shas: Vec<&str> = cited
        .iter()
        .filter(|proof| proof.kind == "sha")
        .map(|proof| proof.target.as_str())
        .collect();
    if shas.is_empty() {
        return BTreeSet::new();
    }
    let mut query = String::new();
    for sha in &shas {
        let _ = writeln!(query, "{sha}^{{commit}}");
    }
    let Some(bytes) = git::answered(
        Command::new("git")
            .arg("-C")
            .arg(ask_in(root))
            .args(["cat-file", "--batch-check"]),
        query,
    ) else {
        return BTreeSet::new();
    };
    let Ok(answers) = String::from_utf8(bytes) else {
        return BTreeSet::new();
    };
    answers
        .lines()
        .zip(&shas)
        .filter(|(answer, _)| answer.ends_with("missing"))
        .map(|(_, sha)| (*sha).to_owned())
        .collect()
}

/// Where to stand while asking: inside the notebook, since it may sit
/// anywhere and the working directory may belong to another repository
/// entirely. A root not yet on disk falls back to its nearest existing
/// ancestor, so a first-run notebook still asks the right repository.
fn ask_in(root: &Path) -> PathBuf {
    root.ancestors()
        .find(|dir| dir.is_dir())
        .unwrap_or(Path::new("."))
        .to_owned()
}
