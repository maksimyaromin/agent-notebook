//! Import and migration through the installed command surface and filesystem.

use anb::fs_storage::FsStorage;
use anb_core::{RecordFile, Storage};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn run(project: &Path, arguments: &[&str]) -> Output {
    command(project).args(arguments).output().unwrap()
}

fn command(project: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_anb"));
    command
        .current_dir(project)
        .env("HOME", project)
        .env_remove("ANB_NOTEBOOK")
        .env_remove("ANB_BY");
    command
}

fn note() -> &'static str {
    "---\nid: note.history\ntype: note\nstate: active\ntitle: History: original evidence\ncreated: 2018-09-10\nupdated: 2019-01-02\n---\nSource body.\r\nNo final newline"
}

fn successful_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn import_preview_creates_no_notebook_and_import_reads_a_directory_relative_to_the_call() {
    let project = TempDir::new().unwrap();
    let mut source = FsStorage::new(project.path().join("source"));
    source.write("notes/note.history.md", note()).unwrap();
    let preview = successful_json(&run(
        project.path(),
        &["import", "source", "--check", "--json"],
    ));
    assert_eq!(preview["check"], true);
    assert_eq!(
        preview["paths"],
        serde_json::json!(["notes/note.history.md"])
    );
    assert!(!project.path().join(".agent-notebook").exists());
    let imported = successful_json(&run(project.path(), &["import", "source", "--json"]));
    assert_eq!(imported["check"], false);
    let text =
        fs::read_to_string(project.path().join(".agent-notebook/notes/note.history.md")).unwrap();
    assert_eq!(
        RecordFile::parse(&text).field("created"),
        Some("2018-09-10")
    );
    assert_eq!(
        RecordFile::parse(&text).body(),
        "Source body.\r\nNo final newline"
    );
    assert_eq!(source.read("notes/note.history.md").unwrap(), note());
}

#[test]
fn migration_reports_its_recoverable_original_without_changing_record_history() {
    let project = TempDir::new().unwrap();
    let mut storage = FsStorage::new(project.path().join(".agent-notebook"));
    storage.write("notes/note.history.md", note()).unwrap();
    let preview = successful_json(&run(project.path(), &["migrate", "--check", "--json"]));
    assert_eq!(
        preview["paths"],
        serde_json::json!(["notes/note.history.md"])
    );
    assert!(preview["backup"].is_null());
    assert!(
        !project
            .path()
            .join(".agent-notebook/.migrations.tmp")
            .exists()
    );
    let result = successful_json(&run(project.path(), &["migrate", "--json"]));
    let journal: Value =
        serde_json::from_str(&storage.read(result["backup"].as_str().unwrap()).unwrap()).unwrap();
    assert_eq!(journal["notes/note.history.md"], note());
    let text = storage.read("notes/note.history.md").unwrap();
    assert_eq!(
        RecordFile::parse(&text).field("updated"),
        Some("2019-01-02")
    );
    assert_eq!(
        RecordFile::parse(&text).body(),
        RecordFile::parse(note()).body()
    );
}

#[test]
fn global_import_refuses_work_records_before_creating_any_file() {
    let project = TempDir::new().unwrap();
    let mut source = FsStorage::new(project.path().join("source"));
    source.write("notes/note.history.md", note()).unwrap();
    source
        .write(
            "tasks/task.work.md",
            "---\nid: task.work\ntype: task\nstate: open\ntitle: Work\ncreated: 2026-09-12\n---\n",
        )
        .unwrap();
    let output = run(project.path(), &["import", "source", "--global", "--json"]);
    assert!(!output.status.success());
    assert!(!project.path().join(".agent-notebook/notes").exists());
    assert!(!project.path().join(".agent-notebook/tasks").exists());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("personal notebooks hold Notes and Decisions")
    );
}

#[test]
fn an_invalid_import_does_not_suggest_reading_a_record_that_was_never_imported() {
    let project = TempDir::new().unwrap();
    let mut source = FsStorage::new(project.path().join("source"));
    let invalid = note().replace("created:", "link: source note.missing\ncreated:");
    source.write("notes/note.history.md", &invalid).unwrap();
    let output = run(project.path(), &["import", "source", "--json"]);
    assert!(!output.status.success());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["try"], serde_json::json!(["anb import --help"]));
    assert!(
        !project
            .path()
            .join(".agent-notebook/notes/note.history.md")
            .exists()
    );
}

#[cfg(unix)]
#[test]
fn migration_refuses_a_symlinked_journal_directory_before_changing_records() {
    let project = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let mut storage = FsStorage::new(project.path().join(".agent-notebook"));
    storage.write("notes/note.history.md", note()).unwrap();
    std::os::unix::fs::symlink(
        outside.path(),
        project.path().join(".agent-notebook/.migrations.tmp"),
    )
    .unwrap();
    let output = run(project.path(), &["migrate", "--json"]);
    assert!(!output.status.success());
    assert_eq!(storage.read("notes/note.history.md").unwrap(), note());
    assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
}

#[test]
fn a_killed_migration_leaves_a_readable_journal_and_resumes_on_the_next_command() {
    let project = TempDir::new().unwrap();
    let root = project.path().join(".agent-notebook");
    fs::create_dir_all(root.join("notes")).unwrap();
    let originals: std::collections::BTreeMap<String, String> = (0..200)
        .map(|index| {
            (
                format!("notes/note.item-{index}.md"),
                note().replace("note.history", &format!("note.item-{index}")),
            )
        })
        .collect();
    for (path, text) in &originals {
        fs::write(root.join(path), text).unwrap();
    }
    let mut child = command(project.path())
        .args(["migrate", "--json"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let journal = root.join(".migrations.tmp/00000001.json");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !journal.exists() && Instant::now() < deadline {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let killed = child.kill();
    let status = child.wait().unwrap();
    assert!(journal.exists(), "the command did not publish its journal");
    assert!(
        killed.is_ok() && !status.success(),
        "the test must interrupt a running migration"
    );
    let saved: std::collections::BTreeMap<String, String> =
        serde_json::from_str(&fs::read_to_string(&journal).unwrap()).unwrap();
    assert_eq!(saved.len(), 200);
    assert_eq!(saved, originals);
    successful_json(&run(project.path(), &["migrate", "--json"]));
    for (path, text) in &originals {
        assert_eq!(
            fs::read_to_string(root.join(path)).unwrap(),
            RecordFile::parse(text).normalize()
        );
    }
}
