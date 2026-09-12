//! Session focus is local; Task ownership remains the accountable human.

use anb::cli::Cli;
use anb::reply::{Host, Reply, execute};
use anb::session::{self, Start};
use anb_core::{MemoryStorage, Notebook, NotebookError, Record, Storage, StorageError};
use clap::Parser;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

const TODAY: &str = "2026-09-12";

fn task(id: &str, state: &str, extra: &str) -> String {
    format!(
        "---\nid: {id}\ntype: task\nstate: {state}\ntitle: {id}\nby: Product\n{extra}created: 2026-09-10\nupdated: 2026-09-10\n---\n\nThe next useful step.\n"
    )
}

fn notebook() -> MemoryStorage {
    MemoryStorage::from_files([
        ("tasks/task.alpha.md", task("task.alpha", "open", "")),
        ("tasks/task.beta.md", task("task.beta", "open", "")),
    ])
}

fn named(id: &str) -> Start {
    Start {
        id: Some(id.to_owned()),
        ..Start::default()
    }
}

fn next() -> Start {
    Start {
        next: true,
        ..Start::default()
    }
}

fn start(storage: &mut dyn Storage, session: &str, request: &Start) -> session::Started {
    session::start(storage, Some(session), Some("Ada"), request, TODAY).unwrap()
}

fn focus(storage: &dyn Storage, session: &str) -> Option<String> {
    session::focus(storage, Some(session), Some("Ada")).unwrap()
}

fn bytes(storage: &dyn Storage, id: &str) -> String {
    storage.read(&format!("tasks/{id}.md")).unwrap()
}

#[test]
fn two_sessions_of_one_human_keep_independent_focus_and_human_attribution() {
    let mut storage = notebook();
    start(&mut storage, "editor", &named("task.alpha"));
    start(&mut storage, "reviewer", &named("task.beta"));
    assert_eq!(focus(&storage, "editor").as_deref(), Some("task.alpha"));
    assert_eq!(focus(&storage, "reviewer").as_deref(), Some("task.beta"));
    for id in ["task.alpha", "task.beta"] {
        let text = bytes(&storage, id);
        let record = Record::parse(&format!("tasks/{id}.md"), &text);
        assert_eq!(record.file().field("by"), Some("Product"));
        assert_eq!(record.taken_by(), Some("Ada"));
        assert_eq!(record.file().field("session"), None);
    }
    assert_eq!(
        start(&mut storage, "editor", &Start::default())
            .transition
            .id,
        "task.alpha"
    );
}

#[test]
fn another_session_cannot_take_the_same_task_without_an_explicit_join() {
    let mut storage = notebook();
    start(&mut storage, "editor", &named("task.alpha"));
    let before = bytes(&storage, "task.alpha");
    let refused = session::start(
        &mut storage,
        Some("reviewer"),
        Some("Ada"),
        &named("task.alpha"),
        TODAY,
    )
    .unwrap_err();
    assert!(
        matches!(refused, NotebookError::SessionConflict { id, session } if id == "task.alpha" && session == "editor")
    );
    assert_eq!(bytes(&storage, "task.alpha"), before);
    assert_eq!(focus(&storage, "reviewer"), None);
    assert_eq!(storage.list(session::DIRECTORY).unwrap().len(), 1);

    let joined = start(
        &mut storage,
        "reviewer",
        &Start {
            join: true,
            ..named("task.alpha")
        },
    );
    assert!(joined.joined);
    assert_eq!(focus(&storage, "editor").as_deref(), Some("task.alpha"));
    assert_eq!(focus(&storage, "reviewer").as_deref(), Some("task.alpha"));
    assert!(start(&mut storage, "reviewer", &Start::default()).joined);
}

#[test]
fn an_ownership_refusal_changes_neither_task_nor_existing_focus() {
    let mut storage = notebook();
    start(&mut storage, "editor", &named("task.alpha"));
    storage
        .write(
            "tasks/task.beta.md",
            &task("task.beta", "open", "taken-by: Grace\n"),
        )
        .unwrap();
    let before = bytes(&storage, "task.beta");
    let refused = session::start(
        &mut storage,
        Some("editor"),
        Some("Ada"),
        &named("task.beta"),
        TODAY,
    )
    .unwrap_err();
    assert!(matches!(refused, NotebookError::Taken { taken_by, .. } if taken_by == "Grace"));
    assert_eq!(bytes(&storage, "task.beta"), before);
    assert_eq!(focus(&storage, "editor").as_deref(), Some("task.alpha"));
}

