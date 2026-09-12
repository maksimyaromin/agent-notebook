//! Several anb processes over one notebook, driven as processes: only a
//! real kernel lock can be observed, and only a real fork can lose a write.

use std::fs::{self, File};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;

/// One command against the notebook at `root`, run to its end.
fn anb(root: &Path, line: &[&str]) -> std::process::Output {
    spawn(root, line)
        .wait_with_output()
        .expect("the binary runs")
}

/// `HOME` is pointed at the notebook's own root: a Status reads the user's
/// notebook behind the project's, and the developer's own must not decide
/// what a case proves. The run stands beside the notebook rather than in
/// the crate, so a verb that emits an artifact beside its caller leaves it
/// in the case's own directory.
fn spawn(root: &Path, line: &[&str]) -> Child {
    Command::new(env!("CARGO_BIN_EXE_anb"))
        .args(["--notebook", root.to_str().unwrap(), "--json"])
        .args(line)
        .current_dir(root.parent().unwrap_or(root))
        .env("HOME", root)
        .env_remove("ANB_NOTEBOOK")
        .env_remove("ANB_SESSION")
        .env_remove("CODEX_THREAD_ID")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary runs")
}

fn ended(writer: Child) -> String {
    let ended = writer.wait_with_output().unwrap();
    assert!(
        ended.status.success(),
        "a writer that waited its turn still succeeds: {}",
        String::from_utf8_lossy(&ended.stderr)
    );
    String::from_utf8(ended.stdout).expect("the reply is UTF-8")
}

/// A notebook holding one Task, ready to be written to from several
/// processes at once.
fn notebook_with_a_task(dir: &TempDir) -> std::path::PathBuf {
    let root = dir.path().join("nb");
    let added = anb(
        &root,
        &["add", "task", "Contended", "--id", "task.contended"],
    );
    assert!(added.status.success(), "the fixture's Task is created");
    root
}

#[test]
fn twenty_concurrent_comments_all_land() {
    let dir = TempDir::new().unwrap();
    let root = notebook_with_a_task(&dir);

    let writers: Vec<Child> = (0..20)
        .map(|entry| {
            spawn(
                &root,
                &["comment", "task.contended", &format!("note-{entry}")],
            )
        })
        .collect();
    for writer in writers {
        ended(writer);
    }

    let record = fs::read_to_string(root.join("tasks/task.contended.md")).unwrap();
    for entry in 0..20 {
        assert!(
            record.contains(&format!("note-{entry}")),
            "every comment that reported success is in the record:\n{record}"
        );
    }
}

#[test]
fn twenty_concurrent_adds_each_mint_their_own_record() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("nb");

    let writers: Vec<Child> = (0..20)
        .map(|_| spawn(&root, &["add", "task", "Parallel"]))
        .collect();
    let minted: std::collections::BTreeSet<String> = writers
        .into_iter()
        .map(|writer| {
            let reply = ended(writer);
            let value: serde_json::Value = serde_json::from_str(&reply).unwrap();
            value["id"].as_str().unwrap().to_owned()
        })
        .collect();

    assert_eq!(minted.len(), 20, "no id was reported to two callers");
    for id in &minted {
        assert!(
            root.join(format!("tasks/{id}.md")).is_file(),
            "and each reported id names the record it was reported for"
        );
    }
}

/// The reader's completion inside the same window is what makes the
/// writer's silence mean waiting rather than a slow machine.
#[test]
fn a_writer_waits_for_a_reader_and_a_reader_does_not() {
    let dir = TempDir::new().unwrap();
    let root = notebook_with_a_task(&dir);
    let held = File::open(root.join(".lock")).unwrap();
    held.lock_shared().unwrap();

    let mut write = spawn(&root, &["comment", "task.contended", "waited"]);
    let read = anb(&root, &["list"]);
    assert!(
        read.status.success(),
        "readers share the notebook: {}",
        String::from_utf8_lossy(&read.stderr)
    );
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        write.try_wait().unwrap().is_none(),
        "the writer cannot have run while a reader held the notebook"
    );

    drop(held);
    ended(write);
    assert!(
        fs::read_to_string(root.join("tasks/task.contended.md"))
            .unwrap()
            .contains("waited")
    );
}

/// A reader cannot inspect a writer's intermediate state.
#[test]
fn a_reader_waits_for_a_writer() {
    let dir = TempDir::new().unwrap();
    let root = notebook_with_a_task(&dir);
    let held = File::open(root.join(".lock")).unwrap();
    held.lock().unwrap();

    let mut read = spawn(&root, &["list"]);
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        read.try_wait().unwrap().is_none(),
        "the reader cannot have run while a writer held the notebook"
    );

    drop(held);
    assert!(ended(read).contains("task.contended"));
}

/// A notebook comes into being on its first record, not on the first
/// command typed at it: a verb that can only be refused leaves the
/// directory it was pointed at exactly as it found it — no root, no lock
/// file — and the record is what brings one about.
#[test]
fn a_refused_first_command_brings_no_notebook_into_being() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("nb");
    for line in [
        &["start", "task.absent"][..],
        &["comment", "task.absent", "text"],
        &["archive", "task.absent"],
    ] {
        let refusal = anb(&root, line);
        assert!(!refusal.status.success(), "{line:?} was not refused");
        assert!(!root.exists(), "{line:?} brought a notebook into being");
    }
    assert!(
        anb(&root, &["add", "task", "The first record"])
            .status
            .success()
    );
    assert!(root.is_dir());
}

