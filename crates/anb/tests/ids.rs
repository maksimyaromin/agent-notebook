//! Independent clones allocate ids without a shared counter or clock.

use serde_json::Value;
use std::process::Command;
use tempfile::TempDir;

fn create(root: &std::path::Path, title: &str, explicit: Option<&str>) -> Value {
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
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}

#[test]
fn independent_clones_can_create_the_same_title_without_colliding() {
    let first = TempDir::new().unwrap();
    let second = TempDir::new().unwrap();
    let first_id = create(first.path(), "The same task", None)["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let second_id = create(second.path(), "The same task", None)["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_ne!(first_id, second_id);
    for id in [first_id, second_id] {
        let suffix = id.strip_prefix("task.").unwrap();
        assert_eq!(suffix.len(), 32);
        assert!(
            suffix
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        );
    }
}

#[test]
fn an_explicit_id_is_preserved() {
    let root = TempDir::new().unwrap();
    assert_eq!(
        create(
            root.path(),
            "Одинаковая задача",
            Some("task.customer-export")
        )["id"],
        "task.customer-export"
    );
}

#[test]
fn a_non_ascii_title_needs_no_handmade_id() {
    let root = TempDir::new().unwrap();
    assert!(
        create(root.path(), "Одинаковая задача", None)["id"]
            .as_str()
            .unwrap()
            .starts_with("task.")
    );
}