#[test]
fn a_session_id_cannot_silently_change_its_human_identity() {
    let mut storage = notebook();
    start(&mut storage, "editor", &named("task.alpha"));
    let refused = session::start(
        &mut storage,
        Some("editor"),
        Some("Grace"),
        &named("task.beta"),
        TODAY,
    )
    .unwrap_err();
    assert!(
        matches!(refused, NotebookError::SessionRecovery { session, .. } if session.as_deref() == Some("editor"))
    );
    assert_eq!(focus(&storage, "editor").as_deref(), Some("task.alpha"));
    assert_eq!(
        Record::parse("tasks/task.beta.md", &bytes(&storage, "task.beta")).state(),
        Some("open")
    );
}

#[test]
fn next_prefers_owned_work_then_untaken_work_and_never_a_colleagues_task() {
    let mut storage = notebook();
    storage
        .write(
            "tasks/task.alpha.md",
            &task("task.alpha", "open", "taken-by: Grace\npriority: 0\n"),
        )
        .unwrap();
    storage
        .write(
            "tasks/task.mine.md",
            &task("task.mine", "open", "taken-by: Ada\npriority: 3\n"),
        )
        .unwrap();
    assert_eq!(
        start(&mut storage, "first", &next()).transition.id,
        "task.mine"
    );
    assert_eq!(
        start(&mut storage, "second", &next()).transition.id,
        "task.beta"
    );
    assert!(session::start(&mut storage, Some("third"), Some("Ada"), &next(), TODAY).is_err());
    assert_eq!(focus(&storage, "third"), None);
}

#[test]
fn next_resumes_active_focus_until_it_is_closed_held_or_submitted() {
    for disposition in ["closed", "held", "review"] {
        let mut storage = notebook();
        assert_eq!(
            start(&mut storage, "editor", &next()).transition.id,
            "task.alpha"
        );
        assert!(start(&mut storage, "editor", &next()).transition.already);
        let mut core = Notebook::new(&mut storage).with_identity(Some("Ada"));
        match disposition {
            "closed" => {
                core.close("task.alpha", None, "Completed and checked.", TODAY)
                    .unwrap();
            }
            "held" => {
                core.hold("task.alpha", "Waiting for customer", None, TODAY)
                    .unwrap();
            }
            "review" => {
                core.submit("task.alpha", None, TODAY).unwrap();
            }
            _ => unreachable!(),
        }
        assert_eq!(
            start(&mut storage, "editor", &next()).transition.id,
            "task.beta",
            "{disposition}"
        );
    }
}

#[test]
fn scoped_next_keeps_a_colleagues_blocker_in_the_readiness_graph() {
    let mut storage = notebook();
    storage
        .write(
            "tasks/task.alpha.md",
            &task("task.alpha", "open", "taken-by: Grace\n"),
        )
        .unwrap();
    storage
        .write(
            "tasks/task.epic.md",
            &task(
                "task.epic",
                "open",
                "blocked-by: task.child\nblocked-by: task.free\n",
            ),
        )
        .unwrap();
    storage
        .write(
            "tasks/task.child.md",
            &task(
                "task.child",
                "open",
                "from: task.epic\nblocked-by: task.alpha\n",
            ),
        )
        .unwrap();
    storage
        .write(
            "tasks/task.free.md",
            &task("task.free", "open", "from: task.epic\n"),
        )
        .unwrap();
    let request = Start {
        hub: Some("task.epic".to_owned()),
        ..next()
    };
    assert_eq!(
        start(&mut storage, "editor", &request).transition.id,
        "task.free"
    );
    assert_eq!(
        Record::parse("tasks/task.child.md", &bytes(&storage, "task.child")).state(),
        Some("open")
    );
}

#[test]
fn next_can_move_an_active_session_to_an_explicit_different_scope() {
    let mut storage = notebook();
    start(&mut storage, "editor", &named("task.alpha"));
    assert_eq!(
        start(
            &mut storage,
            "editor",
            &Start {
                hub: Some("task.beta".to_owned()),
                ..next()
            }
        )
        .transition
        .id,
        "task.beta"
    );
}

#[test]
fn bare_start_does_not_pick_one_of_several_active_tasks_without_focus() {
    let mut storage = notebook();
    start(&mut storage, "first", &named("task.alpha"));
    start(&mut storage, "second", &named("task.beta"));
    let refused =
        session::start(&mut storage, None, Some("Ada"), &Start::default(), TODAY).unwrap_err();
    assert!(matches!(refused, NotebookError::InvalidArgument { .. }));
    assert_eq!(session::focus(&storage, None, Some("Ada")).unwrap(), None);
}