/// A read against a notebook that is not there answers rather than
/// refusing, and still leaves nothing behind.
#[test]
fn a_read_against_no_notebook_answers_and_creates_nothing() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("nb");
    assert!(anb(&root, &["status"]).status.success());
    assert!(!root.exists());
}

/// The classification read through its one consequence: a verb that takes
/// the notebook exclusively leaves the lock file behind, and a reader —
/// which creates nothing — never makes one.
#[test]
fn the_lock_is_taken_by_every_writing_verb_and_by_no_reading_one() {
    let writes = [
        &["add", "task", "Fresh"][..],
        &["start", "task.absent"],
        &["submit", "task.absent"],
        &["close", "task.absent", "--body", "Completed and checked."],
        &["reopen", "task.absent"],
        &["hold", "task.absent", "--reason", "waiting"],
        &["unhold", "task.absent"],
        &["block", "task.absent", "task.other"],
        &["unblock", "task.absent", "task.other"],
        &["comment", "task.absent", "text"],
        &["add", "decision", "Fresh"],
        &["add", "note", "Fresh"],
        &["add", "question", "Fresh"],
        &["close", "question.absent", "--reason", "moot"],
        &["retire", "decision.absent"],
        &["archive", "task.absent"],
        &["restore", "task.absent"],
        &["delete", "task.absent"],
        &["edit", "task.absent", "--title", "New"],
        &["import", "absent.json"],
        &["migrate"],
    ];
    let reads = [
        &["ready"][..],
        &["list"],
        &["show", "task.absent"],
        &["status"],
        &["check"],
        &["debt"],
        &["graph"],
        &["setup"],
        &["skill"],
        &["recall"],
        &["hook"],
        &["import", "absent.json", "--check"],
        &["migrate", "--check"],
    ];
    every_verb_is_classified(&writes, &reads);

    let dir = TempDir::new().unwrap();
    for (verbs, takes_the_lock) in [(&writes[..], true), (&reads[..], false)] {
        for (case, line) in verbs.iter().enumerate() {
            let root = dir.path().join(format!("{}-{case}", line[0]));
            fs::create_dir_all(&root).unwrap();
            anb(&root, line);
            assert_eq!(
                root.join(".lock").exists(),
                takes_the_lock,
                "`anb {}` and the lock",
                line.join(" ")
            );
        }
    }
}

/// The cases above are the whole command surface, asked of the parser
/// rather than of the author: a verb added and left out of both lists
/// fails here instead of going unclassified.
fn every_verb_is_classified(writes: &[&[&str]], reads: &[&[&str]]) {
    use clap::CommandFactory;
    let named: std::collections::BTreeSet<&str> =
        writes.iter().chain(reads).map(|line| line[0]).collect();
    let parsed = anb::cli::Cli::command();
    let surface: std::collections::BTreeSet<&str> = parsed
        .get_subcommands()
        .map(clap::Command::get_name)
        .collect();
    assert_eq!(named, surface);
}

/// The acceptance the lock file has to keep: it lives in the notebook and
/// is not part of it.
#[test]
fn the_lock_is_no_record_of_the_notebook() {
    let dir = TempDir::new().unwrap();
    let root = notebook_with_a_task(&dir);
    assert!(
        root.join(".lock").is_file() && root.join(".gitignore").is_file(),
        "the fixture wrote under a lock"
    );

    let checked = anb(&root, &["check"]);
    let value: serde_json::Value = serde_json::from_slice(&checked.stdout).unwrap();
    assert_eq!(
        value["count"], 0,
        "neither leaving is a file the notebook verifies"
    );
    assert_eq!(value["findings"], serde_json::json!([]));
}

mod the_notebooks_own_ignore_file {
    use super::*;

    #[test]
    fn names_the_leavings_a_run_drops_in_the_notebook() {
        let dir = TempDir::new().unwrap();
        let root = notebook_with_a_task(&dir);
        let ignored = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(ignored.contains(".lock"), "the lock is this machine's");
        assert!(
            ignored.contains("*.tmp"),
            "so is the temp file an interrupted write leaves: {ignored}"
        );
    }

    #[test]
    fn keeps_the_rules_a_notebook_already_states() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("nb");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(".gitignore"), "mine\n").unwrap();

        anb(&root, &["add", "task", "Fresh"]);
        assert_eq!(
            fs::read_to_string(root.join(".gitignore")).unwrap(),
            "mine\n"
        );
    }
}

/// A notebook that cannot hold a lock file cannot be written to blindly:
/// the failure is reported rather than run past.
#[cfg(unix)]
#[test]
fn a_root_that_refuses_the_lock_refuses_the_write() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let root = dir.path().join("nb");
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o500)).unwrap();
    if fs::write(root.join("probe"), "").is_ok() {
        // A superuser is refused nothing, so there is no refusal to observe.
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        return;
    }

    let refused = anb(&root, &["add", "task", "Fresh"]);
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();

    let payload = String::from_utf8_lossy(&refused.stderr);
    assert!(
        !refused.status.success(),
        "the write does not report success"
    );
    assert!(
        payload.contains(".lock"),
        "and names what it could not take: {payload}"
    );
}
