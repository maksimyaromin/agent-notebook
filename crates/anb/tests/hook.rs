//! Native host input and output, exercised through the installed binary.

use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;

struct Fixture {
    _directory: TempDir,
    project: PathBuf,
    home: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = TempDir::new().unwrap();
        let project = directory.path().join("project");
        let home = directory.path().join("home");
        fs::create_dir(&project).unwrap();
        fs::create_dir(&home).unwrap();
        Self {
            _directory: directory,
            project,
            home,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_anb"));
        command
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env("ANB_BY", "Ada")
            .env_remove("ANB_NOTEBOOK")
            .env_remove("ANB_SESSION")
            .env_remove("CODEX_THREAD_ID")
            .env_remove("CLAUDE_ENV_FILE");
        command
    }

    fn run(&self, args: &[&str]) -> Value {
        let output = self.command().arg("--json").args(args).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }

    fn focus(&self, session: &str, task: &str) {
        self.run(&["add", "task", task, "--id", task]);
        self.run(&["--session", session, "start", task]);
    }
}

fn fed(mut command: Command, input: &[u8]) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let written = child.stdin.take().unwrap().write_all(input);
    if let Err(error) = written {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    }
    child.wait_with_output().unwrap()
}

fn context(output: &Output) -> String {
    assert!(
        output.status.success(),
        "hooks must not stop their host: {output:?}"
    );
    assert!(
        output.stderr.is_empty(),
        "hook diagnostics belong in the context: {output:?}"
    );
    let payload: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        payload["hookSpecificOutput"]["hookEventName"],
        "SessionStart"
    );
    payload["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn document(output: &Output) -> Value {
    let context = context(output);
    let (_, toon) = context.split_once("\n\n").unwrap();
    reddb_io_toon::decode(toon).unwrap().to_json_value()
}

#[test]
fn host_input_recalls_exact_session_focus_with_private_practices_without_writing_focus() {
    let project = Fixture::new();
    project.focus("first", "task.alpha");
    project.focus("second", "task.beta");
    project.run(&[
        "--personal",
        "add",
        "note",
        "Review practice",
        "--id",
        "note.practice",
        "--body",
        "Check customer examples first.",
    ]);
    let session_directory = project.project.join(".agent-notebook/.sessions.tmp");
    let before = fs::read_dir(&session_directory)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.path(), fs::read(entry.path()).unwrap())
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut command = project.command();
    command.arg("hook");
    let output = fed(command, &serde_json::to_vec(&json!({"hook_event_name":"SessionStart", "session_id":"first", "source":"resume", "unknown_future_field":true})).unwrap());
    let recalled = document(&output);
    assert_eq!(recalled["focus"]["id"], "task.alpha");
    assert!(context(&output).contains("Check customer examples first."));
    assert!(context(&output).contains("personal"));
    let after = fs::read_dir(&session_directory)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.path(), fs::read(entry.path()).unwrap())
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(after, before);
}

#[test]
fn an_unremembered_host_session_does_not_create_local_focus_state() {
    let project = Fixture::new();
    let mut command = project.command();
    command.arg("hook");
    let output = fed(
        command,
        br#"{"session_id":"new","hook_event_name":"SessionStart"}"#,
    );
    assert!(document(&output).get("error").is_none());
    assert!(!project.project.join(".agent-notebook").exists());
}

#[test]
fn a_quiet_project_still_recalls_a_colleagues_shared_rule_at_session_start() {
    let project = Fixture::new();
    project.run(&[
        "add",
        "decision",
        "Tenant boundary",
        "--id",
        "decision.tenant",
        "--kind",
        "rule",
        "--by",
        "Grace",
        "--body",
        "Every export stays within its tenant boundary.",
    ]);
    let record_path = project
        .project
        .join(".agent-notebook/decisions/decision.tenant.md");
    let before = fs::read(&record_path).unwrap();
    let mut command = project.command();
    command.arg("hook");
    let output = fed(
        command,
        br#"{"session_id":"fresh","hook_event_name":"SessionStart"}"#,
    );
    let recalled = document(&output);
    assert_eq!(recalled["work"]["quiet"], true);
    assert_eq!(recalled["work"]["counts"]["tasks"], 0);
    assert!(
        recalled["memories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|memory| {
                memory["id"] == "decision.tenant"
                    && memory["scope"] == "project"
                    && memory["body"]["head"].as_str().is_some_and(|body| {
                        body.trim() == "Every export stays within its tenant boundary."
                    })
            }),
        "{recalled}"
    );
    assert!(
        !project
            .project
            .join(".agent-notebook/.sessions.tmp")
            .exists()
    );
    assert_eq!(fs::read(record_path).unwrap(), before);
}

