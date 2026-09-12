//! Follow-up commands must read the same audience that produced the reply.

use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_anb"))
        .current_dir(root)
        .args(args)
        .env("HOME", root)
        .env("ANB_BY", "Ada")
        .env_remove("ANB_NOTEBOOK")
        .env_remove("ANB_SESSION")
        .env_remove("CODEX_THREAD_ID")
        .output()
        .unwrap()
}

fn success(root: &Path, args: &[&str]) -> Value {
    let output = run(root, args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn a_named_notebook_read_and_refusal_keep_their_exact_root() {
    let fixture = TempDir::new().unwrap();
    success(
        fixture.path(),
        &[
            "--json",
            "--notebook",
            "a notebook",
            "add",
            "note",
            "A fact",
            "--id",
            "note.fact",
        ],
    );
    let read = success(
        fixture.path(),
        &["--json", "--notebook", "a notebook", "show", "note.fact"],
    );
    assert_eq!(
        read["more"],
        "anb --notebook 'a notebook' show note.fact --all"
    );
    let refused = run(
        fixture.path(),
        &["--json", "--notebook", "a notebook", "show", "note.absent"],
    );
    assert!(!refused.status.success());
    let recovery: Value = serde_json::from_slice(&refused.stderr).unwrap();
    assert_eq!(recovery["try"][0], "anb --notebook 'a notebook' list");
}

#[test]
fn private_work_refusal_does_not_suggest_repeating_the_same_invalid_scope() {
    let fixture = TempDir::new().unwrap();
    let refused = run(
        fixture.path(),
        &["--json", "--personal", "add", "task", "Wrong audience"],
    );
    assert!(!refused.status.success());
    let recovery: Value = serde_json::from_slice(&refused.stderr).unwrap();
    assert_eq!(recovery["try"][0], "anb add task \"<title>\"");
    assert!(!fixture.path().join(".agent-notebook").exists());
}

#[test]
fn status_budget_accounts_for_qualified_followup_commands() {
    let fixture = TempDir::new().unwrap();
    let args = [
        "--notebook",
        "a notebook",
        "--session",
        "a long explicit conversation",
        "status",
        "--budget",
        "400",
    ];
    let text = run(fixture.path(), &args);
    assert!(text.status.success());
    let mut json_args = vec!["--json"];
    json_args.extend(args);
    let document = success(fixture.path(), &json_args);
    assert_eq!(
        document["budget"]["spent"],
        anb_core::estimate_tokens(&String::from_utf8(text.stdout).unwrap())
    );
    assert!(
        document["more"]
            .as_str()
            .unwrap()
            .starts_with("anb --notebook 'a notebook' --session 'a long explicit conversation'")
    );
}