fn recall(storage: &mut MemoryStorage, session: Option<&str>) -> anb::recall::Recall {
    let cli = Cli::try_parse_from(["anb", "recall"]).unwrap();
    let reply = execute(
        cli.command,
        storage,
        Host {
            session,
            identity: || Some("Ada".to_owned()),
            read_file: &|_| panic!("recall reads no external prose"),
            lost_proofs: &|_| Vec::new(),
            user_notebook: None,
            personal_notebook: None,
            audience: anb::recall::Audience::Project,
            project_dir: Path::new("."),
            today: TODAY,
        },
    )
    .unwrap();
    let Reply::Recalled(reply) = reply else {
        panic!("expected recall")
    };
    *reply
}

#[test]
fn recall_follows_each_session_not_updated_date_or_another_sessions_focus() {
    let mut storage = notebook();
    start(&mut storage, "first", &named("task.alpha"));
    start(&mut storage, "second", &named("task.beta"));
    assert_eq!(
        recall(&mut storage, Some("first")).focus.unwrap().id,
        "task.alpha"
    );
    assert_eq!(
        recall(&mut storage, Some("second")).focus.unwrap().id,
        "task.beta"
    );
    let ambiguous = recall(&mut storage, None);
    assert_eq!(ambiguous.focus, None);
    assert_eq!(ambiguous.work.active.len(), 2);
}

struct FaultStorage {
    inner: MemoryStorage,
    writes: usize,
    fail_at: Option<usize>,
    after_commit: bool,
}

impl FaultStorage {
    fn new(fail_at: usize, after_commit: bool) -> Self {
        Self {
            inner: notebook(),
            writes: 0,
            fail_at: Some(fail_at),
            after_commit,
        }
    }
}

impl Storage for FaultStorage {
    fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
        self.inner.list(dir)
    }
    fn read(&self, path: &str) -> Result<String, StorageError> {
        self.inner.read(path)
    }
    fn write(&mut self, path: &str, text: &str) -> Result<(), StorageError> {
        self.writes += 1;
        let fails = self.fail_at == Some(self.writes);
        if !fails || self.after_commit {
            self.inner.write(path, text)?;
        }
        if fails {
            return Err(StorageError::Io {
                path: path.to_owned(),
                detail: "injected atomic-write failure".to_owned(),
            });
        }
        Ok(())
    }
    fn remove(&mut self, path: &str) -> Result<(), StorageError> {
        self.inner.remove(path)
    }
}

#[test]
fn each_interrupted_write_retries_the_same_next_task_before_and_after_commit() {
    for step in 1..=3 {
        for after_commit in [false, true] {
            let mut storage = FaultStorage::new(step, after_commit);
            let refused = session::start(&mut storage, Some("editor"), Some("Ada"), &next(), TODAY)
                .unwrap_err();
            assert!(
                matches!(refused, NotebookError::SessionRecovery { .. }),
                "step {step}, committed {after_commit}"
            );
            storage.fail_at = None;
            assert_eq!(
                start(&mut storage, "editor", &next()).transition.id,
                "task.alpha",
                "step {step}, committed {after_commit}"
            );
            assert_eq!(focus(&storage, "editor").as_deref(), Some("task.alpha"));
            assert_eq!(
                Record::parse("tasks/task.beta.md", &bytes(&storage, "task.beta")).state(),
                Some("open")
            );
            let writes = storage.writes;
            assert!(start(&mut storage, "editor", &next()).transition.already);
            assert_eq!(storage.writes, writes, "a completed retry writes nothing");
        }
    }
}

#[test]
fn task_validation_finishes_before_any_session_intent_is_written() {
    let mut storage = FaultStorage::new(1, false);
    storage
        .inner
        .write(
            "tasks/task.alpha.md",
            &task("task.alpha", "open", "link: context note.absent\n"),
        )
        .unwrap();
    let original = bytes(&storage, "task.alpha");
    let refused = session::start(
        &mut storage,
        Some("editor"),
        Some("Ada"),
        &named("task.alpha"),
        TODAY,
    )
    .unwrap_err();
    assert!(matches!(refused, NotebookError::InvalidRecord { .. }));
    assert_eq!(storage.writes, 0);
    assert_eq!(bytes(&storage, "task.alpha"), original);
    assert_eq!(focus(&storage, "editor"), None);
}