#[test]
fn hook_session_selection_keeps_explicit_environment_and_native_precedence() {
    let project = Fixture::new();
    for (session, task) in [
        ("flag", "task.flag"),
        ("environment", "task.environment"),
        ("native", "task.native"),
        ("input", "task.input"),
    ] {
        project.focus(session, task);
    }
    for (flag, environment, native, expected) in [
        (
            Some("flag"),
            Some("environment"),
            Some("native"),
            "task.flag",
        ),
        (
            None,
            Some("environment"),
            Some("native"),
            "task.environment",
        ),
        (None, None, Some("native"), "task.native"),
        (None, None, None, "task.input"),
    ] {
        let mut command = project.command();
        command.arg("hook");
        if let Some(flag) = flag {
            command.args(["--session", flag]);
        }
        if let Some(session) = environment {
            command.env("ANB_SESSION", session);
        }
        if let Some(session) = native {
            command.env("CODEX_THREAD_ID", session);
        }
        assert_eq!(
            document(&fed(command, br#"{"session_id":"input"}"#))["focus"]["id"],
            expected
        );
    }
}

#[test]
fn claude_exports_roundtrip_shell_metacharacters_without_executing_them() {
    let project = Fixture::new();
    let environment = project.home.join("session-env");
    let marker = project.home.join("must-not-exist");
    let session = format!(
        "Ada 'quoted' $(touch {}) `touch {}` \"$PATH\" ; # \\ 雪",
        marker.display(),
        marker.display()
    );
    fs::write(&environment, "export OTHER='kept'").unwrap();
    let mut command = project.command();
    command.arg("hook").env("CLAUDE_ENV_FILE", &environment);
    context(&fed(
        command,
        &serde_json::to_vec(&json!({"session_id":session})).unwrap(),
    ));
    let first = fs::read(&environment).unwrap();
    let mut command = project.command();
    command.arg("hook").env("CLAUDE_ENV_FILE", &environment);
    context(&fed(
        command,
        &serde_json::to_vec(&json!({"session_id":session})).unwrap(),
    ));
    assert_eq!(
        fs::read(&environment).unwrap(),
        first,
        "replayed exports append nothing"
    );
    let sourced = Command::new("sh")
        .args([
            "-c",
            ". \"$1\"; printf '%s\\n%s' \"$ANB_SESSION\" \"$OTHER\"",
            "hook-test",
        ])
        .arg(&environment)
        .output()
        .unwrap();
    assert!(
        sourced.status.success(),
        "{}",
        String::from_utf8_lossy(&sourced.stderr)
    );
    assert_eq!(
        String::from_utf8(sourced.stdout).unwrap(),
        format!("{session}\nkept")
    );
    assert!(!marker.exists());
    assert!(!project.project.join(".agent-notebook").exists());
}

#[test]
fn malformed_or_wrong_host_input_is_reported_as_unavailable_not_empty_memory() {
    let project = Fixture::new();
    for input in [
        b"{".as_slice(),
        b"[]",
        br#"{"session_id":7}"#,
        br#"{"hook_event_name":"Stop","session_id":"wrong"}"#,
        b"\xff",
    ] {
        let mut command = project.command();
        command.arg("hook");
        let output = fed(command, input);
        assert!(context(&output).contains("recall is unavailable"));
        assert_eq!(document(&output)["error"], "invalid-argument");
        assert!(!project.project.join(".agent-notebook").exists());
    }
}

#[test]
fn oversized_input_is_bounded_and_never_echoed_into_the_failure() {
    let project = Fixture::new();
    let mut command = project.command();
    command.arg("hook");
    let input =
        serde_json::to_vec(&json!({"session_id":"valid", "extra":"sensitive ".repeat(10_000)}))
            .unwrap();
    let output = fed(command, &input);
    assert!(context(&output).contains("64 KiB"));
    assert!(output.stdout.len() < 2000);
    assert!(!context(&output).contains("sensitive"));
}

#[test]
fn a_pipe_kept_open_by_the_host_cannot_stall_startup() {
    let project = Fixture::new();
    let mut child = project
        .command()
        .arg("hook")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let input = child.stdin.take().unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    while child.try_wait().unwrap().is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    if child.try_wait().unwrap().is_none() {
        child.kill().unwrap();
        panic!("the hook waited indefinitely for input")
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(context(&output).contains("input did not finish"));
}

#[test]
fn control_characters_in_session_ids_never_reach_an_environment_script() {
    let project = Fixture::new();
    let environment = project.home.join("session-env");
    for control in ['\0', '\n', '\r', '\t', '\u{1b}', '\u{7f}'] {
        let mut command = project.command();
        command.arg("hook").env("CLAUDE_ENV_FILE", &environment);
        let input =
            serde_json::to_vec(&json!({"session_id":format!("unsafe{control}id")})).unwrap();
        assert!(context(&fed(command, &input)).contains("control characters"));
        assert!(!environment.exists());
    }
}

#[cfg(unix)]
#[test]
fn symlinked_or_nonregular_environment_sinks_are_refused_without_changing_the_target() {
    let project = Fixture::new();
    let target = project.home.join("existing");
    let symlink = project.home.join("link");
    fs::write(&target, "export OTHER=kept\n").unwrap();
    std::os::unix::fs::symlink(&target, &symlink).unwrap();
    for sink in [&symlink, &project.home] {
        let mut command = project.command();
        command.arg("hook").env("CLAUDE_ENV_FILE", sink);
        let output = fed(command, br#"{"session_id":"valid"}"#);
        assert!(context(&output).contains("not a regular file"));
    }
    assert_eq!(fs::read_to_string(target).unwrap(), "export OTHER=kept\n");
}

#[test]
fn a_locked_environment_file_is_reported_without_waiting_or_replacing_exports() {
    let project = Fixture::new();
    let environment = project.home.join("session-env");
    fs::write(&environment, "export OTHER=kept\n").unwrap();
    let held = fs::File::open(&environment).unwrap();
    held.lock().unwrap();
    let mut command = project.command();
    command.arg("hook").env("CLAUDE_ENV_FILE", &environment);
    let output = fed(command, br#"{"session_id":"valid"}"#);
    assert!(context(&output).contains("cannot claim the environment file"));
    assert_eq!(
        fs::read_to_string(environment).unwrap(),
        "export OTHER=kept\n"
    );
}

#[cfg(unix)]
#[test]
fn a_stdin_read_failure_is_not_an_empty_recall() {
    let project = Fixture::new();
    let input = fs::File::open(&project.project).unwrap();
    let output = project
        .command()
        .arg("hook")
        .stdin(Stdio::from(input))
        .output()
        .unwrap();
    assert!(context(&output).contains("cannot read SessionStart input"));
    assert_eq!(document(&output)["error"], "invalid-argument");
}
