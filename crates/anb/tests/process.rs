//! The wiring between a reply and the shell: which stream carries it, what
//! exit it earns, and the fail-soft a session hook depends on. `execute`
//! answers what the reply says; only a process answers what the shell then
//! sees.

use std::fmt::Write as _;
use std::fs;
use std::io::Read as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

/// One command against the notebook at `root`. `HOME` is pointed at the
/// case's own directory: a Status reads the user's notebook behind the
/// project's, and the developer's own must not decide what a case proves.
fn anb(root: &Path, line: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_anb"))
        .args(["--notebook", root.to_str().unwrap()])
        .args(line)
        .env("HOME", root)
        .env_remove("ANB_NOTEBOOK")
        .output()
        .expect("the binary runs")
}

/// [`anb`] with `stdin` piped in, for the `-` a file flag takes.
fn anb_fed(root: &Path, line: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_anb"))
        .args(["--notebook", root.to_str().unwrap()])
        .args(line)
        .env("HOME", root)
        .env_remove("ANB_NOTEBOOK")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary runs");
    std::io::Write::write_all(
        &mut child.stdin.take().expect("stdin is piped"),
        stdin.as_bytes(),
    )
    .expect("stdin is written");
    child.wait_with_output().expect("the binary exits")
}

fn record(root: &Path, path: &str, text: &str) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, text).unwrap();
}

fn task(state: &str, extra: &str) -> String {
    format!(
        "---\nid: task.demo\ntype: task\nstate: {state}\ntitle: A demo record\n{extra}created: 2026-08-24\nupdated: 2026-08-25\n---\n"
    )
}

/// `check` is what a CI step runs to be told the notebook is sound, and it
/// is told by the exit code — a reply nobody parses gates nothing.
#[test]
fn check_exits_failing_on_an_error_and_cleanly_on_a_warning() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("nb");

    record(&root, "tasks/task.demo.md", &task("bogus", ""));
    let refused = anb(&root, &["check"]);
    assert!(
        !refused.status.success(),
        "an error finding must fail the command: {}",
        String::from_utf8_lossy(&refused.stdout)
    );

    record(
        &root,
        "tasks/task.demo.md",
        &task("open", "custom: value\n"),
    );
    let warned = anb(&root, &["check"]);
    assert!(
        warned.status.success(),
        "a warning is reported, not failed: {}",
        String::from_utf8_lossy(&warned.stdout)
    );
    assert!(
        String::from_utf8_lossy(&warned.stdout).contains("unknown-field"),
        "the warning is still named"
    );
}

/// The hook runs before a session and must never be the reason one does not
/// start: whatever it meets, it answers with silence and a clean exit.
#[test]
fn the_session_hook_stays_silent_over_a_notebook_it_cannot_use() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("not-a-notebook");
    fs::write(&file, "a file where a notebook was named").unwrap();

    let hooked = anb(&file, &["status", "--hook"]);
    assert!(hooked.status.success(), "{:?}", hooked.status);
    assert!(hooked.stdout.is_empty(), "{:?}", hooked.stdout);
    assert!(hooked.stderr.is_empty(), "{:?}", hooked.stderr);

    let asked = anb(&file, &["status"]);
    assert!(
        !asked.status.success(),
        "the same notebook still refuses a status nobody hooked"
    );
}

/// An agent that asked for JSON gets JSON on every path, the ones clap
/// refuses included — a prose payload reaches its parser as a crash.
#[test]
fn an_unknown_verb_answers_in_the_format_the_caller_asked_for() {
    let dir = TempDir::new().unwrap();
    let refused = anb(dir.path(), &["--json", "bogusverb"]);

    assert!(!refused.status.success());
    let payload = String::from_utf8(refused.stderr).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(payload.trim()).unwrap_or_else(|_| panic!("not JSON: {payload}"));
    assert_eq!(parsed["error"], "unknown-command");
}

/// A reader that stops early — `anb show <id> | head` — closes the pipe
/// mid-write. That is the reader's choice, not a failed command.
#[test]
fn a_reader_that_stops_early_ends_the_reply_quietly() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("nb");
    // Past any pipe buffer, so the write blocks and then fails.
    let mut body = String::new();
    for line in 0..20_000 {
        let _ = writeln!(body, "  line {line}");
    }
    record(
        &root,
        "tasks/task.big.md",
        &format!(
            "---\nid: task.big\ntype: task\nstate: open\ntitle: Big\ncreated: 2026-08-24\nupdated: 2026-08-25\n---\n\n{body}"
        ),
    );

    let mut view = Command::new(env!("CARGO_BIN_EXE_anb"))
        .args(["--notebook", root.to_str().unwrap(), "show", "task.big"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary runs");
    // The pipe closes when this temporary drops at the end of the
    // statement — binding it would keep the reader listening.
    let mut first = [0u8; 16];
    view.stdout.take().unwrap().read_exact(&mut first).unwrap();

    let ended = view.wait_with_output().unwrap();
    assert!(
        ended.status.success(),
        "a closed pipe is the reader's choice, not a failure: {:?}, {}",
        ended.status,
        String::from_utf8_lossy(&ended.stderr)
    );
    assert!(
        ended.stderr.is_empty(),
        "nothing is reported about it either: {}",
        String::from_utf8_lossy(&ended.stderr)
    );
}

/// A body of several paragraphs travels as a file or a pipe, never as a
/// shell word: `-` on a file flag reads standard input.
#[test]
fn a_body_file_of_dash_reads_standard_input() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("nb");
    let spec =
        "# Scope\n\nThe download keeps the CSV's bytes.\n\n# Exclusions\n\nNo new columns.\n";

    let added = anb_fed(
        &root,
        &[
            "add",
            "note",
            "The download spec",
            "--kind",
            "spec",
            "--body-file",
            "-",
        ],
        spec,
    );
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stderr)
    );
    let written = fs::read_to_string(root.join("notes/note.the-download-spec.md")).unwrap();
    assert!(written.ends_with(spec), "{written}");
}