#[test]
fn session_names_are_data_not_paths() {
    let mut storage = notebook();
    start(&mut storage, "../../Ada/审查", &named("task.alpha"));
    let files = storage.list(session::DIRECTORY).unwrap();
    assert_eq!(files.len(), 1);
    let file = files[0].strip_prefix(".sessions.tmp/").unwrap();
    assert!(
        file.strip_suffix(".json")
            .unwrap()
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    );
    assert_eq!(
        focus(&storage, "../../Ada/审查").as_deref(),
        Some("task.alpha")
    );
}

#[test]
fn an_interrupted_focus_change_retains_the_previous_focus_and_reserves_both_tasks() {
    let mut storage = FaultStorage::new(usize::MAX, false);
    start(&mut storage, "editor", &named("task.alpha"));
    storage.fail_at = Some(storage.writes + 2);
    assert!(
        session::start(
            &mut storage,
            Some("editor"),
            Some("Ada"),
            &named("task.beta"),
            TODAY
        )
        .is_err()
    );
    let path = storage.list(session::DIRECTORY).unwrap().pop().unwrap();
    let state: serde_json::Value = serde_json::from_str(&storage.read(&path).unwrap()).unwrap();
    assert_eq!(state["focus"], "task.alpha");
    assert_eq!(state["pending"]["task"], "task.beta");
    assert!(matches!(
        session::focus(&storage, Some("editor"), Some("Ada")),
        Err(NotebookError::SessionRecovery { .. })
    ));
    let refused = session::start(
        &mut storage,
        Some("other"),
        Some("Ada"),
        &Start {
            join: true,
            ..named("task.beta")
        },
        TODAY,
    )
    .unwrap_err();
    assert!(
        matches!(refused, NotebookError::SessionRecovery { session, .. } if session.as_deref() == Some("editor"))
    );
    storage.fail_at = None;
    start(&mut storage, "editor", &Start::default());
    assert_eq!(focus(&storage, "editor").as_deref(), Some("task.beta"));
}

#[test]
fn recovery_refuses_to_overwrite_a_task_changed_after_the_intent_was_saved() {
    let mut storage = FaultStorage::new(2, false);
    assert!(
        session::start(
            &mut storage,
            Some("editor"),
            Some("Ada"),
            &named("task.alpha"),
            TODAY
        )
        .is_err()
    );
    let changed = format!(
        "{}\nA colleague's new finding.\n",
        bytes(&storage, "task.alpha")
    );
    storage
        .inner
        .write("tasks/task.alpha.md", &changed)
        .unwrap();
    storage.fail_at = None;
    let refused = session::start(
        &mut storage,
        Some("editor"),
        Some("Ada"),
        &Start::default(),
        TODAY,
    )
    .unwrap_err();
    assert!(
        matches!(refused, NotebookError::SessionRecovery { reason, .. } if reason.contains("changed"))
    );
    assert_eq!(bytes(&storage, "task.alpha"), changed);
    assert!(session::focus(&storage, Some("editor"), Some("Ada")).is_err());
}

#[test]
fn corrupt_session_data_is_not_mistaken_for_no_active_session() {
    let mut storage = notebook();
    start(&mut storage, "editor", &named("task.alpha"));
    let path = storage.list(session::DIRECTORY).unwrap().pop().unwrap();
    storage.write(&path, "broken JSON").unwrap();
    let known = session::focus(&storage, Some("editor"), Some("Ada")).unwrap_err();
    let unknown =
        session::start(&mut storage, Some("other"), Some("Ada"), &next(), TODAY).unwrap_err();
    let command = Cli::try_parse_from(["anb", "start", "--next"]).unwrap();
    let subject = anb::recovery::subject(&command.command);
    let known: serde_json::Value =
        serde_json::from_str(&anb::json::render_error(&known, &subject)).unwrap();
    assert_eq!(known["error"], "session-recovery");
    assert_eq!(known["session"], "editor");
    assert_eq!(known["path"], path);
    assert_eq!(
        known["try"],
        serde_json::json!(["anb start --session editor"])
    );
    let unknown: serde_json::Value =
        serde_json::from_str(&anb::json::render_error(&unknown, &subject)).unwrap();
    assert_eq!(unknown["error"], "session-recovery");
    assert!(unknown.get("session").is_none());
    assert_eq!(unknown["path"], path);
    assert_eq!(unknown["try"], serde_json::json!(["anb start --help"]));
}

fn command(root: &Path, session: Option<&str>) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_anb"));
    command
        .args(["--notebook", root.to_str().unwrap(), "--json"])
        .env("HOME", root)
        .env("ANB_BY", "Ada")
        .env_remove("ANB_NOTEBOOK")
        .env_remove("ANB_SESSION")
        .env_remove("CODEX_THREAD_ID");
    if let Some(session) = session {
        command.args(["--session", session]);
    }
    command
}

