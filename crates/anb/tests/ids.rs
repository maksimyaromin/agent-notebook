//! A record's id is read far more often than it is written, so `add`
//! derives it from the title unless the caller chooses one.

use serde_json::Value;
use std::process::{Command, Output};
use tempfile::TempDir;

fn add(root: &std::path::Path, title: &str, explicit: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_anb"));
    command
        .current_dir(root)
        .args(["--json", "add", "task", title])
        .env("HOME", root)
        .env("ANB_BY", "Ada")
        .env_remove("ANB_NOTEBOOK")
        .env_remove("ANB_SESSION")
        .env_remove("CODEX_THREAD_ID");
    if let Some(id) = explicit {
        command.args(["--id", id]);
    }
    command.output().unwrap()
}

fn created(root: &std::path::Path, title: &str, explicit: Option<&str>) -> Value {
    let result = add(root, title, explicit);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}

#[test]
fn the_title_becomes_the_id() {
    let root = TempDir::new().unwrap();
    let reply = created(root.path(), "Parser accepts fenced bodies", None);
    assert_eq!(reply["id"], "task.parser-accepts-fenced-bodies");
    assert!(reply.get("collision").is_none(), "{reply}");
}

#[test]
fn a_title_already_taken_gets_a_suffix_and_the_reply_names_the_holder() {
    let root = TempDir::new().unwrap();
    created(root.path(), "Parser accepts fenced bodies", None);
    let second = created(root.path(), "Parser accepts fenced bodies", None);
    let id = second["id"].as_str().unwrap();
    assert!(
        id.starts_with("task.parser-accepts-fenced-bodies-"),
        "{second}"
    );
    assert_eq!(second["collision"], "task.parser-accepts-fenced-bodies");
}

#[test]
fn a_title_without_an_ascii_word_is_refused_until_an_id_is_chosen() {
    let root = TempDir::new().unwrap();
    let refused = add(root.path(), "Одинаковая задача", None);
    assert!(!refused.status.success());
    let payload: Value = serde_json::from_slice(&refused.stderr).unwrap();
    assert_eq!(payload["error"], "invalid-argument");
    assert!(
        !root.path().join(".agent-notebook/tasks").exists(),
        "a refused add writes no record"
    );
    assert_eq!(
        created(
            root.path(),
            "Одинаковая задача",
            Some("task.customer-export")
        )["id"],
        "task.customer-export"
    );
}
