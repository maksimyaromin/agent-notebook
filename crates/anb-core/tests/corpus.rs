//! The corpus from the format spec's §8: every almost-valid file yields its
//! named findings and keeps its bytes; every valid file — the body-sovereignty
//! cases above all — round-trips byte-exact.
//!
//! Case layout under `testdata/corpus/{valid,invalid}/`:
//! - `<case>.md` — the input. A case nested under a type directory
//!   (`invalid/tasks/task.x.md`) runs the full record pass — placement and
//!   the record model's semantic findings — at that notebook-relative path;
//!   a flat case exercises the grammar alone.
//! - `<case>.findings` — expected findings, one `<line> <code>` per line
//!   (`-` for findings without a line). Absent means none expected; an
//!   invalid case must have one.
//! - `<case>.normalized` — expected canonical form, where one is asserted.

use anb_core::{Finding, Record, RecordFile, Severity};
use std::fs;
use std::path::{Path, PathBuf};

fn corpus_dir(kind: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/corpus")
        .join(kind)
}

fn md_files(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("corpus directory is readable") {
            let path = entry.expect("corpus entry is readable").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|extension| extension == "md") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "no corpus cases under {}",
        root.display()
    );
    files
}

struct Case {
    rel: String,
    input: String,
    file: RecordFile,
    /// Parse findings; a case nested under a type directory carries the
    /// full record pass at that path instead.
    findings: Vec<Finding>,
}

fn load(path: &Path, root: &Path) -> Case {
    let rel = path
        .strip_prefix(root)
        .expect("case sits under its corpus root")
        .to_string_lossy()
        .into_owned();
    let input = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let file = RecordFile::parse(&input);
    let findings = if rel.contains('/') {
        Record::parse(&rel, &input).findings().to_vec()
    } else {
        file.findings().to_vec()
    };
    Case {
        rel,
        input,
        file,
        findings,
    }
}

/// `(line, code)` pairs sorted for order-insensitive comparison.
fn as_tuples(findings: &[Finding]) -> Vec<(Option<usize>, String)> {
    let mut tuples: Vec<_> = findings
        .iter()
        .map(|finding| (finding.line, finding.code.to_string()))
        .collect();
    tuples.sort();
    tuples
}

fn expected_tuples(sidecar: &Path) -> Option<Vec<(Option<usize>, String)>> {
    let text = fs::read_to_string(sidecar).ok()?;
    let mut tuples: Vec<_> = text
        .lines()
        .map(|line| {
            let (line_number, code) = line
                .split_once(' ')
                .unwrap_or_else(|| panic!("bad findings line `{line}` in {}", sidecar.display()));
            (line_number.parse::<usize>().ok(), code.to_owned())
        })
        .collect();
    tuples.sort();
    Some(tuples)
}

#[test]
fn every_valid_case_is_accepted_and_round_trips_byte_exact() {
    let root = corpus_dir("valid");
    for path in md_files(&root) {
        let case = load(&path, &root);
        let rejected = case
            .findings
            .iter()
            .any(|finding| finding.code.severity() == Severity::Error);
        assert!(
            !rejected,
            "{}: expected acceptance, got {:?}",
            case.rel, case.findings
        );
        assert_eq!(
            case.file.render(),
            case.input,
            "{}: render(parse(x)) != x",
            case.rel
        );

        let expected = expected_tuples(&path.with_extension("findings")).unwrap_or_default();
        assert_eq!(
            as_tuples(&case.findings),
            expected,
            "{}: findings mismatch",
            case.rel
        );

        let normalized = case.file.normalize();
        if let Ok(canonical) = fs::read_to_string(path.with_extension("normalized")) {
            assert_eq!(
                normalized, canonical,
                "{}: canonical form mismatch",
                case.rel
            );
        }
        assert_eq!(
            RecordFile::parse(&normalized).normalize(),
            normalized,
            "{}: normalize is not idempotent",
            case.rel
        );
    }
}

#[test]
fn every_invalid_case_is_rejected_with_its_named_findings_and_bytes_untouched() {
    let root = corpus_dir("invalid");
    for path in md_files(&root) {
        let case = load(&path, &root);
        let expected = expected_tuples(&path.with_extension("findings"))
            .unwrap_or_else(|| panic!("{}: invalid case without a .findings sidecar", case.rel));
        assert_eq!(
            as_tuples(&case.findings),
            expected,
            "{}: findings mismatch",
            case.rel
        );

        let rejected = case
            .findings
            .iter()
            .any(|finding| finding.code.severity() == Severity::Error);
        assert!(
            rejected,
            "{}: an invalid case must carry an error finding",
            case.rel
        );
        assert_eq!(
            case.file.render(),
            case.input,
            "{}: an invalid file's bytes must stay untouched",
            case.rel
        );
        // Placement findings live outside the file, so a file invalid only by
        // its path still normalizes; in-file errors freeze the bytes.
        if case.file.has_errors() {
            assert_eq!(
                case.file.normalize(),
                case.input,
                "{}: normalize must never rewrite an invalid file",
                case.rel
            );
        }
    }
}