fn disk_notebook() -> TempDir {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("tasks")).unwrap();
    for id in ["task.alpha", "task.beta"] {
        std::fs::write(
            dir.path().join(format!("tasks/{id}.md")),
            task(id, "open", ""),
        )
        .unwrap();
    }
    dir
}

fn json(output: &Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn simultaneous_next_commands_select_distinct_tasks_under_the_command_lock() {
    let dir = disk_notebook();
    let mut first = command(dir.path(), Some("first"));
    first
        .args(["start", "--next"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut second = command(dir.path(), Some("second"));
    second
        .args(["start", "--next"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let first = first.spawn().unwrap();
    let second = second.spawn().unwrap();
    let ids = [
        json(&first.wait_with_output().unwrap())["id"].clone(),
        json(&second.wait_with_output().unwrap())["id"].clone(),
    ];
    assert_ne!(ids[0], ids[1]);
    for (session, id) in ["first", "second"].into_iter().zip(ids) {
        let recalled = json(
            &command(dir.path(), Some(session))
                .arg("recall")
                .output()
                .unwrap(),
        );
        assert_eq!(recalled["focus"]["id"], id);
    }
}

#[test]
fn simultaneous_starts_of_one_task_have_one_winner_and_a_typed_session_conflict() {
    let dir = disk_notebook();
    let mut children = Vec::new();
    for session in ["first", "second"] {
        children.push(
            command(dir.path(), Some(session))
                .args(["start", "task.alpha"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap(),
        );
    }
    let outputs = children
        .into_iter()
        .map(|child| child.wait_with_output().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        outputs
            .iter()
            .filter(|output| output.status.success())
            .count(),
        1
    );
    let refused = outputs
        .iter()
        .find(|output| !output.status.success())
        .unwrap();
    let refusal: serde_json::Value = serde_json::from_slice(&refused.stderr).unwrap();
    assert_eq!(refusal["error"], "session-conflict");
    assert_eq!(refusal["id"], "task.alpha");
    assert!(["first", "second"].contains(&refusal["session"].as_str().unwrap()));
}

#[test]
fn explicit_session_precedes_anb_session_which_precedes_the_native_agent_session() {
    for (explicit, anb_session, expected) in [
        (Some("flag"), Some("environment"), "flag"),
        (None, Some("environment"), "environment"),
        (None, None, "native"),
    ] {
        let dir = disk_notebook();
        let mut command = command(dir.path(), explicit);
        command.env("CODEX_THREAD_ID", "native");
        if let Some(session) = anb_session {
            command.env("ANB_SESSION", session);
        }
        assert_eq!(
            json(&command.args(["start", "task.alpha"]).output().unwrap())["session"],
            expected
        );
    }
}

#[test]
fn invalid_session_input_keeps_the_session_hook_fail_soft() {
    let dir = disk_notebook();
    let output = command(dir.path(), None)
        .env("ANB_SESSION", "\n")
        .arg("hook")
        .output()
        .unwrap();
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        payload["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .contains("recall is unavailable")
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn session_state_is_ignored_even_when_the_notebook_has_custom_ignore_rules() {
    let dir = disk_notebook();
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(dir.path())
            .status()
            .unwrap()
            .success()
    );
    std::fs::write(dir.path().join(".gitignore"), "scratch/\n").unwrap();
    json(
        &command(dir.path(), Some("editor"))
            .args(["start", "task.alpha"])
            .output()
            .unwrap(),
    );
    let state = std::fs::read_dir(dir.path().join(session::DIRECTORY))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .unwrap();
    let ignored = Command::new("git")
        .args(["check-ignore", "--quiet"])
        .arg(&state)
        .current_dir(dir.path())
        .status()
        .unwrap();
    assert!(
        ignored.success(),
        "session identity and pending Task bytes must not become repository content"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".gitignore")).unwrap(),
        "scratch/\n"
    );
}

#[cfg(unix)]
#[test]
fn a_symlinked_session_directory_cannot_redirect_local_state_outside_the_notebook() {
    let dir = disk_notebook();
    let outside = TempDir::new().unwrap();
    std::os::unix::fs::symlink(outside.path(), dir.path().join(session::DIRECTORY)).unwrap();
    let output = command(dir.path(), Some("editor"))
        .args(["start", "task.alpha"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("tasks/task.alpha.md")).unwrap(),
        task("task.alpha", "open", "")
    );
}
