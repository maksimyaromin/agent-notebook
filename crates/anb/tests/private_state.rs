//! Local execution metadata must stay private even with custom repository rules.

use anb::fs_storage::FsStorage;
use anb::session::{self, Start};
use anb_core::{NotebookError, Storage};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn git(root: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .unwrap()
        .success()
}

#[test]
fn private_directories_ignore_their_contents_without_replacing_project_rules() {
    let project = TempDir::new().unwrap();
    assert!(git(project.path(), &["init", "--quiet"]));
    fs::write(project.path().join(".gitignore"), "scratch/\n").unwrap();
    let mut storage = FsStorage::new(project.path().to_owned());
    for path in [
        ".sessions.tmp/session.json",
        ".migrations.tmp/00000001.json",
    ] {
        storage.write(path, "private metadata").unwrap();
        assert!(
            git(project.path(), &["check-ignore", "--quiet", path]),
            "{path}"
        );
    }
    assert_eq!(
        fs::read_to_string(project.path().join(".gitignore")).unwrap(),
        "scratch/\n"
    );
}

#[test]
fn migration_journals_stay_ignored_with_custom_project_rules() {
    let project = TempDir::new().unwrap();
    assert!(git(project.path(), &["init", "--quiet"]));
    fs::write(project.path().join(".gitignore"), "scratch/\n").unwrap();
    let mut storage = FsStorage::new(project.path().to_owned());
    storage.write("notes/note.history.md", "---\nid: note.history\ntype: note\nstate: active\ntitle: History: original evidence\ncreated: 2018-09-10\nupdated: 2019-01-02\n---\nSource body.\r\nNo final newline").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_anb"))
        .args([
            "--notebook",
            project.path().to_str().unwrap(),
            "migrate",
            "--json",
        ])
        .env("HOME", project.path())
        .env_remove("ANB_NOTEBOOK")
        .env_remove("ANB_SESSION")
        .env_remove("CODEX_THREAD_ID")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reply: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let journal = reply["backup"]
        .as_str()
        .expect("normalization saved original bytes");
    assert!(git(project.path(), &["check-ignore", "--quiet", journal]));
    assert_eq!(
        fs::read_to_string(project.path().join(".gitignore")).unwrap(),
        "scratch/\n"
    );
}

#[test]
fn an_incompatible_private_ignore_file_refuses_metadata_without_rewriting_user_rules() {
    for directory in [".sessions.tmp", ".migrations.tmp"] {
        for rules in ["!*.json\n", " *\n", "*\n!*.json\n", ""] {
            let project = TempDir::new().unwrap();
            fs::create_dir(project.path().join(directory)).unwrap();
            let ignore = project.path().join(directory).join(".gitignore");
            fs::write(&ignore, rules).unwrap();
            let path = format!("{directory}/state.json");
            let mut storage = FsStorage::new(project.path().to_owned());
            assert!(storage.write(&path, "private metadata").is_err());
            assert!(!project.path().join(path).exists());
            assert_eq!(fs::read_to_string(ignore).unwrap(), rules);
        }
    }
}

#[cfg(unix)]
#[test]
fn a_symlinked_ignore_file_cannot_authorize_private_metadata_writes() {
    let project = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let target = outside.path().join("rules");
    fs::write(&target, "*\n").unwrap();
    fs::create_dir(project.path().join(".sessions.tmp")).unwrap();
    std::os::unix::fs::symlink(&target, project.path().join(".sessions.tmp/.gitignore")).unwrap();
    let mut storage = FsStorage::new(project.path().to_owned());
    assert!(
        storage
            .write(".sessions.tmp/state.json", "private metadata")
            .is_err()
    );
    assert!(!project.path().join(".sessions.tmp/state.json").exists());
    assert_eq!(fs::read_to_string(target).unwrap(), "*\n");
}

#[cfg(unix)]
#[test]
fn a_symlinked_session_file_is_an_unreadable_claim_not_a_new_session() {
    let project = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let mut storage = FsStorage::new(project.path().to_owned());
    storage.write("tasks/task.alpha.md", "---\nid: task.alpha\ntype: task\nstate: open\ntitle: Alpha\ncreated: 2026-09-12\nupdated: 2026-09-12\n---\n").unwrap();
    session::start(
        &mut storage,
        Some("editor"),
        Some("Ada"),
        &Start {
            id: Some("task.alpha".to_owned()),
            ..Start::default()
        },
        "2026-09-12",
    )
    .unwrap();
    let state = storage
        .list(session::DIRECTORY)
        .unwrap()
        .into_iter()
        .find(|path| {
            Path::new(path)
                .extension()
                .is_some_and(|extension| extension == "json")
        })
        .unwrap();
    let target = outside.path().join("state.json");
    fs::write(&target, storage.read(&state).unwrap()).unwrap();
    fs::remove_file(project.path().join(&state)).unwrap();
    std::os::unix::fs::symlink(&target, project.path().join(&state)).unwrap();
    assert!(matches!(
        session::focus(&storage, Some("editor"), Some("Ada")),
        Err(NotebookError::SessionRecovery { .. })
    ));
    assert!(matches!(
        session::start(
            &mut storage,
            Some("other"),
            Some("Ada"),
            &Start {
                id: Some("task.alpha".to_owned()),
                ..Start::default()
            },
            "2026-09-12"
        ),
        Err(NotebookError::SessionRecovery { .. })
    ));
}
