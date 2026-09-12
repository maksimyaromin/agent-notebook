//! Personal practices are outside Git and follow the person, not the author of a shared record.

use anb::fs_storage::personal_root;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn output(project: &Path, home: &Path, who: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_anb"))
        .current_dir(project)
        .arg("--json")
        .args(args)
        .env("HOME", home)
        .env("ANB_BY", who)
        .env_remove("ANB_NOTEBOOK")
        .env_remove("ANB_SESSION")
        .env_remove("CODEX_THREAD_ID")
        .output()
        .unwrap()
}

fn run(project: &Path, home: &Path, who: &str, args: &[&str]) -> Value {
    let output = output(project, home, who, args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn mkdir(root: &Path, name: &str) -> std::path::PathBuf {
    let path = root.join(name);
    fs::create_dir_all(&path).unwrap();
    path
}

fn bodies(recalled: &Value) -> Vec<(&str, &str)> {
    recalled["memories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["scope"].as_str().unwrap(),
                row["body"]["head"].as_str().unwrap(),
            )
        })
        .collect()
}

#[test]
fn personal_and_global_practices_follow_their_declared_audiences() {
    let root = TempDir::new().unwrap();
    let project = mkdir(root.path(), "project");
    let other_project = mkdir(root.path(), "other-project");
    let ada = mkdir(root.path(), "ada");
    let grace = mkdir(root.path(), "grace");
    run(
        &project,
        &ada,
        "Ada",
        &[
            "add",
            "note",
            "Project rule",
            "--id",
            "note.shared",
            "--body",
            "Customer exports are tenant-scoped.",
        ],
    );
    run(
        &project,
        &ada,
        "Ada",
        &[
            "--personal",
            "add",
            "note",
            "Project practice",
            "--id",
            "note.practice",
            "--body",
            "Ask me before running this project's slow suite.",
        ],
    );
    run(
        &project,
        &ada,
        "Ada",
        &[
            "--global",
            "add",
            "note",
            "General practice",
            "--id",
            "note.practice",
            "--body",
            "Review behavior before formatting.",
        ],
    );
    let recalled = run(&project, &ada, "Ada", &["recall", "--all"]);
    let recalled = bodies(&recalled);
    assert!(
        recalled
            .iter()
            .any(|(scope, body)| *scope == "personal" && body.contains("slow suite"))
    );
    assert!(
        recalled
            .iter()
            .any(|(scope, body)| *scope == "global" && body.contains("formatting"))
    );
    assert!(
        recalled
            .iter()
            .any(|(scope, body)| *scope == "project" && body.contains("tenant-scoped"))
    );
    let colleague = run(&project, &grace, "Grace", &["recall", "--all"]);
    assert_eq!(bodies(&colleague).len(), 1);
    assert_eq!(bodies(&colleague)[0].0, "project");
    let elsewhere = run(&other_project, &ada, "Ada", &["recall", "--all"]);
    assert_eq!(bodies(&elsewhere).len(), 1);
    assert_eq!(bodies(&elsewhere)[0].0, "global");
    assert!(
        !project
            .join(".agent-notebook/notes/note.practice.md")
            .exists()
    );
    assert!(
        !other_project.join(".agent-notebook").exists(),
        "recall must not initialize an empty project"
    );
}

#[test]
fn personal_scopes_refuse_work_records_before_creating_their_store() {
    let root = TempDir::new().unwrap();
    let project = mkdir(root.path(), "project");
    let home = mkdir(root.path(), "home");
    for scope in ["--personal", "--global"] {
        for kind in ["task", "question"] {
            let refused = output(
                &project,
                &home,
                "Ada",
                &[scope, "add", kind, "Private work"],
            );
            assert!(!refused.status.success());
            let error: Value = serde_json::from_slice(&refused.stderr).unwrap();
            assert_eq!(error["error"], "invalid-argument");
        }
    }
    assert!(!home.join(".agent-notebook").exists());
}

#[test]
fn an_unreadable_personal_source_is_an_error_not_an_empty_recall() {
    let root = TempDir::new().unwrap();
    let project = mkdir(root.path(), "project");
    let home = mkdir(root.path(), "home");
    fs::write(home.join(".agent-notebook"), "Not a directory").unwrap();
    let refused = output(&project, &home, "Ada", &["recall"]);
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert!(serde_json::from_slice::<Value>(&refused.stderr).unwrap()["message"].is_string());
}

fn git(project: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(project)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn linked_worktrees_share_personal_practices_but_independent_clones_do_not() {
    let root = TempDir::new().unwrap();
    let project = mkdir(root.path(), "project");
    let home = mkdir(root.path(), "home");
    let user_notebook = home.join(".agent-notebook");
    git(&project, &["init", "--quiet"]);
    git(
        &project,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "--allow-empty",
            "-m",
            "Test fixture",
        ],
    );
    let worktree = root.path().join("worktree");
    git(
        &project,
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "parallel",
            worktree.to_str().unwrap(),
        ],
    );
    let clone = root.path().join("clone");
    git(
        root.path(),
        &[
            "clone",
            "--quiet",
            project.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    assert_eq!(
        personal_root(&project, &user_notebook).unwrap(),
        personal_root(&worktree, &user_notebook).unwrap()
    );
    assert_ne!(
        personal_root(&project, &user_notebook).unwrap(),
        personal_root(&clone, &user_notebook).unwrap()
    );
    run(
        &project,
        &home,
        "Ada",
        &[
            "--personal",
            "add",
            "note",
            "Practice",
            "--id",
            "note.practice",
            "--body",
            "Ask before the slow suite.",
        ],
    );
    let parallel = run(&worktree, &home, "Ada", &["recall", "--all"]);
    assert!(
        bodies(&parallel)
            .iter()
            .any(|(scope, body)| *scope == "personal" && body.contains("slow suite"))
    );
    assert!(bodies(&run(&clone, &home, "Ada", &["recall", "--all"])).is_empty());
    assert!(!project.join(".agent-notebook").exists());
    assert!(!worktree.join(".agent-notebook").exists());
}
