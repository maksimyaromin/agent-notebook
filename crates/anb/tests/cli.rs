//! The thin e2e pass over golden outputs: a command line in, the rendered
//! reply out, over an in-memory notebook. Every expected text derives from
//! the output contract, never from running the code.

use anb::cli::{Cli, Command};
use anb::reply::{Host, execute};
use anb::{json, text};
use anb_core::MemoryStorage;
use anb_core::StorageError;
use clap::{CommandFactory as _, Parser};

const TODAY: &str = "2026-08-28";
const IDENTITY: &str = "Maks";

#[test]
fn divergent_move_recovery_names_both_files_without_destructive_or_creation_retries() {
    for command in ["archive", "restore"] {
        let live = record_file(
            "task.demo",
            "task",
            "closed",
            "Verification",
            &[],
            "Alex verified the browser.\n",
        );
        let archived = record_file(
            "task.demo",
            "task",
            "closed",
            "Verification",
            &[],
            "Grace verified privacy.\n",
        );
        let mut storage = storage_with(&[
            ("tasks/task.demo.md".to_owned(), live.clone()),
            ("archive/tasks/task.demo.md".to_owned(), archived.clone()),
        ]);

        let output = run(&mut storage, &[command, "task.demo", "--json"]).unwrap_err();
        let error: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(error["error"], "duplicate-id");
        assert_eq!(error["findings"][0], "live: tasks/task.demo.md");
        assert_eq!(error["findings"][1], "archived: archive/tasks/task.demo.md");
        let guidance = error["findings"][2].as_str().unwrap();
        assert!(guidance.contains("Preserve both originals"));
        assert!(guidance.contains("Do not use anb delete"));
        assert_eq!(error["try"], serde_json::json!(["anb check --all"]));
        assert_eq!(
            anb_core::Storage::read(&storage, "tasks/task.demo.md").unwrap(),
            live
        );
        assert_eq!(
            anb_core::Storage::read(&storage, "archive/tasks/task.demo.md").unwrap(),
            archived
        );
    }
}

#[test]
fn close_accepts_one_outcome_path_without_special_proof_modes() {
    for flag in ["--note", "--pr", "--sha", "--report", "--no-proof"] {
        let refused = Cli::try_parse_from(["anb", "close", "task.demo", flag]);
        assert!(
            matches!(refused, Err(error) if error.kind() == clap::error::ErrorKind::UnknownArgument),
            "{flag}"
        );
    }
    assert!(
        Cli::try_parse_from([
            "anb",
            "close",
            "task.demo",
            "--body",
            "Tests pass; evidence is in the linked pull request."
        ])
        .is_ok()
    );
}

#[test]
fn unfinished_close_returns_blockers_without_suggesting_their_removal() {
    let mut storage = storage_with(&[
        open_task("task.child", "A prerequisite", &[]),
        open_task(
            "task.hub",
            "The overall result",
            &["blocked-by: task.child"],
        ),
    ]);
    ok(&mut storage, &["start", "task.hub"]);
    let before = anb_core::Storage::read(&storage, "tasks/task.hub.md").unwrap();
    let output = run(
        &mut storage,
        &["close", "task.hub", "--body", "Done.", "--json"],
    )
    .unwrap_err();
    let error: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(error["error"], "unfinished-dependencies");
    assert_eq!(error["findings"], serde_json::json!(["task.child"]));
    assert_eq!(error["try"], serde_json::json!(["anb show task.child"]));
    assert_eq!(
        anb_core::Storage::read(&storage, "tasks/task.hub.md").unwrap(),
        before
    );
    ok(
        &mut storage,
        &["close", "task.hub", "--reason", "No longer needed."],
    );
    let child = anb_core::Storage::read(&storage, "tasks/task.child.md").unwrap();
    assert!(child.contains("state: open\n"));
}

#[test]
fn prose_arguments_accept_hyphen_leading_values() {
    let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
    for line in [
        vec!["comment", "task.demo", "--body-file is an input option"],
        vec!["comment", "task.demo", "--body", "- A Markdown item"],
        vec!["edit", "task.demo", "--title", "--body-file input"],
        vec!["edit", "task.demo", "--body", "--body-file accepts a path"],
        vec!["hold", "task.demo", "--reason", "--body-file needs review"],
    ] {
        assert!(run(&mut storage, &line).is_ok(), "{line:?}");
    }
    assert!(
        run(
            &mut storage,
            &[
                "add",
                "note",
                "--body-file details",
                "--body",
                "- First item"
            ]
        )
        .is_ok()
    );
    let task = anb_core::Storage::read(&storage, "tasks/task.demo.md").unwrap();
    let record = anb_core::Record::parse("tasks/task.demo.md", &task);
    assert_eq!(record.file().field("title"), Some("--body-file input"));
    assert_eq!(record.hold(), Some("--body-file needs review"));
    assert!(task.ends_with("--body-file accepts a path\n"));
}

#[test]
fn comment_body_files_and_inline_text_use_the_same_entry_format() {
    let mut storage = storage_with(&[(
        "notes/note.demo.md".to_owned(),
        record_file(
            "note.demo",
            "note",
            "active",
            "Domain names",
            &[],
            "The investigation.\n",
        ),
    )]);
    let text = "The names are confirmed.\nSee the domain map.\n";
    run_reading(
        &mut storage,
        &[
            "comment",
            "note.demo",
            "--via",
            "codex",
            "--body-file",
            "outcome.md",
        ],
        &[("outcome.md", text)],
    )
    .unwrap();
    let written = anb_core::Storage::read(&storage, "notes/note.demo.md").unwrap();
    assert!(
        written.ends_with(
            "- 2026-08-28 Maks/codex: The names are confirmed.\n  See the domain map.\n"
        )
    );
    let replay = run(
        &mut storage,
        &[
            "comment",
            "note.demo",
            "--body",
            text,
            "--via",
            "codex",
            "--json",
        ],
    )
    .unwrap();
    assert!(replay.contains("\"already\":true"));
    assert_eq!(
        anb_core::Storage::read(&storage, "notes/note.demo.md").unwrap(),
        written
    );
}

#[test]
fn retiring_an_idea_preserves_its_outcome_in_the_archive() {
    let mut storage = storage_with(&[(
        "notes/note.demo.md".to_owned(),
        record_file(
            "note.demo",
            "note",
            "active",
            "Domain names",
            &["kind: idea"],
            "The investigation.\n",
        ),
    )]);
    run(
        &mut storage,
        &[
            "retire",
            "note.demo",
            "--body",
            "Four names confirmed in the domain map.",
            "--via",
            "codex",
        ],
    )
    .unwrap();
    run(&mut storage, &["archive", "note.demo"]).unwrap();
    let archived = anb_core::Storage::read(&storage, "archive/notes/note.demo.md").unwrap();
    assert!(archived.contains("state: retired\n"));
    assert!(
        archived.ends_with("- 2026-08-28 Maks/codex: Four names confirmed in the domain map.\n")
    );
    assert!(run(&mut storage, &["check"]).is_ok());
}

#[test]
fn check_offers_a_runnable_unlink_for_a_dangling_link() {
    let mut storage = storage_with(&[open_task(
        "task.demo",
        "A demo record",
        &["link: context note.missing"],
    )]);
    let findings = anb_core::Notebook::new(&mut storage).check().unwrap();
    let repair = findings[0]
        .repair
        .as_ref()
        .expect("a dangling link has an unlink repair");
    let command = anb::reply::repair_command(repair, &findings[0].path);
    let words = shell_words(&command).skip(1).collect::<Vec<_>>();
    run(
        &mut storage,
        &words.iter().map(String::as_str).collect::<Vec<_>>(),
    )
    .unwrap();
    assert!(
        anb_core::Notebook::new(&mut storage)
            .check()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn dangling_link_records_are_not_ready_or_recalled_as_valid_knowledge() {
    let mut storage = storage_with(&[
        open_task(
            "task.demo",
            "A demo record",
            &["link: context note.missing"],
        ),
        (
            "notes/note.demo.md".to_owned(),
            record_file(
                "note.demo",
                "note",
                "active",
                "A fact",
                &["link: context note.missing"],
                "An unsupported fact.\n",
            ),
        ),
    ]);
    let notebook = anb_core::Notebook::new(&mut storage);
    assert!(
        notebook
            .ready(&anb_core::Filter::default())
            .unwrap()
            .is_empty()
    );
    let recalled = notebook.recall(None, None).unwrap();
    assert!(recalled.records.is_empty());
    assert!(recalled.invalid.contains(&"notes/note.demo.md".to_owned()));
}

#[test]
fn close_keeps_the_outcome_on_the_task_without_a_report_note() {
    let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
    run(&mut storage, &["start", "task.demo"]).unwrap();
    run_reading(
        &mut storage,
        &[
            "close",
            "task.demo",
            "--body-file",
            "outcome.md",
            "--via",
            "codex",
        ],
        &[("outcome.md", "Parser and preservation checks pass.\n")],
    )
    .unwrap();
    let written = anb_core::Storage::read(&storage, "tasks/task.demo.md").unwrap();
    assert!(written.contains("state: closed\n"));
    assert!(written.ends_with("- 2026-08-28 Maks/codex: Parser and preservation checks pass.\n"));
    assert!(
        anb_core::Storage::list(&storage, "notes")
            .unwrap()
            .is_empty()
    );
    run(
        &mut storage,
        &[
            "close",
            "task.demo",
            "--body",
            "A repeated close must not replace the proof.",
        ],
    )
    .unwrap();
    assert_eq!(
        anb_core::Storage::read(&storage, "tasks/task.demo.md").unwrap(),
        written
    );
}

#[test]
fn an_outcome_signature_without_text_does_not_change_lifecycle_state() {
    let note = record_file("note.demo", "note", "active", "A note", &[], "");
    let task = record_file("task.demo", "task", "active", "A task", &[], "");
    let mut storage = storage_with(&[
        ("notes/note.demo.md".to_owned(), note.clone()),
        ("tasks/task.demo.md".to_owned(), task.clone()),
    ]);
    for line in [
        vec!["retire", "note.demo", "--via", "codex"],
        vec![
            "close",
            "task.demo",
            "--reason",
            "Cancelled",
            "--via",
            "codex",
        ],
    ] {
        assert!(run(&mut storage, &line).is_err());
    }
    assert_eq!(
        anb_core::Storage::read(&storage, "notes/note.demo.md").unwrap(),
        note
    );
    assert_eq!(
        anb_core::Storage::read(&storage, "tasks/task.demo.md").unwrap(),
        task
    );
}

/// Parse and run one command line; `Ok` is stdout, `Err` is the payload a
/// failure prints.
fn run(storage: &mut MemoryStorage, line: &[&str]) -> Result<String, String> {
    run_reading(storage, line, &[])
}

/// [`run`] with prose files from outside the notebook. A path the list
/// does not name is absent.
fn run_reading(
    storage: &mut MemoryStorage,
    line: &[&str],
    files: &[(&str, &str)],
) -> Result<String, String> {
    run_with(storage, line, &|path: &str| {
        files
            .iter()
            .find(|(named, _)| *named == path)
            .map(|(_, text)| (*text).to_owned())
            .ok_or_else(|| StorageError::NotFound {
                path: path.to_owned(),
            })
    })
}

/// One command line against a host whose one reach outside the notebook —
/// the file it reads — the case chooses.
fn run_with(
    storage: &mut MemoryStorage,
    line: &[&str],
    read_file: &dyn Fn(&str) -> Result<String, StorageError>,
) -> Result<String, String> {
    run_behind(storage, None, line, read_file)
}

/// [`run_with`] against a project whose user's notebook, when one is given,
/// stands behind it.
fn run_behind(
    storage: &mut MemoryStorage,
    user_notebook: Option<&dyn anb_core::Storage>,
    line: &[&str],
    read_file: &dyn Fn(&str) -> Result<String, StorageError>,
) -> Result<String, String> {
    let mut args = vec!["anb"];
    args.extend_from_slice(line);
    let cli = Cli::try_parse_from(args).expect("the test drives a well-formed command line");
    let subject = anb::recovery::subject(&cli.command);
    let wants_json = cli.json;
    let host = Host {
        session: cli.session.as_deref(),
        identity: || Some(IDENTITY.to_owned()),
        read_file,
        lost_proofs: &nothing_lost,
        user_notebook,
        personal_notebook: None,
        audience: anb::recall::Audience::Project,
        project_dir: std::path::Path::new("."),
        today: TODAY,
    };
    match execute(cli.command, storage, host) {
        Ok(reply) => Ok(if wants_json {
            json::render(&reply)
        } else {
            text::render(&reply)
        }),
        Err(error) => Err(if wants_json {
            json::render_error(&error, &subject)
        } else {
            text::render_error(&error, &subject)
        }),
    }
}

/// The host of a shell whose clock has gone wrong: the one fact these cases
/// vary.
fn undated_host() -> Host<'static> {
    Host {
        session: None,
        identity: || None,
        read_file: &missing_report,
        lost_proofs: &nothing_lost,
        user_notebook: None,
        personal_notebook: None,
        audience: anb::recall::Audience::Project,
        project_dir: std::path::Path::new("."),
        today: "not-a-date",
    }
}

/// The host of a shell that knows nobody: no `ANB_BY`, no git identity.
fn anonymous_host() -> Host<'static> {
    Host {
        session: None,
        identity: || None,
        read_file: &missing_report,
        lost_proofs: &nothing_lost,
        user_notebook: None,
        personal_notebook: None,
        audience: anb::recall::Audience::Project,
        project_dir: std::path::Path::new("."),
        today: TODAY,
    }
}

/// The world of a case that is not about reconciliation: it still holds
/// every proof the notebook cites.
fn nothing_lost(_: &[anb_core::CitedProof]) -> Vec<anb_core::CitedProof> {
    Vec::new()
}

/// The reader for tests that name no prose file: every path is absent.
fn missing_report(path: &str) -> Result<String, StorageError> {
    Err(StorageError::NotFound {
        path: path.to_owned(),
    })
}

fn ok(storage: &mut MemoryStorage, line: &[&str]) -> String {
    run(storage, line).expect("the command must succeed")
}

fn refused(storage: &mut MemoryStorage, line: &[&str]) -> String {
    run(storage, line).expect_err("the command must be refused")
}

fn assert_reply(actual: impl AsRef<str>, expected: serde_json::Value) {
    assert_fields(&reply_value(actual), expected);
}

fn reply_value(actual: impl AsRef<str>) -> serde_json::Value {
    if actual.as_ref().starts_with('{') {
        return serde_json::from_str(actual.as_ref()).expect("the reply is JSON");
    }
    reddb_io_toon::decode(actual.as_ref())
        .expect("the reply is canonical TOON")
        .to_json_value()
}

fn assert_fields(actual: &serde_json::Value, expected: serde_json::Value) {
    match expected {
        serde_json::Value::Object(fields) => {
            for (key, value) in fields {
                assert_fields(&actual[&key], value);
            }
        }
        serde_json::Value::Array(items) => {
            let actual = actual.as_array().expect("the reply field is an array");
            assert_eq!(actual.len(), items.len());
            for (actual, expected) in actual.iter().zip(items) {
                assert_fields(actual, expected);
            }
        }
        _ => assert_eq!(*actual, expected),
    }
}

fn record_file(
    id: &str,
    type_word: &str,
    state: &str,
    title: &str,
    extra_lines: &[&str],
    body: &str,
) -> String {
    let mut text = format!("---\nid: {id}\ntype: {type_word}\nstate: {state}\ntitle: {title}\n");
    for line in extra_lines {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str("created: 2026-08-24\nupdated: 2026-08-25\n---\n");
    text.push_str(body);
    text
}

fn storage_with(files: &[(String, String)]) -> MemoryStorage {
    MemoryStorage::from_files(
        files
            .iter()
            .map(|(path, text)| (path.as_str(), text.as_str())),
    )
}

fn open_task(id: &str, title: &str, extra_lines: &[&str]) -> (String, String) {
    (
        format!("tasks/{id}.md"),
        record_file(id, "task", "open", title, extra_lines, ""),
    )
}

fn many_open_tasks(count: usize) -> MemoryStorage {
    let files: Vec<(String, String)> = (0..count)
        .map(|n| open_task(&format!("task.t{n:02}"), "A demo record", &[]))
        .collect();
    storage_with(&files)
}

mod task_cycle_replies {
    use super::*;

    /// A review is handed to a named person and a doubt is put to one; the
    /// name lands in the envelope, and the listing narrowed to that person
    /// reaches both, its hint carrying the flag.
    #[test]
    fn a_review_and_a_question_are_addressed_to_a_person() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        ok(&mut storage, &["start", "task.demo"]);
        assert_reply(
            ok(&mut storage, &["submit", "task.demo", "--to", "Grace"]),
            serde_json::json!({
              "ok": "submit",
              "id": "task.demo",
              "from": "active",
              "to": "review"
            }),
        );
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .contains("\ntaken-by: Maks\nto: Grace\n")
        );
        ok(
            &mut storage,
            &["add", "question", "Do fences nest?", "--to", "Grace"],
        );
        assert!(
            storage
                .read("questions/question.do-fences-nest.md")
                .unwrap()
                .contains("\nby: Maks\nto: Grace\n")
        );
        let payload = ok(&mut storage, &["--json", "list", "--to", "Grace"]);
        let listed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(listed["count"], 2);
        assert_eq!(listed["records"][0]["to"], "Grace");
        assert_eq!(listed["records"][1]["to"], "Grace");
        assert_reply(
            refused(
                &mut storage,
                &["add", "decision", "Addressed", "--to", "Grace"],
            ),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "to: applies only to a task or a question",
              "try": [
                "anb add decision \"<title>\""
              ]
            }),
        );
    }

    /// The report is read from the shell's world, so a read that fails on its
    /// encoding is a refused argument the caller retypes — not the notebook's
    /// fault, and not a close.
    #[test]
    fn a_report_that_is_not_utf8_refuses_the_close_and_moves_nothing() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        let unreadable = |path: &str| {
            Err(StorageError::NotUtf8 {
                path: path.to_owned(),
            })
        };

        assert_reply(
            run_with(
                &mut storage,
                &["close", "task.demo", "--body-file", "report.md"],
                &unreadable,
            )
            .expect_err("bytes outside UTF-8 are no proof"),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "body-file: `report.md` is not UTF-8",
              "try": [
                "anb close task.demo --body \"<outcome>\"",
                "anb close task.demo --reason \"<why>\""
              ]
            }),
        );
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .contains("state: active"),
            "the task stays where it was"
        );
    }
    use anb_core::Storage as _;

    #[test]
    fn add_mints_an_id_and_answers_the_path() {
        let mut storage = MemoryStorage::new();
        let output = ok(
            &mut storage,
            &["add", "task", "Grammar parser accepts fenced envelopes"],
        );
        assert_reply(
            output,
            serde_json::json!({
              "ok": "add",
              "id": "task.grammar-parser-accepts-fenced-envelopes",
              "path": "tasks/task.grammar-parser-accepts-fenced-envelopes.md"
            }),
        );
        let written = storage
            .read("tasks/task.grammar-parser-accepts-fenced-envelopes.md")
            .unwrap();
        assert!(
            written.contains("\nby: Maks\n"),
            "git identity fills `by` when no flag names one: {written}"
        );
    }

    /// A planner hands a Task over in the command that writes it, and a
    /// developer takes one for themself without spelling their own name.
    #[test]
    fn add_task_hands_the_task_over_or_takes_it_for_the_caller() {
        let mut storage = MemoryStorage::new();
        ok(
            &mut storage,
            &["add", "task", "Planned for Grace", "--taken-by", "Grace"],
        );
        ok(&mut storage, &["add", "task", "Planned for me", "--mine"]);
        ok(&mut storage, &["add", "task", "Planned for nobody"]);
        assert!(
            storage
                .read("tasks/task.planned-for-grace.md")
                .unwrap()
                .contains("\nby: Maks\ntaken-by: Grace\n")
        );
        assert!(
            storage
                .read("tasks/task.planned-for-me.md")
                .unwrap()
                .contains("\nby: Maks\ntaken-by: Maks\n")
        );
        assert_reply(
            ok(&mut storage, &["ready"]),
            serde_json::json!({
              "count": 3,
              "ready": [
                {
                  "id": "task.planned-for-grace",
                  "priority": null,
                  "created": "2026-08-28",
                  "taken-by": "Grace",
                  "title": "Planned for Grace"
                },
                {
                  "id": "task.planned-for-me",
                  "priority": null,
                  "created": "2026-08-28",
                  "taken-by": "Maks",
                  "title": "Planned for me"
                },
                {
                  "id": "task.planned-for-nobody",
                  "priority": null,
                  "created": "2026-08-28",
                  "taken-by": null,
                  "title": "Planned for nobody"
                }
              ]
            }),
        );
    }

    /// A host that knows nobody has nobody to take the Task for; the
    /// refusal names the fix.
    #[test]
    fn add_task_mine_without_an_identity_is_a_recovery_payload() {
        let mut storage = MemoryStorage::new();
        let line = ["anb", "add", "task", "Planned for me", "--mine"];
        let cli = Cli::try_parse_from(line).unwrap();
        let error = execute(cli.command, &mut storage, anonymous_host()).unwrap_err();
        assert_reply(
            text::render_error(
                &error,
                &anb::recovery::subject(&Cli::try_parse_from(line).unwrap().command),
            ),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "mine: no identity to take the task for; set git user.name or ANB_BY",
              "try": [
                "anb add task \"<title>\""
              ]
            }),
        );
        assert!(
            storage.list("tasks").unwrap().is_empty(),
            "nothing was written"
        );
    }

    /// Whether the value is an integer is the command line\'s question;
    /// whether it is a priority is the notebook\'s, and both answers reach
    /// the caller in one shape.
    #[test]
    fn a_priority_outside_the_scale_is_refused_the_same_way_at_any_size() {
        for out_of_range in ["9", "300"] {
            assert_reply(
                refused(
                    &mut MemoryStorage::new(),
                    &["add", "task", "A triaged task", "--priority", out_of_range],
                ),
                serde_json::json!({"error":"invalid-argument", "message": format!("priority: {out_of_range} is outside 0 to 4"), "try":["anb add task \"<title>\""]}),
            );
        }
    }

    /// The refusals a mistyped command earns, each naming the argument it
    /// judged: the shape of an id, of a tag, of a link, of a date.
    #[test]
    fn a_malformed_argument_is_refused_before_any_byte_moves() {
        for (line, reason) in [
            (
                vec!["start", "not-an-id"],
                "id: `not-an-id` is not `<type>.<slug>`",
            ),
            (
                vec!["add", "task", "A tagged task", "--tag", "Bad Tag"],
                "tags: `Bad Tag` is not a `[a-z0-9-]+` tag",
            ),
            (
                vec!["add", "task", "A linked task", "--link", "foo"],
                "link: `foo` is not `<kind> <target>`",
            ),
            (
                vec!["add", "task", "A note by another name", "--id", "note.demo"],
                "id: `note.demo` names a note, the draft is a task",
            ),
            (
                vec!["add", "task", "???"],
                "title: yields an empty id; pass an explicit id",
            ),
        ] {
            let mut storage = MemoryStorage::new();
            let refusal = refused(&mut storage, &line);
            assert_reply(
                refusal,
                serde_json::json!({"error":"invalid-argument", "message":reason}),
            );
            assert!(
                storage.list("tasks").unwrap().is_empty(),
                "`anb {}` wrote a record it had refused",
                line.join(" ")
            );
        }
    }

    #[test]
    fn a_hold_until_that_is_not_a_date_is_refused() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(
                &mut storage,
                &[
                    "hold",
                    "task.demo",
                    "--reason",
                    "waiting",
                    "--until",
                    "soon",
                ],
            ),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "hold-until: `soon` is not `YYYY-MM-DD` or an RFC 3339 timestamp",
              "try": [
                "anb hold task.demo --reason \"<why>\""
              ]
            }),
        );
    }

    #[test]
    fn start_answers_the_transition() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(&mut storage, &["start", "task.demo"]),
            serde_json::json!({
              "ok": "start",
              "id": "task.demo",
              "from": "open",
              "to": "active"
            }),
        );
    }

    #[test]
    fn a_replay_names_the_standing_state() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        ok(&mut storage, &["start", "task.demo"]);
        assert_reply(
            ok(&mut storage, &["start", "task.demo"]),
            serde_json::json!({
              "ok": "start",
              "id": "task.demo",
              "already": true,
              "to": "active"
            }),
        );
    }

    #[test]
    fn close_names_its_computed_consequences() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md".to_owned(),
                record_file("task.demo", "task", "active", "A demo record", &[], ""),
            ),
            open_task(
                "task.waiting",
                "A blocked record",
                &["blocked-by: task.demo"],
            ),
            (
                "questions/question.doubt.md".to_owned(),
                record_file(
                    "question.doubt",
                    "question",
                    "open",
                    "A parked doubt",
                    &["from: task.demo"],
                    "",
                ),
            ),
        ]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "close",
                    "task.demo",
                    "--body",
                    "Completed in https://example.com/pull/7",
                ],
            ),
            serde_json::json!({
              "ok": "close",
              "id": "task.demo",
              "from": "active",
              "to": "closed",
              "unblocked": {
                "count": 1,
                "rows": [
                  "task.waiting"
                ]
              },
              "open-questions": {
                "count": 1,
                "rows": [
                  "question.doubt"
                ]
              }
            }),
        );
    }

    #[test]
    fn close_with_an_outcome_file_signs_the_task_body() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_reply(
            run_reading(
                &mut storage,
                &["close", "task.demo", "--body-file", "reports/demo.md"],
                &[("reports/demo.md", "# What shipped\n")],
            )
            .expect("the command must succeed"),
            serde_json::json!({
              "ok": "close",
              "id": "task.demo",
              "from": "active",
              "to": "closed"
            }),
        );
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .ends_with("- 2026-08-28 Maks: # What shipped\n"),
            "the outcome is signed by whoever closed the Task"
        );
        assert!(
            anb_core::Storage::list(&storage, "notes")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn close_with_a_missing_outcome_file_is_a_recovery_payload() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_reply(
            run_reading(
                &mut storage,
                &["close", "task.demo", "--body-file", "gone.md"],
                &[],
            )
            .expect_err("the command must be refused"),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "body-file: no file at `gone.md`",
              "try": [
                "anb close task.demo --body \"<outcome>\"",
                "anb close task.demo --reason \"<why>\""
              ]
            }),
        );
        assert_eq!(
            storage.read("tasks/task.demo.md").unwrap(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
            "an unreadable report closes nothing"
        );
    }

    #[test]
    fn close_with_both_an_outcome_and_a_cancellation_names_the_conflict() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_reply(
            refused(
                &mut storage,
                &[
                    "close",
                    "task.demo",
                    "--body",
                    "Completed and checked.",
                    "--reason",
                    "Cancelled",
                ],
            ),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "close: pass exactly one of --body \"<outcome>\", --body-file <path>, --reason \"<why>\", or --resolved-by <id>",
              "try": [
                "anb close task.demo --body \"<outcome>\"",
                "anb close task.demo --reason \"<why>\""
              ]
            }),
        );
    }

    #[test]
    fn close_without_an_outcome_is_a_recovery_payload() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_reply(
            refused(&mut storage, &["close", "task.demo"]),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "close: pass one of --body \"<outcome>\", --body-file <path>, --reason \"<why>\", or --resolved-by <id>",
              "try": [
                "anb close task.demo --body \"<outcome>\"",
                "anb close task.demo --reason \"<why>\""
              ]
            }),
        );
    }

    #[test]
    fn an_invalid_transition_lists_the_valid_commands() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(
                &mut storage,
                &["close", "task.demo", "--body", "Completed and checked."],
            ),
            serde_json::json!({
              "error": "invalid-transition",
              "message": "`task.demo` is open; valid: start, close --reason",
              "try": [
                "anb start task.demo",
                "anb close task.demo --reason \"<why>\""
              ]
            }),
        );
    }

    #[test]
    fn a_close_outcome_shows_among_the_valid_commands() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "review", "A demo record", &[], ""),
        )]);
        assert_reply(
            refused(&mut storage, &["reopen", "task.demo"]),
            serde_json::json!({
              "error": "invalid-transition",
              "message": "`task.demo` is review; valid: start, close, close --reason",
              "try": [
                "anb start task.demo",
                "anb close task.demo --body \"<outcome>\"",
                "anb close task.demo --reason \"<why>\""
              ]
            }),
        );
    }

    #[test]
    fn an_edge_that_would_cycle_walks_the_chain() {
        let mut storage = storage_with(&[
            open_task("task.a", "The first", &["blocked-by: task.b"]),
            open_task("task.b", "The second", &[]),
        ]);
        assert_reply(
            refused(&mut storage, &["block", "task.b", "task.a"]),
            serde_json::json!({
              "error": "would-cycle",
              "message": "the edge would close a dependency cycle: task.b → task.a → task.b",
              "try": [
                "anb unblock task.a task.b"
              ]
            }),
        );
    }

    #[test]
    fn block_and_unblock_answer_the_edge() {
        let mut storage = storage_with(&[
            open_task("task.a", "The first", &[]),
            open_task("task.b", "The second", &[]),
        ]);
        assert_reply(
            ok(&mut storage, &["block", "task.a", "task.b"]),
            serde_json::json!({"ok":"block", "id":"task.a", "on":"task.b", "already":false}),
        );
        assert_reply(
            ok(&mut storage, &["unblock", "task.a", "task.b"]),
            serde_json::json!({"ok":"unblock", "id":"task.a", "on":"task.b", "already":false}),
        );
    }

    #[test]
    fn a_replayed_unblock_marks_already() {
        let mut storage = storage_with(&[open_task("task.a", "The first", &[])]);
        assert_reply(
            ok(&mut storage, &["unblock", "task.a", "task.b"]),
            serde_json::json!({"ok":"unblock", "id":"task.a", "on":"task.b", "already":true}),
        );
    }

    #[test]
    fn a_citation_into_nothing_is_nudged_under_the_ok_line() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "comment",
                    "task.demo",
                    "waits on task.ghost and task.wraith",
                ],
            ),
            serde_json::json!({
              "ok": "comment",
              "id": "task.demo",
              "dangling-mention": {
                "count": 2,
                "rows": [
                  "task.ghost",
                  "task.wraith"
                ]
              }
            }),
        );
    }

    #[test]
    fn an_add_body_citing_nothing_is_nudged_the_same_way() {
        let mut storage = MemoryStorage::new();
        assert_reply(
            ok(
                &mut storage,
                &[
                    "add",
                    "task",
                    "A demo record",
                    "--body",
                    "Blocked by task.ghost.",
                ],
            ),
            serde_json::json!({
              "ok": "add",
              "id": "task.a-demo-record",
              "path": "tasks/task.a-demo-record.md",
              "dangling-mention": {
                "count": 1,
                "rows": [
                  "task.ghost"
                ]
              }
            }),
        );
    }

    #[test]
    fn an_invalid_record_names_its_findings() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "cancelled", "A demo record", &[], ""),
        )]);
        assert_reply(
            refused(&mut storage, &["start", "task.demo"]),
            serde_json::json!({
              "error": "invalid-record",
              "message": "tasks/task.demo.md is invalid (1 finding)",
              "try": [
                "anb show task.demo"
              ],
              "findings": [
                "line 4: bad-value state: `cancelled` is not one of open, active, review, closed for a task"
              ]
            }),
        );
    }

    /// A refusal's detail lines are the reason it refused. The data
    /// rendering carries them, or an agent parsing it is told only that
    /// something was wrong.
    #[test]
    fn a_json_refusal_carries_its_detail_lines() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "cancelled", "A demo record", &[], ""),
        )]);
        let refusal: serde_json::Value =
            serde_json::from_str(&refused(&mut storage, &["start", "task.demo", "--json"]))
                .unwrap();
        assert_eq!(refusal["error"], serde_json::json!("invalid-record"));
        let findings = refusal["findings"].as_array().unwrap();
        assert_eq!(findings.len(), 1);
        assert!(
            findings[0].as_str().unwrap().contains("bad-value"),
            "{findings:?}"
        );
    }

    #[test]
    fn an_archived_record_suggests_viewing_it_and_the_way_back() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.done.md".to_owned(),
            record_file("task.done", "task", "closed", "Shipped work", &[], ""),
        )]);
        assert_reply(
            refused(&mut storage, &["start", "task.done"]),
            serde_json::json!({
              "error": "archived",
              "message": "`task.done` is archived",
              "try": [
                "anb show task.done",
                "anb restore task.done"
              ]
            }),
        );
    }

    #[test]
    fn a_wrong_typed_target_suggests_viewing_it() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(&mut storage, &["block", "task.demo", "decision.d"]),
            serde_json::json!({
              "error": "wrong-type",
              "message": "`decision.d` is not a task",
              "try": [
                "anb show decision.d"
              ]
            }),
        );
    }

    #[test]
    fn block_on_a_missing_task_suggests_creating_it() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(&mut storage, &["block", "task.demo", "task.ghost"]),
            serde_json::json!({
              "error": "dangling-ref",
              "message": "blocked-by: `task.ghost` names no record",
              "try": [
                "anb add task \"<title>\" --id task.ghost",
                "anb list"
              ]
            }),
        );
    }

    #[test]
    fn comment_logs_with_the_acting_hand() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "comment",
                    "task.demo",
                    "parser done, tests next",
                    "--via",
                    "claude-code",
                ],
            ),
            serde_json::json!({
              "ok": "comment",
              "id": "task.demo"
            }),
        );
        let written = anb_core::Storage::read(&storage, "tasks/task.demo.md").unwrap();
        assert!(
            written.ends_with("- 2026-08-28 Maks/claude-code: parser done, tests next\n"),
            "{written}"
        );
    }

    /// The refusal names who took the Task and offers the hand-over as a
    /// template: it is decided by a person, never filled in.
    #[test]
    fn starting_another_persons_task_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task(
            "task.demo",
            "A demo record",
            &["taken-by: Grace"],
        )]);
        assert_reply(
            refused(&mut storage, &["start", "task.demo"]),
            serde_json::json!({
              "error": "taken",
              "message": "`task.demo` is taken by Grace",
              "try": [
                "anb edit task.demo --taken-by \"<name>\"",
                "anb show task.demo"
              ]
            }),
        );
        assert_reply(
            refused(&mut storage, &["start", "task.demo", "--json"]),
            serde_json::json!({
              "error": "taken",
              "message": "`task.demo` is taken by Grace",
              "try": [
                "anb edit task.demo --taken-by \"<name>\"",
                "anb show task.demo"
              ]
            }),
        );
    }

    #[test]
    fn a_comment_without_via_signs_as_the_identity() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        ok(&mut storage, &["comment", "task.demo", "stopped here"]);
        let written = anb_core::Storage::read(&storage, "tasks/task.demo.md").unwrap();
        assert!(
            written.ends_with("- 2026-08-28 Maks: stopped here\n"),
            "{written}"
        );
    }

    #[test]
    fn hold_and_unhold_answer_their_outcome() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "hold",
                    "task.demo",
                    "--reason",
                    "waiting for the release",
                    "--until",
                    "2026-09-01",
                ],
            ),
            serde_json::json!({"ok":"hold", "id":"task.demo", "until":"2026-09-01", "already":false}),
        );
        assert_reply(
            ok(&mut storage, &["unhold", "task.demo"]),
            serde_json::json!({"ok":"unhold", "id":"task.demo", "already":false}),
        );
    }

    #[test]
    fn hold_without_a_reason_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(&mut storage, &["hold", "task.demo"]),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "hold: the reason must not be empty",
              "try": [
                "anb hold task.demo --reason \"<why>\""
              ]
            }),
        );
    }

    #[test]
    fn a_taken_id_suggests_the_next_commands() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(
                &mut storage,
                &["add", "task", "Another demo", "--id", "task.demo"],
            ),
            serde_json::json!({
              "error": "duplicate-id",
              "message": "`task.demo` already exists at tasks/task.demo.md",
              "try": [
                "anb show task.demo",
                "anb add task \"<title>\""
              ]
            }),
        );
    }

    #[test]
    fn an_unknown_id_is_a_recovery_payload() {
        let mut storage = MemoryStorage::new();
        assert_reply(
            refused(&mut storage, &["start", "task.absent"]),
            serde_json::json!({
              "error": "unknown-id",
              "message": "no record `task.absent`",
              "try": [
                "anb list"
              ]
            }),
        );
    }
}

mod knowledge_replies {
    use super::*;

    /// A Note supersedes as a Decision does, so its reply names the record
    /// it replaced the same way — the consequence a caller must not have to
    /// go looking for.
    #[test]
    fn a_note_that_supersedes_names_the_one_it_replaces() {
        let mut storage = storage_with(&[(
            "notes/note.old.md".to_owned(),
            record_file("note.old", "note", "active", "The old note", &[], ""),
        )]);
        assert_reply(
            ok(
                &mut storage,
                &["add", "note", "The new note", "--supersedes", "note.old"],
            ),
            serde_json::json!({
              "ok": "add",
              "id": "note.the-new-note",
              "path": "notes/note.the-new-note.md",
              "superseded": "note.old"
            }),
        );
    }

    /// `--resolved-by` closes a Question into the record that settled it and
    /// `--reason` ends it without one; passing both leaves nothing to choose
    /// between, and silently keeping one would discard the other's words.
    #[test]
    fn closing_a_question_with_both_a_resolver_and_a_reason_names_the_conflict() {
        let mut storage = storage_with(&[
            (
                "questions/question.doubt.md".to_owned(),
                record_file("question.doubt", "question", "open", "A doubt", &[], ""),
            ),
            (
                "decisions/decision.settled.md".to_owned(),
                record_file("decision.settled", "decision", "active", "Settled", &[], ""),
            ),
        ]);
        assert_reply(
            refused(
                &mut storage,
                &[
                    "close",
                    "question.doubt",
                    "--resolved-by",
                    "decision.settled",
                    "--reason",
                    "not worth it",
                ],
            ),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "close: pass exactly one of --body \"<outcome>\", --body-file <path>, --reason \"<why>\", or --resolved-by <id>",
              "try": [
                "anb close question.doubt --resolved-by <id>",
                "anb close question.doubt --reason \"<why>\""
              ]
            }),
        );
        assert!(
            storage
                .read("questions/question.doubt.md")
                .unwrap()
                .contains("state: open"),
            "a refused close closes nothing"
        );
    }
    use anb_core::Storage as _;

    #[test]
    fn add_decision_records_a_decision_and_answers_the_path() {
        let mut storage = MemoryStorage::new();
        assert_reply(
            ok(
                &mut storage,
                &["add", "decision", "No mise toml", "--kind", "rule"],
            ),
            serde_json::json!({
              "ok": "add",
              "id": "decision.no-mise-toml",
              "path": "decisions/decision.no-mise-toml.md"
            }),
        );
        let written = storage.read("decisions/decision.no-mise-toml.md").unwrap();
        assert!(written.contains("\nstate: active\n"), "{written}");
        assert!(written.contains("\nkind: rule\n"), "{written}");
    }

    #[test]
    fn add_decision_with_supersedes_names_the_replaced_decision() {
        let mut storage = storage_with(&[(
            "decisions/decision.go-for-the-cli.md".to_owned(),
            record_file(
                "decision.go-for-the-cli",
                "decision",
                "active",
                "Go for the CLI",
                &[],
                "",
            ),
        )]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "add",
                    "decision",
                    "Rust for the CLI",
                    "--supersedes",
                    "decision.go-for-the-cli",
                ],
            ),
            serde_json::json!({
              "ok": "add",
              "id": "decision.rust-for-the-cli",
              "path": "decisions/decision.rust-for-the-cli.md",
              "superseded": "decision.go-for-the-cli"
            }),
        );
    }

    #[test]
    fn shared_tags_do_not_imply_conflicting_decisions() {
        let mut storage = storage_with(&[(
            "decisions/decision.first.md".to_owned(),
            record_file(
                "decision.first",
                "decision",
                "active",
                "The first ruling",
                &["by: supolka", "via: claude-code", "tags: parser, grammar"],
                "",
            ),
        )]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "add",
                    "decision",
                    "Fences stay",
                    "--tag",
                    "parser",
                    "--tag",
                    "grammar",
                ],
            ),
            serde_json::json!({
              "ok": "add",
              "id": "decision.fences-stay",
              "path": "decisions/decision.fences-stay.md",
              "may-conflict": null
            }),
        );
        assert!(
            storage.read("decisions/decision.fences-stay.md").is_ok(),
            "shared tags do not prevent a new decision"
        );
    }

    #[test]
    fn add_note_records_a_term() {
        let mut storage = MemoryStorage::new();
        assert_reply(
            ok(&mut storage, &["add", "note", "Record", "--kind", "term"]),
            serde_json::json!({
              "ok": "add",
              "id": "note.record",
              "path": "notes/note.record.md"
            }),
        );
        let written = storage.read("notes/note.record.md").unwrap();
        assert!(written.contains("\nkind: term\n"), "{written}");
    }

    #[test]
    fn add_takes_the_body_from_a_file() {
        let mut storage = MemoryStorage::new();
        let model = "# The operation library\n\nAn operation is reviewed before it is published.\n";
        run_reading(
            &mut storage,
            &[
                "add",
                "note",
                "The operation library",
                "--kind",
                "model",
                "--body-file",
                "model.md",
            ],
            &[("model.md", model)],
        )
        .expect("the command must succeed");
        let written = storage.read("notes/note.the-operation-library.md").unwrap();
        assert!(
            written.ends_with(model),
            "the file's text is the body, as --body would have landed it: {written}"
        );
    }

    #[test]
    fn a_body_file_that_is_not_there_is_a_recovery_payload() {
        let mut storage = MemoryStorage::new();
        assert_reply(
            run(
                &mut storage,
                &[
                    "add",
                    "note",
                    "A fact",
                    "--kind",
                    "fact",
                    "--body-file",
                    "missing.md",
                ],
            )
            .expect_err("the command must be refused"),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "body-file: no file at `missing.md`",
              "try": [
                "anb add note \"<title>\""
              ]
            }),
        );
        assert!(
            storage.list("notes").unwrap().is_empty(),
            "nothing is written"
        );
    }

    #[test]
    fn a_body_beside_a_body_file_is_refused_by_the_command_line() {
        let parsed = Cli::try_parse_from([
            "anb",
            "add",
            "note",
            "A fact",
            "--body",
            "inline",
            "--body-file",
            "fact.md",
        ]);
        let Err(error) = parsed else {
            panic!("the two flags exclude each other")
        };
        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn a_kind_outside_the_types_enum_is_a_recovery_payload() {
        let mut storage = MemoryStorage::new();
        assert_reply(
            refused(&mut storage, &["add", "note", "A fact", "--kind", "law"]),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "kind: `law` is not one of fact, term, guide, idea, model, spec for a note",
              "try": [
                "anb add note \"<title>\""
              ]
            }),
        );
    }

    #[test]
    fn add_question_files_a_question_with_its_origin() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "add",
                    "question",
                    "Does the parser keep fences?",
                    "--from",
                    "task.demo",
                ],
            ),
            serde_json::json!({
              "ok": "add",
              "id": "question.does-the-parser-keep-fences",
              "path": "questions/question.does-the-parser-keep-fences.md"
            }),
        );
        let written = storage
            .read("questions/question.does-the-parser-keep-fences.md")
            .unwrap();
        assert!(written.contains("\nstate: open\n"), "{written}");
        assert!(written.contains("\nfrom: task.demo\n"), "{written}");
    }

    #[test]
    fn add_with_a_link_naming_no_record_is_a_recovery_payload() {
        let mut storage = MemoryStorage::new();
        assert_reply(
            refused(
                &mut storage,
                &[
                    "add",
                    "decision",
                    "Fences never nest",
                    "--kind",
                    "rule",
                    "--link",
                    "within decision.ghost",
                ],
            ),
            serde_json::json!({
              "error": "dangling-ref",
              "message": "link: `decision.ghost` names no record",
              "try": [
                "anb list"
              ]
            }),
        );
        assert!(
            storage.list("decisions").unwrap().is_empty(),
            "nothing is written"
        );
    }

    #[test]
    fn add_from_a_missing_origin_is_a_recovery_payload() {
        let mut storage = MemoryStorage::new();
        assert_reply(
            refused(
                &mut storage,
                &["add", "question", "A doubt", "--from", "task.ghost"],
            ),
            serde_json::json!({
              "error": "dangling-ref",
              "message": "from: `task.ghost` names no record",
              "try": [
                "anb add task \"<title>\" --id task.ghost",
                "anb list"
              ]
            }),
        );
    }

    /// A dangling reference to a record no `add` shape is offered for gets
    /// no creating command; only a missing Task is worth minting on the spot.
    #[test]
    fn a_dangling_reference_to_another_type_offers_no_creating_command() {
        let mut storage = MemoryStorage::new();
        assert_reply(
            refused(
                &mut storage,
                &["add", "question", "A doubt", "--from", "note.ghost"],
            ),
            serde_json::json!({
              "error": "dangling-ref",
              "message": "from: `note.ghost` names no record",
              "try": [
                "anb list"
              ]
            }),
        );
    }

    #[test]
    fn close_resolved_by_closes_the_question_into_the_record_that_settled_it() {
        let mut storage = storage_with(&[
            (
                "questions/question.doubt.md".to_owned(),
                record_file("question.doubt", "question", "open", "A doubt", &[], ""),
            ),
            (
                "decisions/decision.ruling.md".to_owned(),
                record_file("decision.ruling", "decision", "active", "A ruling", &[], ""),
            ),
        ]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "close",
                    "question.doubt",
                    "--resolved-by",
                    "decision.ruling",
                ],
            ),
            serde_json::json!({
              "ok": "close",
              "id": "question.doubt",
              "from": "open",
              "to": "closed",
              "resolved-by": "decision.ruling"
            }),
        );
        let written = storage.read("questions/question.doubt.md").unwrap();
        assert!(written.contains("\nstate: closed\n"), "{written}");
        assert!(
            written.contains("\nresolved-by: decision.ruling\n"),
            "{written}"
        );
    }

    #[test]
    fn close_reason_ends_the_question_with_the_reason_in_its_envelope() {
        let mut storage = storage_with(&[(
            "questions/question.doubt.md".to_owned(),
            record_file("question.doubt", "question", "open", "A doubt", &[], ""),
        )]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "close",
                    "question.doubt",
                    "--reason",
                    "overtaken by the rewrite",
                ],
            ),
            serde_json::json!({
              "ok": "close",
              "id": "question.doubt",
              "from": "open",
              "to": "closed"
            }),
        );
        let written = storage.read("questions/question.doubt.md").unwrap();
        assert!(written.contains("\nstate: closed\n"), "{written}");
        assert!(
            written.contains("\nreason: overtaken by the rewrite\n"),
            "{written}"
        );
    }

    #[test]
    fn closing_a_question_without_an_outcome_is_a_recovery_payload() {
        let mut storage = storage_with(&[(
            "questions/question.doubt.md".to_owned(),
            record_file("question.doubt", "question", "open", "A doubt", &[], ""),
        )]);
        assert_reply(
            refused(&mut storage, &["close", "question.doubt"]),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "close: pass one of --body \"<outcome>\", --body-file <path>, --reason \"<why>\", or --resolved-by <id>",
              "try": [
                "anb close question.doubt --resolved-by <id>",
                "anb close question.doubt --reason \"<why>\""
              ]
            }),
        );
    }

    #[test]
    fn retire_ends_a_decision_without_a_successor() {
        let mut storage = storage_with(&[(
            "decisions/decision.old-rule.md".to_owned(),
            record_file(
                "decision.old-rule",
                "decision",
                "active",
                "An old rule",
                &[],
                "",
            ),
        )]);
        assert_reply(
            ok(&mut storage, &["retire", "decision.old-rule"]),
            serde_json::json!({
              "ok": "retire",
              "id": "decision.old-rule",
              "from": "active",
              "to": "retired"
            }),
        );
    }
}

mod flat_lists {
    use super::*;

    /// The ready queue of the interaction contract's worked example: ages
    /// derive from `created` against today, order from priority then age.
    fn worked_example() -> MemoryStorage {
        storage_with(&[
            (
                "tasks/task.parser-fences.md".to_owned(),
                record_file(
                    "task.parser-fences",
                    "task",
                    "open",
                    "Grammar parser accepts fenced envelopes",
                    &["priority: 1"],
                    "",
                )
                .replace("created: 2026-08-24", "created: 2026-08-26"),
            ),
            (
                "tasks/task.check-corpus.md".to_owned(),
                record_file(
                    "task.check-corpus",
                    "task",
                    "open",
                    "Negative corpus wired into CI",
                    &["priority: 2"],
                    "",
                )
                .replace("created: 2026-08-24", "created: 2026-08-19"),
            ),
            (
                "tasks/task.status-budget.md".to_owned(),
                record_file(
                    "task.status-budget",
                    "task",
                    "open",
                    "Status degrades sections, keeps Budget",
                    &["priority: 2"],
                    "",
                ),
            ),
        ])
    }

    #[test]
    fn ready_ranks_and_ages_the_rows() {
        assert_reply(
            ok(&mut worked_example(), &["ready"]),
            serde_json::json!({
              "count": 3,
              "ready": [
                {
                  "id": "task.parser-fences",
                  "priority": 1,
                  "created": "2026-08-26",
                  "taken-by": null,
                  "title": "Grammar parser accepts fenced envelopes"
                },
                {
                  "id": "task.check-corpus",
                  "priority": 2,
                  "created": "2026-08-19",
                  "taken-by": null,
                  "title": "Negative corpus wired into CI"
                },
                {
                  "id": "task.status-budget",
                  "priority": 2,
                  "created": "2026-08-24",
                  "taken-by": null,
                  "title": "Status degrades sections, keeps Budget"
                }
              ]
            }),
        );
    }

    /// A reader picking work sees who a Task is taken before the
    /// pick; a Task nobody holds shows `-`.
    #[test]
    fn the_queue_names_who_a_task_is_spoken_for() {
        let mut storage = storage_with(&[
            open_task("task.free", "Open to anyone", &[]),
            open_task("task.hers", "Already taken", &["taken-by: Grace"]),
        ]);
        assert_reply(
            ok(&mut storage, &["ready"]),
            serde_json::json!({
              "count": 2,
              "ready": [
                {
                  "id": "task.free",
                  "priority": null,
                  "created": "2026-08-24",
                  "taken-by": null,
                  "title": "Open to anyone"
                },
                {
                  "id": "task.hers",
                  "priority": null,
                  "created": "2026-08-24",
                  "taken-by": "Grace",
                  "title": "Already taken"
                }
              ]
            }),
        );
    }

    /// `--mine` is `--by` with the identity the writers sign with, so the
    /// hint that lifts the bound spells the name out and stays runnable.
    #[test]
    fn mine_narrows_to_the_callers_own_and_the_hint_carries_the_name() {
        let mut files: Vec<(String, String)> = (0..22)
            .map(|n| {
                open_task(
                    &format!("task.m{n:02}"),
                    "A demo record",
                    &["taken-by: Maks"],
                )
            })
            .collect();
        files.push(open_task(
            "task.hers",
            "Somebody else's",
            &["taken-by: Grace"],
        ));
        let mut storage = storage_with(&files);
        let mine = ok(&mut storage, &["ready", "--mine"]);
        assert_reply(
            &mine,
            serde_json::json!({"by":"Maks", "count":22,"omitted":2,"more":"anb ready --by Maks --all"}),
        );
        assert!(!mine.contains("task.hers"), "{mine}");
        let lifted = ok(&mut storage, &["ready", "--by", "Maks", "--all"]);
        assert_eq!(reply_value(lifted)["ready"].as_array().unwrap().len(), 22);
    }

    #[test]
    fn a_name_with_a_space_travels_in_the_hint_as_one_shell_word() {
        let files: Vec<(String, String)> = (0..21)
            .map(|n| {
                open_task(
                    &format!("task.g{n:02}"),
                    "A demo record",
                    &["taken-by: Grace Hopper"],
                )
            })
            .collect();
        let mut storage = storage_with(&files);
        let listed = ok(&mut storage, &["list", "--by", "Grace Hopper"]);
        assert_reply(
            listed,
            serde_json::json!({"more":"anb list --by 'Grace Hopper' --all", "count":21,"omitted":1}),
        );
    }

    /// An empty answer would read as "nothing is yours"; a host that knows
    /// nobody has nothing to narrow by and says so.
    #[test]
    fn mine_without_an_identity_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        let cli = Cli::try_parse_from(["anb", "list", "--mine"]).unwrap();
        let error = execute(cli.command, &mut storage, anonymous_host()).unwrap_err();
        assert_reply(
            text::render_error(
                &error,
                &anb::recovery::subject(
                    &Cli::try_parse_from(["anb", "list", "--mine"])
                        .unwrap()
                        .command,
                ),
            ),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "mine: no identity to match; set git user.name or ANB_BY, or pass --team",
              "try": []
            }),
        );
    }

    #[test]
    fn a_bounded_list_hints_its_truncation() {
        let mut storage = many_open_tasks(22);

        let bounded = ok(&mut storage, &["ready"]);
        assert_reply(
            &bounded,
            serde_json::json!({"count":22,"omitted":2,"more":"anb ready --all"}),
        );
        assert_eq!(reply_value(bounded)["ready"].as_array().unwrap().len(), 20);

        let unbounded = ok(&mut storage, &["ready", "--all"]);
        assert!(unbounded.contains("ready[22]{"), "{unbounded}");
        assert!(!unbounded.contains("more:"), "{unbounded}");
    }

    #[test]
    fn list_shows_every_live_type() {
        let mut storage = storage_with(&[
            open_task("task.demo", "A demo record", &[]),
            (
                "decisions/decision.why-rust.md".to_owned(),
                record_file(
                    "decision.why-rust",
                    "decision",
                    "active",
                    "Rust for the CLI",
                    &[],
                    "",
                ),
            ),
            (
                "questions/question.doubt.md".to_owned(),
                record_file(
                    "question.doubt",
                    "question",
                    "open",
                    "What, exactly?",
                    &[],
                    "",
                ),
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["list"]),
            serde_json::json!({
              "count": 3,
              "records": [
                {
                  "id": "task.demo",
                  "state": "open",
                  "priority": null,
                  "title": "A demo record"
                },
                {
                  "id": "decision.why-rust",
                  "state": "active",
                  "priority": null,
                  "title": "Rust for the CLI"
                },
                {
                  "id": "question.doubt",
                  "state": "open",
                  "priority": null,
                  "title": "What, exactly?"
                }
              ]
            }),
        );
    }
}

mod single_record {
    use super::*;

    fn viewed_storage() -> MemoryStorage {
        storage_with(&[
            (
                "tasks/task.demo.md".to_owned(),
                record_file(
                    "task.demo",
                    "task",
                    "active",
                    "A demo record",
                    &["priority: 1"],
                    "The plan follows decision.chosen.\n\n- 2026-08-25 Maks: started\n",
                ),
            ),
            (
                "decisions/decision.chosen.md".to_owned(),
                record_file(
                    "decision.chosen",
                    "decision",
                    "active",
                    "The chosen shape",
                    &[],
                    "Applies to task.demo.\n",
                ),
            ),
        ])
    }

    fn log_entries(entries: std::ops::RangeInclusive<usize>) -> String {
        use std::fmt::Write as _;
        entries.fold(String::new(), |mut body, entry| {
            writeln!(body, "- entry {entry}").unwrap();
            body
        })
    }

    fn logged_task(entries: usize) -> MemoryStorage {
        let body = log_entries(1..=entries);
        storage_with(&[(
            "tasks/task.long.md".to_owned(),
            record_file("task.long", "task", "active", "A demo record", &[], &body),
        )])
    }

    #[test]
    fn an_archived_view_names_its_residence() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.done.md".to_owned(),
            record_file("task.done", "task", "closed", "A finished task", &[], ""),
        )]);
        assert_reply(
            ok(&mut storage, &["show", "task.done"]),
            serde_json::json!({"id":"task.done","path":"archive/tasks/task.done.md","archived":true}),
        );
    }

    #[test]
    fn show_carries_ordered_fields_body_and_both_mention_directions() {
        for format in [
            vec!["show", "task.demo"],
            vec!["show", "task.demo", "--json"],
        ] {
            assert_reply(
                ok(&mut viewed_storage(), &format),
                serde_json::json!({
                    "id":"task.demo", "path":"tasks/task.demo.md", "archived":false,
                    "fields":{"count":7,"omitted":0,"rows":[["id","task.demo"],["type","task"],["state","active"],["title","A demo record"],["priority","1"],["created","2026-08-24"],["updated","2026-08-25"]]},
                    "body":{"lines":3,"head":"The plan follows decision.chosen.\n\n- 2026-08-25 Maks: started\n","omitted":0},
                    "mentions":{"count":1,"omitted":0,"rows":["decision.chosen"]},
                    "mentioned-by":{"count":1,"omitted":0,"rows":["decision.chosen"]},
                    "linked-by":{"count":0,"omitted":0,"rows":[]}
                }),
            );
        }
    }

    #[test]
    fn incoming_links_keep_their_kind() {
        let mut storage = storage_with(&[
            (
                "notes/note.schema.md".to_owned(),
                record_file(
                    "note.schema",
                    "note",
                    "active",
                    "The schema",
                    &["kind: spec"],
                    "",
                ),
            ),
            (
                "notes/note.credits.md".to_owned(),
                record_file(
                    "note.credits",
                    "note",
                    "active",
                    "Credits",
                    &["kind: term", "link: schema note.schema"],
                    "",
                ),
            ),
        ]);
        for format in [
            vec!["show", "note.schema"],
            vec!["show", "note.schema", "--json"],
        ] {
            assert_reply(
                ok(&mut storage, &format),
                serde_json::json!({"linked-by":{"count":1,"omitted":0,"rows":[{"id":"note.credits","kind":"schema"}]}}),
            );
        }
    }

    #[test]
    fn a_long_envelope_and_value_are_bounded_and_all_restores_them() {
        let edges = (0..80)
            .map(|n| format!("blocked-by: task.c{n:03}"))
            .collect::<Vec<_>>();
        let lines = edges.iter().map(String::as_str).collect::<Vec<_>>();
        let title = "word ".repeat(400);
        let mut storage = storage_with(&[(
            "tasks/task.hub.md".to_owned(),
            record_file("task.hub", "task", "open", &title, &lines, ""),
        )]);
        let bounded = reply_value(ok(&mut storage, &["show", "task.hub"]));
        assert_eq!(bounded["more"], "anb show task.hub --all");
        assert!(bounded["fields"]["omitted"].as_u64().unwrap() > 0);
        assert!(bounded["fields"]["rows"][3][1].as_str().unwrap().len() < title.len());
        let whole = reply_value(ok(&mut storage, &["show", "task.hub", "--all"]));
        assert_eq!(whole["fields"]["omitted"], 0);
        assert_eq!(whole["fields"]["rows"][3][1], title.trim_end());
        assert!(
            whole["fields"]["rows"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!(["blocked-by", "task.c079"]))
        );
    }

    #[test]
    fn a_long_body_preserves_both_ends_and_reports_omitted_characters() {
        for format in [
            vec!["show", "task.long"],
            vec!["show", "task.long", "--json"],
        ] {
            let value = reply_value(ok(&mut logged_task(60), &format));
            let body = &value["body"];
            assert_eq!(body["lines"], 60);
            assert_eq!(body["head"], log_entries(1..=20));
            assert_eq!(body["tail"], log_entries(41..=60));
            assert_eq!(
                body["omitted"],
                (21..=40)
                    .map(|entry| format!("- entry {entry}\n").chars().count())
                    .sum::<usize>()
            );
            assert_eq!(value["more"], "anb show task.long --all");
        }
    }

    #[test]
    fn all_restores_every_body_line_without_an_omission() {
        let value = reply_value(ok(&mut logged_task(60), &["show", "task.long", "--all"]));
        assert_eq!(value["body"]["head"], log_entries(1..=60));
        assert_eq!(value["body"]["omitted"], 0);
        assert!(value["body"].get("tail").is_none());
    }

    #[test]
    fn a_body_at_the_line_boundary_is_not_shortened() {
        let value = reply_value(ok(&mut logged_task(41), &["show", "task.long"]));
        assert_eq!(value["body"]["head"], log_entries(1..=41));
        assert_eq!(value["body"]["omitted"], 0);
    }
}
mod session_status {
    use super::*;

    fn active_task_storage() -> MemoryStorage {
        storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )])
    }

    /// A held Task is paused on purpose, so it is not the active line a
    /// session resumes from: it waits in its own section, reason and all,
    /// on both renderings.
    #[test]
    fn a_held_task_waits_in_its_own_section_not_among_the_active() {
        let mut storage = storage_with(&[
            (
                "tasks/task.parked.md".to_owned(),
                record_file(
                    "task.parked",
                    "task",
                    "active",
                    "A parked record",
                    &["hold: waits for the API key", "hold-until: 2026-09-20"],
                    "",
                ),
            ),
            (
                "tasks/task.demo.md".to_owned(),
                record_file("task.demo", "task", "active", "A demo record", &[], ""),
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["status", "--budget", "0"]),
            serde_json::json!({
              "counts": {
                "tasks": 2,
                "decisions": 0,
                "notes": 0,
                "questions": 0
              },
              "active": {
                "rows": [
                  {
                    "id": "task.demo",
                    "title": "A demo record"
                  }
                ]
              },
              "held": {
                "count": 1,
                "rows": [
                  {
                    "id": "task.parked",
                    "reason": "waits for the API key",
                    "until": "2026-09-20",
                    "taken-by": null
                  }
                ]
              }
            }),
        );
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["status", "--json"])).unwrap();
        assert_eq!(value["active"]["count"], serde_json::json!(1));
        assert_eq!(
            value["held"]["rows"][0],
            serde_json::json!({"id": "task.parked", "reason": "waits for the API key", "until": "2026-09-20"})
        );
    }

    /// The reader's own Task leads and carries the log line, whoever
    /// touched what last; the other person's line says whose it is, on
    /// both renderings.
    #[test]
    fn another_persons_active_line_carries_their_name_and_yields_the_lead() {
        let mut storage = storage_with(&[
            (
                "tasks/task.hers.md".to_owned(),
                record_file(
                    "task.hers",
                    "task",
                    "active",
                    "Her record",
                    &["taken-by: Grace"],
                    "- 2026-08-25 Grace: her stop\n",
                )
                .replace("updated: 2026-08-25", "updated: 2026-08-27"),
            ),
            (
                "tasks/task.mine.md".to_owned(),
                record_file(
                    "task.mine",
                    "task",
                    "active",
                    "My record",
                    &["taken-by: Maks"],
                    "- 2026-08-25 Maks: my stop\n",
                ),
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["status", "--budget", "0"]),
            serde_json::json!({
              "counts": {
                "tasks": 2,
                "decisions": 0,
                "notes": 0,
                "questions": 0
              },
              "active": {
                "rows": [
                  {
                    "id": "task.mine",
                    "title": "My record",
                    "log": "- 2026-08-25 Maks: my stop"
                  },
                  {
                    "id": "task.hers",
                    "title": "Her record",
                    "taken-by": "Grace"
                  }
                ]
              }
            }),
        );
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["status", "--json"])).unwrap();
        assert_fields(
            &value["active"]["rows"][1],
            serde_json::json!({"id": "task.hers", "title": "Her record", "taken-by": "Grace", "log":"- 2026-08-25 Grace: her stop"}),
        );
    }

    /// A review handed to the reader and a doubt put to them are the
    /// reader's turn: a narrowed dashboard opens on them, each row naming
    /// whom it waits on, on both renderings; what waits on somebody else
    /// stays out.
    #[test]
    fn what_waits_on_the_reader_opens_their_narrowed_dashboard() {
        let mut storage = storage_with(&[
            (
                "tasks/task.hers-for-me.md".to_owned(),
                record_file(
                    "task.hers-for-me",
                    "task",
                    "review",
                    "Her record",
                    &["taken-by: Grace", "to: Maks"],
                    "",
                ),
            ),
            (
                "tasks/task.hers-for-her.md".to_owned(),
                record_file(
                    "task.hers-for-her",
                    "task",
                    "review",
                    "Her other record",
                    &["taken-by: Grace", "to: Grace"],
                    "",
                ),
            ),
            (
                "questions/question.for-me.md".to_owned(),
                record_file(
                    "question.for-me",
                    "question",
                    "open",
                    "Which gate opens first?",
                    &["by: Grace", "to: Maks"],
                    "",
                ),
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["status", "--mine", "--budget", "0"]),
            serde_json::json!({
              "counts": {
                "tasks": 2,
                "decisions": 0,
                "notes": 0,
                "questions": 1
              },
              "by": "Maks",
              "review": {
                "count": 1,
                "rows": [
                  {
                    "id": "task.hers-for-me",
                    "taken-by": "Grace",
                    "to": "Maks"
                  }
                ]
              },
              "questions": {
                "count": 1,
                "rows": [
                  {
                    "id": "question.for-me",
                    "created": "2026-08-24",
                    "by": "Grace",
                    "to": "Maks",
                    "title": "Which gate opens first?"
                  }
                ]
              }
            }),
        );
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["status", "--json", "--mine"])).unwrap();
        assert_eq!(
            value["review"]["rows"],
            serde_json::json!([{"id": "task.hers-for-me", "taken-by": "Grace", "to": "Maks"}])
        );
        assert_eq!(value["questions"]["rows"][0]["to"], "Maks");
    }

    /// The dashboard is the work: an open Question is on it with who asked,
    /// a rule is not. Recall adds that rule as knowledge, not work.
    #[test]
    fn an_open_question_reaches_the_dashboard_and_the_hook_and_a_rule_does_not() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.gates.md".to_owned(),
                record_file(
                    "decision.gates",
                    "decision",
                    "active",
                    "The developer opens each gate",
                    &["kind: rule"],
                    "",
                ),
            ),
            (
                "questions/question.gates.md".to_owned(),
                record_file(
                    "question.gates",
                    "question",
                    "open",
                    "Which gate opens first?",
                    &["by: Grace", "to: Maks"],
                    "",
                ),
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["status", "--budget", "0"]),
            serde_json::json!({
              "counts": {
                "tasks": 0,
                "decisions": 1,
                "notes": 0,
                "questions": 1
              },
              "questions": {
                "count": 1,
                "rows": [
                  {
                    "id": "question.gates",
                    "created": "2026-08-24",
                    "by": "Grace",
                    "to": "Maks",
                    "title": "Which gate opens first?"
                  }
                ]
              }
            }),
        );
        let recalled = reply_value(ok(&mut storage, &["hook"]));
        assert_eq!(
            recalled["work"]["questions"]["rows"][0]["id"],
            "question.gates"
        );
        assert!(
            recalled["memories"]
                .as_array()
                .unwrap()
                .iter()
                .any(|memory| memory["id"] == "decision.gates")
        );
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["status", "--json"])).unwrap();
        assert_eq!(
            value["questions"]["rows"][0],
            serde_json::json!({"id": "question.gates", "created": "2026-08-24", "by": "Grace", "to":"Maks", "title": "Which gate opens first?"})
        );
    }

    /// `--mine` narrows the dashboard to the caller's own and says so;
    /// `--team` widens one call past a notebook whose config narrows it.
    #[test]
    fn the_dashboard_narrows_to_the_callers_own_and_widens_on_request() {
        let mut storage = storage_with(&[
            (
                "tasks/task.hers.md".to_owned(),
                record_file(
                    "task.hers",
                    "task",
                    "active",
                    "Her record",
                    &["taken-by: Grace"],
                    "",
                ),
            ),
            (
                "tasks/task.mine.md".to_owned(),
                record_file(
                    "task.mine",
                    "task",
                    "active",
                    "My record",
                    &["taken-by: Maks"],
                    "",
                ),
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["status", "--mine", "--budget", "0"]),
            serde_json::json!({
              "counts": {
                "tasks": 2,
                "decisions": 0,
                "notes": 0,
                "questions": 0
              },
              "by": "Maks",
              "active": {
                "rows": [
                  {
                    "id": "task.mine",
                    "title": "My record"
                  }
                ]
              }
            }),
        );
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["status", "--json", "--mine"])).unwrap();
        assert_eq!(value["by"], serde_json::json!("Maks"));
        assert_eq!(value["active"]["count"], serde_json::json!(1));

        anb_core::Storage::write(&mut storage, "config", "scope: mine\n").unwrap();
        assert!(
            !ok(&mut storage, &["status"]).contains("task.hers"),
            "the config key narrows the plain call"
        );
        let widened = ok(&mut storage, &["status", "--team", "--budget", "0"]);
        assert_reply(
            &widened,
            serde_json::json!({"active":{"count":2,"rows":[{"id":"task.mine"},{"id":"task.hers","taken-by":"Grace"}]}}),
        );
        assert!(reply_value(widened).get("by").is_none());
        assert_eq!(
            reply_value(ok(&mut storage, &["hook"]))["work"]["by"],
            "Maks"
        );
    }

    #[test]
    fn a_quiet_notebook_keeps_its_count_without_active_work() {
        let mut storage = storage_with(&[(
            "tasks/task.done.md".to_owned(),
            record_file("task.done", "task", "closed", "Shipped work", &[], ""),
        )]);
        assert_reply(
            ok(&mut storage, &["status"]),
            serde_json::json!({"quiet":true,"counts":{"tasks":1,"decisions":0,"notes":0,"questions":0},"active":{"count":0,"rows":[]}}),
        );
    }

    #[test]
    fn a_budget_flag_of_zero_lifts_the_ceiling() {
        let output = ok(&mut active_task_storage(), &["status", "--budget", "0"]);
        assert_eq!(
            reply_value(output)["budget"]["limit"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn the_budget_flag_outranks_the_config_key() {
        let mut storage = active_task_storage();
        anb_core::Storage::write(&mut storage, "config", "budget: 40\n").unwrap();
        let from_key = ok(&mut storage, &["status"]);
        assert_eq!(reply_value(from_key)["budget"]["limit"], 40);
        let from_flag = ok(&mut storage, &["status", "--budget", "900"]);
        assert_eq!(reply_value(from_flag)["budget"]["limit"], 900);
    }

    #[test]
    fn the_hook_composes_work_for_the_host_adapter() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file(
                "task.demo",
                "task",
                "active",
                "A demo record",
                &["taken-by: Maks"],
                "",
            ),
        )]);
        assert_reply(
            ok(&mut storage, &["hook"]),
            serde_json::json!({"work":{"active":{"count":1,"rows":[{"id":"task.demo"}]}}}),
        );
    }

    #[test]
    fn hook_read_errors_reach_the_host_adapter() {
        let mut storage = MemoryStorage::new();
        let cli = Cli::try_parse_from(["anb", "hook"]).unwrap();
        assert!(execute(cli.command, &mut storage, undated_host()).is_err());
    }

    #[test]
    fn without_the_hook_the_same_failure_is_a_payload() {
        let mut storage = MemoryStorage::new();
        let cli = Cli::try_parse_from(["anb", "status"]).unwrap();
        let error = execute(cli.command, &mut storage, undated_host()).unwrap_err();
        assert!(matches!(
            error,
            anb_core::NotebookError::InvalidArgument { .. }
        ));
    }

    #[test]
    fn status_json_carries_the_model() {
        let output = ok(&mut active_task_storage(), &["status", "--json"]);
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["quiet"], serde_json::json!(false));
        assert_eq!(value["counts"]["tasks"], serde_json::json!(1));
        assert_eq!(
            value["active"]["rows"][0]["id"],
            serde_json::json!("task.demo")
        );
        let keys = value.as_object().unwrap();
        assert!(
            !keys.contains_key("text") && !keys.contains_key("spent"),
            "the model carries no copy of the text rendering, nor its budget"
        );
    }
}

mod json_surface {
    use super::*;

    /// The queue is what an agent parses to pick up work, so its row names
    /// what it carries — and omits an absent priority like every other absent
    /// field, rather than sending a null through.
    #[test]
    fn a_ready_row_names_its_fields_and_omits_what_is_unset() {
        let mut storage = storage_with(&[
            open_task("task.plain", "Untriaged work", &[]),
            open_task(
                "task.urgent",
                "Triaged work",
                &["by: Maks", "taken-by: Grace", "priority: 1"],
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["ready", "--json"]),
            serde_json::json!({
              "count": 2,
              "ready": [
                {
                  "id": "task.urgent",
                  "priority": 1,
                  "created": "2026-08-24",
                  "by": "Maks",
                  "taken-by": "Grace",
                  "title": "Triaged work"
                },
                {
                  "id": "task.plain",
                  "created": "2026-08-24",
                  "title": "Untriaged work"
                }
              ]
            }),
        );
    }

    #[test]
    fn a_mutation_confirms_in_compact_json() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(&mut storage, &["start", "task.demo", "--json"]),
            serde_json::json!({
              "ok": "start",
              "id": "task.demo",
              "from": "open",
              "to": "active",
              "already": false
            }),
        );
    }

    #[test]
    fn a_refusal_confirms_in_compact_json() {
        let mut storage = MemoryStorage::new();
        assert_reply(
            refused(&mut storage, &["start", "task.absent", "--json"]),
            serde_json::json!({
              "error": "unknown-id",
              "message": "no record `task.absent`",
              "try": [
                "anb list"
              ]
            }),
        );
    }

    #[test]
    fn json_lists_bound_the_same_rows_as_the_text() {
        let mut storage = many_open_tasks(22);
        let ready: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["ready", "--json"])).unwrap();
        assert_eq!(ready["count"], serde_json::json!(22));
        assert_eq!(ready["ready"].as_array().unwrap().len(), 20);
        let all: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["list", "--json", "--all"])).unwrap();
        assert_eq!(all["records"].as_array().unwrap().len(), 22);
    }

    #[test]
    fn a_json_dashboard_section_is_bounded_at_the_dashboard_s_own_rows() {
        let mut storage = many_open_tasks(22);
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["status", "--json"])).unwrap();
        assert_eq!(value["ready"]["count"], serde_json::json!(22));
        assert_eq!(
            value["ready"]["rows"].as_array().unwrap().len(),
            5,
            "the dashboard is a fixed opening in either rendering; `anb ready` is where a \
             section opens whole"
        );
    }

    /// The dashboard counts Debt and points at the read; both renderings
    /// of the dashboard carry the count alone.
    #[test]
    fn the_dashboard_counts_debt_in_both_renderings() {
        let mut storage = storage_with(&[(
            "notes/note.cites.md".to_owned(),
            record_file(
                "note.cites",
                "note",
                "active",
                "A demo record",
                &[],
                "cites task.gone here.\n",
            ),
        )]);
        assert_reply(
            ok(&mut storage, &["status"]),
            serde_json::json!({"debt":{"count":1,"more":"anb debt"}}),
        );
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["status", "--json"])).unwrap();
        assert_fields(&value["debt"], serde_json::json!({"count": 1}));
    }

    #[test]
    fn the_json_debt_rows_are_the_debt_lines_the_text_prints() {
        let mut files: Vec<(String, String)> = (0..3)
            .map(|n| {
                (
                    format!("notes/note.m{n}.md"),
                    record_file(
                        &format!("note.m{n}"),
                        "note",
                        "active",
                        "A demo record",
                        &[],
                        &format!("cites task.gone-{n} here.\n"),
                    ),
                )
            })
            .collect();
        files.extend((0..6).map(|n| {
            (
                format!("tasks/task.bad{n}.md"),
                "not an envelope\n".to_owned(),
            )
        }));
        let mut storage = storage_with(&files);

        let printed: Vec<String> = reply_value(ok(&mut storage, &["debt"]))["debt"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["line"].as_str().unwrap().to_owned())
            .collect();
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["debt", "--json"])).unwrap();
        let carried: Vec<String> = value["debt"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["line"].as_str().unwrap().to_owned())
            .collect();

        assert_eq!(value["count"], serde_json::json!(9));
        assert_eq!(carried, printed, "one listing, two renderings");
        assert_eq!(carried.len(), 9, "every signal, in the clock table's order");
    }

    #[test]
    fn a_json_debt_row_carries_the_signals_fields_beside_its_line() {
        let mut storage = storage_with(&[
            (
                "tasks/task.quiet.md".to_owned(),
                "---\nid: task.quiet\ntype: task\nstate: active\ntitle: A demo record\ncreated: 2026-08-01\nupdated: 2026-08-01\n---\n".to_owned(),
            ),
            (
                "notes/note.cites.md".to_owned(),
                record_file(
                    "note.cites",
                    "note",
                    "active",
                    "A demo record",
                    &[],
                    "cites task.gone here.\n",
                ),
            ),
            (
                "decisions/decision.first.md".to_owned(),
                record_file(
                    "decision.first",
                    "decision",
                    "active",
                    "A demo record",
                    &["by: Ada"],
                    "",
                ),
            ),
            (
                "decisions/decision.second.md".to_owned(),
                record_file(
                    "decision.second",
                    "decision",
                    "active",
                    "A demo record",
                    &["by: Bo", "via: codex"],
                    "departs from decision.first.\n",
                ),
            ),
            (
                "tasks/task.bad.md".to_owned(),
                "---\nid: task.bad\ntype: task\nstate: bogus\ntitle: Broken\ncreated: 2026-08-24\n---\n".to_owned(),
            ),
        ]);
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["debt", "--json"])).unwrap();
        let rows = value["debt"].as_array().unwrap();
        let row = |code: &str| {
            rows.iter()
                .find(|row| row["code"] == code)
                .unwrap_or_else(|| panic!("no {code} row in {rows:?}"))
        };

        assert_eq!(row("task-stale")["id"], serde_json::json!("task.quiet"));
        assert_eq!(
            row("task-stale")["days"],
            serde_json::json!(27),
            "the days between the last touch and today"
        );
        assert_eq!(
            row("dangling-mention")["id"],
            serde_json::json!("note.cites")
        );
        assert_eq!(
            row("dangling-mention")["target"],
            serde_json::json!("task.gone")
        );
        assert!(rows.iter().all(|row| row["code"] != "may-conflict"));
        assert_eq!(
            row("invalid")["file"],
            serde_json::json!("tasks/task.bad.md")
        );
        assert_eq!(
            row("invalid")["errors"],
            serde_json::json!(1),
            "one error finding: the state"
        );
        assert!(
            row("invalid")["line"].is_string(),
            "the printed line stays beside the fields"
        );
    }

    #[test]
    fn json_does_not_infer_conflict_from_shared_tags() {
        let mut storage = storage_with(&[(
            "decisions/decision.first.md".to_owned(),
            record_file(
                "decision.first",
                "decision",
                "active",
                "The first ruling",
                &["by: supolka", "tags: parser, grammar"],
                "",
            ),
        )]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "add",
                    "decision",
                    "Fences stay",
                    "--tag",
                    "parser",
                    "--tag",
                    "grammar",
                    "--json",
                ],
            ),
            serde_json::json!({
              "ok": "add",
              "id": "decision.fences-stay",
              "path": "decisions/decision.fences-stay.md",
              "may-conflict": null
            }),
        );
    }

    #[test]
    fn a_close_reply_on_a_question_carries_its_resolver() {
        let mut storage = storage_with(&[
            (
                "questions/question.doubt.md".to_owned(),
                record_file("question.doubt", "question", "open", "A doubt", &[], ""),
            ),
            (
                "decisions/decision.ruling.md".to_owned(),
                record_file("decision.ruling", "decision", "active", "A ruling", &[], ""),
            ),
        ]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "close",
                    "question.doubt",
                    "--resolved-by",
                    "decision.ruling",
                    "--json",
                ],
            ),
            serde_json::json!({
              "ok": "close",
              "id": "question.doubt",
              "from": "open",
              "to": "closed",
              "already": false,
              "resolved-by": "decision.ruling",
              "unblocked": {
                "count": 0,
                "rows": []
              },
              "open-questions": {
                "count": 0,
                "rows": []
              }
            }),
        );
    }

    #[test]
    fn a_comment_reply_carries_only_the_citations_into_nothing() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "comment",
                    "task.demo",
                    "waits on task.ghost, not the `task.quoted` case",
                    "--json",
                ],
            ),
            serde_json::json!({
              "ok": "comment",
              "id": "task.demo",
              "already": false,
              "dangling-mention": {
                "count": 1,
                "rows": [
                  "task.ghost"
                ]
              }
            }),
        );
        assert_reply(
            ok(
                &mut storage,
                &["comment", "task.demo", "plain text", "--json"],
            ),
            serde_json::json!({
              "ok": "comment",
              "id": "task.demo",
              "already": false
            }),
        );
    }

    #[test]
    fn a_close_reply_carries_its_consequences() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "close",
                    "task.demo",
                    "--body",
                    "Completed and checked.",
                    "--json",
                ],
            ),
            serde_json::json!({
              "ok": "close",
              "id": "task.demo",
              "from": "active",
              "to": "closed",
              "already": false,
              "unblocked": {
                "count": 0,
                "rows": []
              },
              "open-questions": {
                "count": 0,
                "rows": []
              }
            }),
        );
    }

    #[test]
    fn closing_with_a_file_has_no_separate_report_artifact_in_json() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_reply(
            run_reading(
                &mut storage,
                &["close", "task.demo", "--body-file", "r.md", "--json"],
                &[("r.md", "# What shipped\n")],
            )
            .expect("the command must succeed"),
            serde_json::json!({
              "ok": "close",
              "id": "task.demo",
              "from": "active",
              "to": "closed",
              "already": false,
              "unblocked": {
                "count": 0,
                "rows": []
              },
              "open-questions": {
                "count": 0,
                "rows": []
              }
            }),
        );
        assert!(
            anb_core::Storage::list(&storage, "notes")
                .unwrap()
                .is_empty()
        );
    }
}

/// The surface itself: every verb of the task cycle parses, so a rename in
/// the clap tree cannot slip out silently.
#[test]
fn a_skill_check_without_the_directory_it_checks_is_refused() {
    let Err(refusal) = Cli::try_parse_from(["anb", "skill", "--check"]) else {
        panic!("a check with nothing to check against parsed");
    };
    assert_eq!(
        refusal.kind(),
        clap::error::ErrorKind::MissingRequiredArgument,
        "{refusal}"
    );
}

#[test]
fn the_command_vocabulary_parses() {
    for line in [
        vec!["anb", "add", "task", "A title"],
        vec!["anb", "start", "task.x"],
        vec!["anb", "submit", "task.x"],
        vec!["anb", "close", "task.x", "--body", "Completed and checked."],
        vec!["anb", "reopen", "task.x"],
        vec!["anb", "hold", "task.x", "--reason", "why"],
        vec!["anb", "unhold", "task.x"],
        vec!["anb", "block", "task.x", "task.y"],
        vec!["anb", "unblock", "task.x", "task.y"],
        vec!["anb", "comment", "task.x", "note"],
        vec!["anb", "add", "decision", "A ruling", "--kind", "rule"],
        vec![
            "anb",
            "add",
            "note",
            "A fact",
            "--kind",
            "fact",
            "--supersedes",
            "note.y",
        ],
        vec!["anb", "add", "question", "A doubt", "--from", "task.x"],
        vec!["anb", "close", "question.x", "--resolved-by", "decision.y"],
        vec!["anb", "close", "question.x", "--reason", "why"],
        vec!["anb", "close", "task.x", "--reason", "why"],
        vec!["anb", "retire", "decision.x"],
        vec!["anb", "ready"],
        vec!["anb", "list"],
        vec!["anb", "show", "task.x"],
        vec!["anb", "status", "--budget", "0"],
        vec!["anb", "hook"],
        vec!["anb", "check", "--all"],
        vec!["anb", "archive", "task.x"],
        vec!["anb", "delete", "task.x"],
        vec!["anb", "--notebook", "elsewhere", "list"],
        vec!["anb", "ready", "--for", "task.epic"],
        vec!["anb", "list", "--for", "task.epic"],
        vec!["anb", "list", "--notebook", "elsewhere"],
        vec!["anb", "--global", "list"],
        vec!["anb", "list", "--global"],
        vec!["anb", "graph"],
        vec![
            "anb",
            "graph",
            "--for",
            "task.epic",
            "--type",
            "task",
            "--mine",
        ],
        vec![
            "anb",
            "list",
            "--type",
            "decision,note",
            "--kind",
            "rule",
            "--tag",
            "parser",
        ],
        vec!["anb", "ready", "--tag", "parser", "--team"],
        vec!["anb", "list", "--match", "fence", "--archive"],
        vec!["anb", "status", "--mine"],
        vec!["anb", "status", "--by", "Grace", "--budget", "0"],
        vec!["anb", "debt", "--all"],
        vec!["anb", "setup"],
        vec!["anb", "setup", "--remove"],
        vec!["anb", "skill"],
        vec!["anb", "skill", ".agents/skills/anb", "--check"],
        vec![
            "anb",
            "edit",
            "task.x",
            "--title",
            "New",
            "--body",
            "Prose",
            "--tag",
            "epic",
            "--untag",
            "idea",
            "--from",
            "task.hub",
            "--priority",
            "1",
            "--review-by",
            "2026-09-01",
        ],
    ] {
        assert!(Cli::try_parse_from(&line).is_ok(), "must parse: {line:?}");
    }
    assert!(matches!(
        Cli::try_parse_from(["anb", "hook"]).unwrap().command,
        Command::Hook
    ));
}

/// The retry shapes a refusal offers and the repair a finding names are
/// command lines, so the parser must accept them. A flag renamed on one
/// side and not the other leaves an agent typing something that no longer
/// exists.
#[test]
fn every_command_the_tool_offers_can_be_typed_back() {
    // Matched, not listed: a repair the notebook learns to name has to be
    // spelled here before it can be printed at all.
    let repairs = [
        anb_core::Repair::Clear("from"),
        anb_core::Repair::Unblock("task.other".to_owned()),
        anb_core::Repair::Unlink("context note.other".to_owned()),
        anb_core::Repair::Unhold,
        anb_core::Repair::Archive,
        anb_core::Repair::Restore,
    ];
    for repair in &repairs {
        match repair {
            anb_core::Repair::Clear(_)
            | anb_core::Repair::Unblock(_)
            | anb_core::Repair::Unlink(_)
            | anb_core::Repair::Unhold
            | anb_core::Repair::Archive
            | anb_core::Repair::Restore => {}
        }
    }
    let taken = anb::recovery::Recovery::new(
        &anb_core::NotebookError::Taken {
            id: "task.demo".to_owned(),
            taken_by: "Grace".to_owned(),
        },
        &anb::recovery::Subject {
            verb: "start",
            id: Some("task.demo".to_owned()),
            record_type: None,
        },
    );
    let command = Cli::command();
    let offered = command
        .get_subcommands()
        .filter_map(|verb| anb::recovery::runnable(verb.get_name(), Some("task.demo")))
        .flatten()
        .chain(
            repairs
                .iter()
                .map(|repair| anb::reply::repair_command(repair, "tasks/task.demo.md")),
        )
        .chain(taken.tries.iter().cloned());

    let mut checked = 0;
    for shape in offered {
        // The angle-bracketed parts are for a human to fill in; anything
        // else in the line is the contract under test.
        let filled = shell_words(&shape).map(|word| {
            if word.starts_with('<') {
                "task.other".to_owned()
            } else {
                word
            }
        });
        assert!(
            Cli::try_parse_from(filled).is_ok(),
            "the tool offers a command it cannot parse: {shape}"
        );
        checked += 1;
    }
    let named_verbs = command
        .get_subcommands()
        .filter(|verb| anb::recovery::runnable(verb.get_name(), Some("task.demo")).is_some())
        .count();
    assert!(
        checked > named_verbs && named_verbs > 0,
        "only {checked} shapes over {named_verbs} verbs were reached"
    );
}

/// A printed command line back into its words: a quoted run is one
/// word, however many spaces it holds.
fn shell_words(line: &str) -> impl Iterator<Item = String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    for character in line.chars() {
        match character {
            c if quote == Some(c) => quote = None,
            c @ ('"' | '\'') if quote.is_none() => quote = Some(c),
            c if c.is_whitespace() && quote.is_none() => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            c => word.push(c),
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words.into_iter()
}

mod maintenance_replies {
    use super::*;
    use anb_core::Storage as _;

    fn closed_task(id: &str) -> (String, String) {
        (
            format!("tasks/{id}.md"),
            record_file(id, "task", "closed", "A demo record", &[], ""),
        )
    }

    #[test]
    fn a_clean_check_answers_count_zero() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(&mut storage, &["check"]),
            serde_json::json!({
              "count": 0,
              "findings": []
            }),
        );
    }

    #[test]
    fn check_names_file_line_severity_code_repair_and_reason() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "cancelled", "A demo record", &[], ""),
        )]);
        assert_reply(
            ok(&mut storage, &["check"]),
            serde_json::json!({
              "count": 1,
              "findings": [
                {
                  "file": "tasks/task.demo.md",
                  "line": 4,
                  "severity": "error",
                  "code": "bad-value",
                  "repair": null,
                  "message": "state: `cancelled` is not one of open, active, review, closed for a task"
                }
              ]
            }),
        );
    }

    /// The promise of the column: every repair it names runs, and running
    /// them is all it takes. A record broken twice over is repaired one
    /// verb at a time, so the loop must terminate on that too.
    #[test]
    fn running_the_repair_each_finding_names_clears_the_notebook() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md".to_owned(),
                record_file(
                    "task.a",
                    "task",
                    "open",
                    "A demo record",
                    &["from: task.ghost", "blocked-by: task.b"],
                    "",
                ),
            ),
            (
                "tasks/task.b.md".to_owned(),
                record_file(
                    "task.b",
                    "task",
                    "open",
                    "A demo record",
                    &["blocked-by: task.a"],
                    "",
                ),
            ),
            (
                "notes/note.stray.md".to_owned(),
                record_file(
                    "note.stray",
                    "note",
                    "retired",
                    "A demo record",
                    &["kind: fact", "priority: 9"],
                    "",
                ),
            ),
            (
                "tasks/task.twice.md".to_owned(),
                record_file(
                    "task.twice",
                    "task",
                    "open",
                    "A demo record",
                    &["priority: 9", "review-by: nope", "hold-until: 2026-09-01"],
                    "",
                ),
            ),
        ]);
        let mut repaired = 0;
        while let Some(repair) = first_repair(&mut storage) {
            let command: Vec<&str> = repair.split(' ').skip(1).collect();
            ok(&mut storage, &command);
            repaired += 1;
            assert!(repaired < 10, "`{repair}` left its own finding standing");
        }
        assert_reply(
            ok(&mut storage, &["check"]),
            serde_json::json!({
              "count": 0,
              "findings": []
            }),
        );
    }

    /// Only a Task is taken, and only work or a doubt waits on anyone; on
    /// any other record the line is an orphan `edit` erases, and the
    /// finding names that eraser.
    #[test]
    fn a_person_named_on_another_type_names_its_eraser() {
        for (field, line) in [("taken-by", "taken-by: Grace"), ("to", "to: Grace")] {
            let mut storage = storage_with(&[(
                "notes/note.stray.md".to_owned(),
                record_file("note.stray", "note", "active", "A demo record", &[line], ""),
            )]);
            assert_eq!(
                first_repair(&mut storage).as_deref(),
                Some(format!("anb edit note.stray --clear {field}").as_str())
            );
            ok(&mut storage, &["edit", "note.stray", "--clear", field]);
            assert_reply(
                ok(&mut storage, &["check"]),
                serde_json::json!({"count":0,"findings":[]}),
            );
        }
    }

    /// A repair is a command that runs. A line only a Task verb erases,
    /// standing on a record of another type, names no repair — one that did
    /// would send an agent straight into a `wrong-type` refusal.
    #[test]
    fn a_task_only_line_on_another_type_names_no_repair() {
        let mut storage = storage_with(&[(
            "notes/note.stray.md".to_owned(),
            record_file(
                "note.stray",
                "note",
                "active",
                "A demo record",
                &["hold-until: 2026-09-01"],
                "",
            ),
        )]);
        let report: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["check", "--json"])).unwrap();
        let findings = report["findings"].as_array().unwrap();
        assert!(!findings.is_empty(), "the stray line is a finding");
        for finding in findings {
            assert!(finding.get("repair").is_none(), "{finding}");
        }
    }

    /// A correcting verb resolves an id to the one live path its type
    /// dictates, and the archive is reached only to be moved back or
    /// deleted — so a record at neither canonical path is a record no verb
    /// can repair, and a row that named a command anyway would send an
    /// agent in a circle.
    #[test]
    fn a_finding_on_a_record_no_verb_can_reach_names_no_repair() {
        let broken =
            |id: &str| record_file(id, "task", "open", "A demo record", &["priority: 9"], "");
        let mut storage = storage_with(&[
            ("notes/task.stray.md".to_owned(), broken("task.stray")),
            ("tasks/weird.md".to_owned(), broken("task.nameless")),
        ]);
        let report: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["check", "--json", "--all"])).unwrap();
        for finding in report["findings"].as_array().unwrap() {
            assert!(
                finding.get("repair").is_none(),
                "no verb reaches this file: {finding}"
            );
        }
    }

    /// The archive is reachable by exactly one verb, so the only repair
    /// named on an archived file is the residence move itself; what is
    /// broken deeper inside gets its repair once the record is back.
    #[test]
    fn an_archived_record_names_restore_for_its_residence_finding_alone() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.filed.md".to_owned(),
            record_file(
                "task.filed",
                "task",
                "open",
                "A demo record",
                &["priority: 9"],
                "",
            ),
        )]);
        let report: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["check", "--json", "--all"])).unwrap();
        let repair_of = |code: &str| {
            report["findings"]
                .as_array()
                .unwrap()
                .iter()
                .find(|finding| finding["code"] == code)
                .unwrap_or_else(|| panic!("{code} must be reported"))
                .get("repair")
                .cloned()
        };
        assert_eq!(
            repair_of("archived-live-record"),
            Some(serde_json::json!("anb restore task.filed"))
        );
        assert_eq!(repair_of("bad-value"), None);
    }

    /// The first finding that names a repair, as the command line to run.
    fn first_repair(storage: &mut MemoryStorage) -> Option<String> {
        let report: serde_json::Value =
            serde_json::from_str(&ok(storage, &["check", "--json"])).unwrap();
        report["findings"]
            .as_array()?
            .iter()
            .find_map(|finding| Some(finding.get("repair")?.as_str()?.to_owned()))
    }

    #[test]
    fn a_check_with_error_findings_is_a_failing_exit() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "cancelled", "A demo record", &[], ""),
        )]);
        let cli = Cli::try_parse_from(["anb", "check"]).unwrap();
        let reply = execute(
            cli.command,
            &mut storage,
            Host {
                session: None,
                identity: || None,
                read_file: &missing_report,
                lost_proofs: &nothing_lost,
                user_notebook: None,
                personal_notebook: None,
                audience: anb::recall::Audience::Project,
                project_dir: std::path::Path::new("."),
                today: TODAY,
            },
        )
        .unwrap();
        assert!(reply.failed(), "error findings must gate a caller like CI");
    }

    #[test]
    fn a_warning_only_check_is_a_passing_exit() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file(
                "task.demo",
                "task",
                "open",
                "A demo record",
                &["custom: kept"],
                "",
            ),
        )]);
        let cli = Cli::try_parse_from(["anb", "check"]).unwrap();
        let reply = execute(
            cli.command,
            &mut storage,
            Host {
                session: None,
                identity: || None,
                read_file: &missing_report,
                lost_proofs: &nothing_lost,
                user_notebook: None,
                personal_notebook: None,
                audience: anb::recall::Audience::Project,
                project_dir: std::path::Path::new("."),
                today: TODAY,
            },
        )
        .unwrap();
        assert!(!reply.failed(), "warnings keep the record usable");
    }

    #[test]
    fn a_replayed_archive_answers_already() {
        let mut storage = storage_with(&[closed_task("task.demo")]);
        ok(&mut storage, &["archive", "task.demo"]);
        assert_reply(
            ok(&mut storage, &["archive", "task.demo"]),
            serde_json::json!({
              "ok": "archive",
              "id": "task.demo",
              "already": true
            }),
        );
    }

    #[test]
    fn a_restore_is_the_archive_move_made_back() {
        let mut storage = storage_with(&[closed_task("task.demo")]);
        ok(&mut storage, &["archive", "task.demo"]);
        assert_reply(
            ok(&mut storage, &["restore", "task.demo"]),
            serde_json::json!({
              "ok": "restore",
              "id": "task.demo",
              "from": "archive/tasks/task.demo.md",
              "to": "tasks/task.demo.md"
            }),
        );
    }

    #[test]
    fn a_replayed_restore_answers_already() {
        let mut storage = storage_with(&[closed_task("task.demo")]);
        assert_reply(
            ok(&mut storage, &["restore", "task.demo"]),
            serde_json::json!({
              "ok": "restore",
              "id": "task.demo",
              "already": true
            }),
        );
    }

    #[test]
    fn archiving_a_live_task_names_the_settling_command() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_reply(
            refused(&mut storage, &["archive", "task.demo"]),
            serde_json::json!({
              "error": "invalid-transition",
              "message": "`task.demo` is active; valid: close",
              "try": [
                "anb close task.demo --body \"<outcome>\"",
                "anb close task.demo --reason \"<why>\""
              ]
            }),
        );
    }

    #[test]
    fn expunge_answers_the_file_that_is_gone() {
        let mut storage = storage_with(&[(
            "notes/note.mistake.md".to_owned(),
            record_file(
                "note.mistake",
                "note",
                "active",
                "Written in error",
                &[],
                "",
            ),
        )]);
        assert_reply(
            ok(&mut storage, &["delete", "note.mistake"]),
            serde_json::json!({"ok":"delete","id":"note.mistake","paths":["notes/note.mistake.md"]}),
        );
    }

    #[test]
    fn a_held_expunge_lists_every_blocker_with_its_carrier() {
        let mut storage = storage_with(&[
            (
                "notes/note.mistake.md".to_owned(),
                record_file(
                    "note.mistake",
                    "note",
                    "active",
                    "Written in error",
                    &[],
                    "",
                ),
            ),
            open_task("task.born", "Born from the slip", &["from: note.mistake"]),
            (
                "tasks/task.citing.md".to_owned(),
                record_file(
                    "task.citing",
                    "task",
                    "open",
                    "A task naming it in prose",
                    &[],
                    "grew out of note.mistake\n",
                ),
            ),
        ]);
        assert_reply(
            refused(&mut storage, &["delete", "note.mistake"]),
            serde_json::json!({
              "error": "still-referenced",
              "message": "`note.mistake` is still referenced by 2 records",
              "try": [
                "anb show task.born",
                "anb show task.citing"
              ],
              "findings": [
                "task.born — from",
                "task.citing — body"
              ]
            }),
        );
    }

    #[test]
    fn one_carrier_holding_two_edges_is_opened_once() {
        let mut storage = storage_with(&[
            (
                "notes/note.mistake.md".to_owned(),
                record_file(
                    "note.mistake",
                    "note",
                    "active",
                    "Written in error",
                    &[],
                    "",
                ),
            ),
            (
                "tasks/task.holder.md".to_owned(),
                record_file(
                    "task.holder",
                    "task",
                    "open",
                    "Holding it twice over",
                    &["from: note.mistake"],
                    "and it says so again: note.mistake\n",
                ),
            ),
        ]);
        assert_reply(
            refused(&mut storage, &["delete", "note.mistake"]),
            serde_json::json!({
              "error": "still-referenced",
              "message": "`note.mistake` is still referenced by 1 record",
              "try": [
                "anb show task.holder"
              ],
              "findings": [
                "task.holder — from",
                "task.holder — body"
              ]
            }),
        );
    }

    /// A hub with two children, one closed, and a Question born a level
    /// down — enough for every epic surface to have something to say.
    fn an_epic() -> MemoryStorage {
        storage_with(&[
            (
                "tasks/task.epic-auth.md".to_owned(),
                record_file(
                    "task.epic-auth",
                    "task",
                    "open",
                    "Auth end to end",
                    &[
                        "blocked-by: task.auth-login",
                        "blocked-by: task.auth-tokens",
                    ],
                    "",
                ),
            ),
            (
                "tasks/task.auth-login.md".to_owned(),
                record_file(
                    "task.auth-login",
                    "task",
                    "closed",
                    "The login screen",
                    &["from: task.epic-auth"],
                    "",
                ),
            ),
            open_task(
                "task.auth-tokens",
                "Token rotation",
                &["from: task.epic-auth"],
            ),
        ])
    }

    #[test]
    fn a_scoped_queue_answers_only_the_epic_it_was_asked_about() {
        let mut storage = an_epic();
        ok(&mut storage, &["add", "task", "Something else entirely"]);
        assert_reply(
            ok(&mut storage, &["ready", "--for", "task.epic-auth"]),
            serde_json::json!({
              "count": 1,
              "ready": [
                {
                  "id": "task.auth-tokens",
                  "priority": null,
                  "created": "2026-08-24",
                  "taken-by": null,
                  "title": "Token rotation"
                }
              ]
            }),
        );
    }

    /// A scoped listing answers a narrower question than the bare verb, so
    /// the command that lifts its bound has to carry the scope — the hint
    /// is a command a reader runs, not a decoration.
    #[test]
    fn a_scoped_listing_lifts_with_the_scope_it_was_asked_with() {
        let mut files: Vec<(String, String)> = (0..25)
            .map(|n| {
                open_task(
                    &format!("task.m{n:02}"),
                    "A demo record",
                    &["from: task.hub"],
                )
            })
            .collect();
        let waits: Vec<String> = (0..25)
            .map(|n| format!("blocked-by: task.m{n:02}"))
            .collect();
        files.push(open_task(
            "task.hub",
            "The hub",
            &waits.iter().map(String::as_str).collect::<Vec<&str>>(),
        ));
        let mut storage = storage_with(&files);
        for (verb, hint, rows_in_scope) in [
            ("list", "anb list --for task.hub --all", 25 + 1),
            ("ready", "anb ready --for task.hub --all", 25),
        ] {
            let out = ok(&mut storage, &[verb, "--for", "task.hub"]);
            assert_eq!(reply_value(out)["more"], hint);

            // The hint is a command, so running it must answer what it
            // promises: every row of the same scope, and nothing left to hint.
            let lifted = ok(&mut storage, &[verb, "--for", "task.hub", "--all"]);
            let key = if verb == "list" { "records" } else { "ready" };
            assert_eq!(
                reply_value(&lifted)[key].as_array().unwrap().len(),
                rows_in_scope
            );
            assert!(!lifted.contains("more:"), "{lifted}");
        }
    }

    #[test]
    fn a_scope_named_by_no_record_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(&mut storage, &["ready", "--for", "task.no-such-epic"]),
            serde_json::json!({
              "error": "unknown-id",
              "message": "no record `task.no-such-epic`",
              "try": [
                "anb list"
              ]
            }),
        );
    }

    #[test]
    fn a_repeated_clear_flag_names_every_field_it_erased() {
        let mut storage = storage_with(&[open_task(
            "task.demo",
            "A demo record",
            &["from: task.parent", "priority: 2"],
        )]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "edit",
                    "task.demo",
                    "--clear",
                    "from",
                    "--clear",
                    "priority",
                ],
            ),
            serde_json::json!({
              "ok": "edit",
              "id": "task.demo",
              "changed": [
                "from",
                "priority"
              ],
              "already": false
            }),
        );
    }

    #[test]
    fn a_field_with_no_eraser_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(&mut storage, &["edit", "task.demo", "--clear", "state"]),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "clear: `state` is not an erasable field; from, priority, review-by, taken-by, to",
              "try": [
                "anb edit task.demo --title \"<title>\""
              ]
            }),
        );
    }

    #[test]
    fn an_edit_changing_nothing_answers_already() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(
                &mut storage,
                &["edit", "task.demo", "--title", "A demo record"],
            ),
            serde_json::json!({
              "ok": "edit",
              "id": "task.demo",
              "changed": [],
              "already": true
            }),
        );
    }

    #[test]
    fn an_edited_body_carries_the_dangling_mention_nudge() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(
                &mut storage,
                &["edit", "task.demo", "--body", "Blocked by task.ghost."],
            ),
            serde_json::json!({
              "ok": "edit",
              "id": "task.demo",
              "changed": [
                "body"
              ],
              "already": false,
              "dangling-mention": {
                "count": 1,
                "rows": [
                  "task.ghost"
                ]
              }
            }),
        );
    }

    #[test]
    fn an_edit_takes_the_whole_body_from_standard_input() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            run_reading(
                &mut storage,
                &["edit", "task.demo", "--body-file", "-"],
                &[("-", "The body as piped.\n")],
            )
            .expect("the command must succeed"),
            serde_json::json!({
              "ok": "edit",
              "id": "task.demo",
              "changed": [
                "body"
              ],
              "already": false
            }),
        );
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .ends_with("---\n\nThe body as piped.\n")
        );
    }

    #[test]
    fn an_empty_body_file_clears_the_body_as_an_empty_body_does() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file(
                "task.demo",
                "task",
                "open",
                "A demo record",
                &[],
                "\nOld prose.\n",
            ),
        )]);
        assert_reply(
            run_reading(
                &mut storage,
                &["edit", "task.demo", "--body-file", "empty.md"],
                &[("empty.md", "")],
            )
            .expect("the command must succeed"),
            serde_json::json!({
              "ok": "edit",
              "id": "task.demo",
              "changed": [
                "body"
              ],
              "already": false
            }),
        );
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .ends_with("---\n"),
            "no body follows the envelope"
        );
    }

    #[test]
    fn an_explicit_link_records_a_relationship_without_guessing_conflict() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.first.md".to_owned(),
                record_file(
                    "decision.first",
                    "decision",
                    "active",
                    "A demo record",
                    &["kind: rule"],
                    "",
                ),
            ),
            (
                "decisions/decision.second.md".to_owned(),
                record_file(
                    "decision.second",
                    "decision",
                    "active",
                    "A demo record",
                    &["kind: drift"],
                    "Departs from decision.first for the import.\n",
                ),
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["debt"]),
            serde_json::json!({"count":0,"debt":[]}),
        );
        assert_reply(
            ok(
                &mut storage,
                &[
                    "edit",
                    "decision.second",
                    "--link",
                    "departs-from decision.first",
                ],
            ),
            serde_json::json!({
              "ok": "edit",
              "id": "decision.second",
              "changed": [
                "link"
              ],
              "already": false
            }),
        );
        assert!(
            !ok(&mut storage, &["debt"]).contains("may-conflict"),
            "explicit relationships do not infer a contradiction"
        );
    }

    #[test]
    fn either_record_can_declare_an_explicit_link() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.first.md".to_owned(),
                record_file(
                    "decision.first",
                    "decision",
                    "active",
                    "A demo record",
                    &["kind: rule"],
                    "",
                ),
            ),
            (
                "decisions/decision.second.md".to_owned(),
                record_file(
                    "decision.second",
                    "decision",
                    "active",
                    "A demo record",
                    &["kind: rule"],
                    "Part of decision.first.\n",
                ),
            ),
        ]);
        ok(
            &mut storage,
            &["edit", "decision.first", "--link", "within decision.second"],
        );
        assert!(
            !ok(&mut storage, &["status"]).contains("may-conflict"),
            "either record may declare the edge"
        );
    }

    #[test]
    fn a_link_naming_no_record_is_refused_before_any_byte_moves() {
        let text = record_file(
            "decision.first",
            "decision",
            "active",
            "A demo record",
            &["kind: rule"],
            "",
        );
        let mut storage = storage_with(&[("decisions/decision.first.md".to_owned(), text.clone())]);
        assert_reply(
            refused(
                &mut storage,
                &["edit", "decision.first", "--link", "within decision.ghost"],
            ),
            serde_json::json!({
              "error": "dangling-ref",
              "message": "link: `decision.ghost` names no record",
              "try": [
                "anb list"
              ]
            }),
        );
        assert_eq!(storage.read("decisions/decision.first.md").unwrap(), text);
    }

    #[test]
    fn a_link_without_a_target_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(&mut storage, &["edit", "task.demo", "--link", "within"]),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "link: `within` is not `<kind> <target>`",
              "try": [
                "anb edit task.demo --title \"<title>\""
              ]
            }),
        );
    }

    #[test]
    fn an_edit_requesting_nothing_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(&mut storage, &["edit", "task.demo"]),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "edit: nothing to change; pass --title, --body, --tag, --untag, --link, --unlink, --from, --priority, --review-by, --taken-by, --to, or --clear",
              "try": [
                "anb edit task.demo --title \"<title>\""
              ]
            }),
        );
    }

    #[test]
    fn a_file_the_adapter_cannot_read_renders_the_not_utf8_payload() {
        use anb_core::{NotebookError, StorageError};
        let subject = anb::recovery::Subject {
            verb: "show",
            id: Some("task.demo".to_owned()),
            record_type: None,
        };
        let error = NotebookError::Storage(StorageError::NotUtf8 {
            path: "tasks/task.demo.md".to_owned(),
        });
        assert_reply(
            text::render_error(&error, &subject),
            serde_json::json!({
              "error": "not-utf8",
              "message": "not UTF-8: tasks/task.demo.md",
              "try": [
                "anb check"
              ]
            }),
        );
    }
}

mod narrowed_listings {
    use super::*;

    /// A text reaches the archive only when asked for: history is
    /// findable, and it is not what a working question is about.
    #[test]
    fn a_text_narrows_the_listing_and_the_archive_joins_when_asked_for() {
        let mut storage = storage_with(&[
            open_task("task.parser", "Grammar parser work", &[]),
            (
                "archive/tasks/task.spike.md".to_owned(),
                record_file("task.spike", "task", "closed", "Parser spike", &[], ""),
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["list", "--match", "parser"]),
            serde_json::json!({
              "count": 1,
              "records": [
                {
                  "id": "task.parser",
                  "state": "open",
                  "priority": null,
                  "title": "Grammar parser work"
                }
              ]
            }),
        );
        assert_reply(
            ok(&mut storage, &["list", "--match", "parser", "--archive"]),
            serde_json::json!({
              "count": 2,
              "records": [
                {
                  "id": "task.parser",
                  "state": "open",
                  "priority": null,
                  "title": "Grammar parser work"
                },
                {
                  "id": "task.spike",
                  "state": "closed",
                  "priority": null,
                  "title": "Parser spike"
                }
              ]
            }),
        );
    }

    /// Every standing rule, and every Note of one tag, is one listing.
    #[test]
    fn a_type_a_kind_and_a_tag_narrow_the_listing() {
        let mut storage = storage_with(&[
            open_task("task.demo", "A demo record", &["tags: parser"]),
            (
                "decisions/decision.nest.md".to_owned(),
                record_file(
                    "decision.nest",
                    "decision",
                    "active",
                    "Fences never nest",
                    &["kind: rule", "tags: parser"],
                    "",
                ),
            ),
            (
                "decisions/decision.rust.md".to_owned(),
                record_file(
                    "decision.rust",
                    "decision",
                    "active",
                    "Rust for the CLI",
                    &["kind: shape", "tags: stack"],
                    "",
                ),
            ),
            (
                "notes/note.fence.md".to_owned(),
                record_file(
                    "note.fence",
                    "note",
                    "active",
                    "Fence",
                    &["kind: term", "tags: parser, domain-model"],
                    "",
                ),
            ),
        ]);
        assert_reply(
            ok(
                &mut storage,
                &["list", "--type", "decision", "--kind", "rule"],
            ),
            serde_json::json!({
              "count": 1,
              "records": [
                {
                  "id": "decision.nest",
                  "state": "active",
                  "priority": null,
                  "title": "Fences never nest"
                }
              ]
            }),
        );
        assert_reply(
            ok(
                &mut storage,
                &["list", "--type", "note", "--tag", "domain-model"],
            ),
            serde_json::json!({
              "count": 1,
              "records": [
                {
                  "id": "note.fence",
                  "state": "active",
                  "priority": null,
                  "title": "Fence"
                }
              ]
            }),
        );
        assert_reply(
            ok(&mut storage, &["list", "--tag", "parser"]),
            serde_json::json!({
              "count": 3,
              "records": [
                {
                  "id": "task.demo",
                  "state": "open",
                  "priority": null,
                  "title": "A demo record"
                },
                {
                  "id": "decision.nest",
                  "state": "active",
                  "priority": null,
                  "title": "Fences never nest"
                },
                {
                  "id": "note.fence",
                  "state": "active",
                  "priority": null,
                  "title": "Fence"
                }
              ]
            }),
        );
    }

    /// The hint is meant to be typed back, so every narrowing travels in
    /// it, and a text survives the shell — including the quote that would
    /// end the word.
    #[test]
    fn the_truncation_hint_carries_every_narrowing_as_shell_words() {
        let mut storage = many_open_tasks(22);
        assert_reply(
            ok(
                &mut storage,
                &["list", "--type", "task", "--match", "demo record"],
            ),
            serde_json::json!({"omitted":2,"more":"anb list --type task --match 'demo record' --all"}),
        );
        let files: Vec<(String, String)> = (0..22)
            .map(|n| {
                open_task(
                    &format!("task.q{n:02}"),
                    "Won't fix, won't file",
                    &["tags: parser"],
                )
            })
            .collect();
        assert_reply(
            ok(
                &mut storage_with(&files),
                &["ready", "--tag", "parser", "--match", "won't"],
            ),
            serde_json::json!({"omitted":2,"more":r"anb ready --tag parser --match 'won'\''t' --all"}),
        );
    }

    #[test]
    fn a_match_of_nothing_answers_count_zero() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(&mut storage, &["list", "--match", "zeppelin"]),
            serde_json::json!({
              "count": 0,
              "records": []
            }),
        );
    }

    /// The queue is live open Tasks by definition, so it takes no flag
    /// that could only admit nothing more: the command line refuses it
    /// before any record is read.
    #[test]
    fn the_queue_takes_no_narrowing_it_cannot_answer() {
        for flag in [&["--type", "task"][..], &["--kind", "rule"], &["--archive"]] {
            let mut line = vec!["anb", "ready"];
            line.extend_from_slice(flag);
            assert!(
                Cli::try_parse_from(&line).is_err(),
                "the queue must refuse {flag:?}"
            );
        }
        assert!(Cli::try_parse_from(["anb", "list", "--type", "task", "--archive"]).is_ok());
    }

    /// A kind no type allows is refused with the vocabulary named, rather
    /// than answered with a listing of nothing.
    #[test]
    fn a_kind_no_type_allows_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            refused(&mut storage, &["list", "--kind", "law"]),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "kind: `law` is not one of rule, shape, drift, fact, term, guide, idea, model, spec",
              "try": []
            }),
        );
    }

    /// `--team` widens one call past a notebook whose config narrows every
    /// read to the caller's own; `--mine` narrows one call under the team's.
    /// A read narrowed to one identity says whose it is, flag or config key
    /// alike, and how to widen it: the key narrows by nothing the caller
    /// typed, and a `count: 0` under it would otherwise read as an empty
    /// notebook.
    #[test]
    fn the_scope_key_narrows_every_read_and_team_widens_one_call() {
        let mut storage = storage_with(&[
            open_task("task.mine", "My record", &["taken-by: Maks"]),
            open_task("task.hers", "Her record", &["taken-by: Grace"]),
        ]);
        anb_core::Storage::write(&mut storage, "config", "scope: mine\n").unwrap();
        assert_reply(
            ok(&mut storage, &["ready"]),
            serde_json::json!({
              "by": "Maks",
              "count": 1,
              "ready": [
                {
                  "id": "task.mine",
                  "priority": null,
                  "created": "2026-08-24",
                  "taken-by": "Maks",
                  "title": "My record"
                }
              ]
            }),
        );
        assert_reply(
            ok(&mut storage, &["list", "--team"]),
            serde_json::json!({
              "count": 2,
              "records": [
                {
                  "id": "task.hers",
                  "state": "open",
                  "priority": null,
                  "title": "Her record"
                },
                {
                  "id": "task.mine",
                  "state": "open",
                  "priority": null,
                  "title": "My record"
                }
              ]
            }),
        );
        assert_reply(
            ok(&mut storage, &["list", "--by", "Grace"]),
            serde_json::json!({
              "by": "Grace",
              "count": 1,
              "records": [
                {
                  "id": "task.hers",
                  "state": "open",
                  "priority": null,
                  "title": "Her record"
                }
              ]
            }),
        );
        assert_reply(
            ok(
                &mut storage,
                &["list", "--type", "question", "--tag", "parser"],
            ),
            serde_json::json!({"by":"Maks","count":0,"records":[]}),
        );
        assert_reply(
            ok(&mut storage, &["--json", "list", "--type", "question"]),
            serde_json::json!({
              "by": "Maks",
              "count": 0,
              "records": []
            }),
        );
        assert_reply(
            ok(&mut storage, &["graph"]),
            serde_json::json!({"slice":{"by":"Maks"},"team":"anb graph --team","nodes":{"count":1}}),
        );
        assert_reply(
            ok(&mut storage, &["graph", "--focus", "task.mine", "--full"]),
            serde_json::json!({"team":"anb graph --focus task.mine --depth 1 --full --team"}),
        );
        assert!(
            !ok(&mut storage, &["--json", "list", "--team"]).contains("\"by\""),
            "everyone's read names nobody"
        );
    }

    /// The pool is the Tasks nobody holds, one read under any scope: it
    /// answers whose outright, so `scope: mine` does not narrow it, and the
    /// hint that lifts its bound carries the flag. On the dashboard the
    /// pool is a count under a narrowing and part of the queue otherwise.
    #[test]
    fn untaken_is_the_pool_under_any_scope_and_the_dashboard_counts_it() {
        let mut files: Vec<(String, String)> = (0..21)
            .map(|n| open_task(&format!("task.free{n:02}"), "Anyone's", &["by: Grace"]))
            .collect();
        files.push(open_task("task.hers", "Her record", &["taken-by: Grace"]));
        let mut storage = storage_with(&files);
        anb_core::Storage::write(&mut storage, "config", "scope: mine\n").unwrap();
        let pool = ok(&mut storage, &["ready", "--untaken"]);
        assert_reply(
            &pool,
            serde_json::json!({"count":21,"omitted":1,"more":"anb ready --untaken --all"}),
        );
        assert!(!pool.contains("task.hers"), "{pool}");
        let listed = ok(&mut storage, &["list", "--untaken", "--all"]);
        assert_eq!(reply_value(listed)["records"].as_array().unwrap().len(), 21);

        let status = ok(&mut storage, &["status", "--budget", "0"]);
        assert_reply(
            status,
            serde_json::json!({"untaken":{"count":21,"more":"anb ready --untaken"},"ready":{"count":0,"rows":[]}}),
        );
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["status", "--json"])).unwrap();
        assert_fields(&value["untaken"], serde_json::json!({"count": 21}));
        let team = ok(&mut storage, &["status", "--team", "--budget", "0"]);
        assert_eq!(
            reply_value(team)["ready"]["rows"].as_array().unwrap().len(),
            22
        );
    }

    /// The pool is nobody's, so asking for it beside a name asks a
    /// contradiction, and the command line refuses the pair.
    #[test]
    fn untaken_beside_a_name_is_refused_by_the_command_line() {
        for line in [
            vec!["anb", "ready", "--untaken", "--mine"],
            vec!["anb", "list", "--untaken", "--by", "Grace"],
            vec!["anb", "graph", "--untaken", "--team"],
        ] {
            assert!(Cli::try_parse_from(line.clone()).is_err(), "{line:?}");
        }
        assert!(Cli::try_parse_from(["anb", "status", "--untaken"]).is_err());
    }

    /// Under `scope: mine` a host that knows nobody has nothing to narrow
    /// by; the refusal names the fix and the way around.
    #[test]
    fn the_scope_key_without_an_identity_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        anb_core::Storage::write(&mut storage, "config", "scope: mine\n").unwrap();
        let cli = Cli::try_parse_from(["anb", "list"]).unwrap();
        let error = execute(cli.command, &mut storage, anonymous_host()).unwrap_err();
        assert_reply(
            text::render_error(
                &error,
                &anb::recovery::subject(&Cli::try_parse_from(["anb", "list"]).unwrap().command),
            ),
            serde_json::json!({
              "error": "invalid-argument",
              "message": "scope: mine needs an identity; set git user.name or ANB_BY, or pass --team",
              "try": []
            }),
        );
    }
}

/// The graph: the rows a caller reads, the slice they name, and the file
/// the shell leaves when they ask for a picture instead.
mod task_graph {
    use super::*;

    /// A chain of two beside a Task of its own: three tiles, one line, and
    /// only two of them startable now.
    fn a_chain() -> MemoryStorage {
        storage_with(&[
            open_task("task.first", "The blocker", &["tags: parser"]),
            open_task(
                "task.second",
                "The waiter",
                &["blocked-by: task.first", "tags: parser"],
            ),
            open_task("task.stray", "Another line of work", &[]),
        ])
    }

    /// Every other verb answers an agent, and a picture answers nobody
    /// without a browser. So the graph itself is what the verb prints.
    #[test]
    fn the_verb_prints_the_graph_itself() {
        assert_reply(
            ok(&mut a_chain(), &["graph"]),
            serde_json::json!({
              "nodes": {
                "count": 3,
                "rows": [
                  {
                    "id": "task.first",
                    "type": "task",
                    "state": "open",
                    "ready": true,
                    "archived": false,
                    "degree": 1,
                    "priority": null,
                    "created": "2026-08-24",
                    "epic": null,
                    "title": "The blocker"
                  },
                  {
                    "id": "task.second",
                    "type": "task",
                    "state": "open",
                    "ready": false,
                    "archived": false,
                    "degree": 1,
                    "priority": null,
                    "created": "2026-08-24",
                    "epic": null,
                    "title": "The waiter"
                  },
                  {
                    "id": "task.stray",
                    "type": "task",
                    "state": "open",
                    "ready": true,
                    "archived": false,
                    "degree": 0,
                    "priority": null,
                    "created": "2026-08-24",
                    "epic": null,
                    "title": "Another line of work"
                  }
                ]
              },
              "edges": {
                "count": 1,
                "rows": [
                  {
                    "from": "task.first",
                    "to": "task.second",
                    "kind": "waits"
                  }
                ]
              }
            }),
        );
    }

    /// A link between two records is drawn under the link's own word, out
    /// of the record that declares it, on both surfaces.
    #[test]
    fn a_link_between_records_is_an_edge_under_the_links_own_word() {
        let mut storage = storage_with(&[
            open_task("task.first", "The blocker", &[]),
            open_task(
                "task.second",
                "The waiter",
                &["link: spec task.first", "link: pr https://example.com/2"],
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["graph"]),
            serde_json::json!({
              "nodes": {
                "count": 2,
                "rows": [
                  {
                    "id": "task.first",
                    "type": "task",
                    "state": "open",
                    "ready": true,
                    "archived": false,
                    "degree": 1,
                    "priority": null,
                    "created": "2026-08-24",
                    "epic": null,
                    "title": "The blocker"
                  },
                  {
                    "id": "task.second",
                    "type": "task",
                    "state": "open",
                    "ready": true,
                    "archived": false,
                    "degree": 1,
                    "priority": null,
                    "created": "2026-08-24",
                    "epic": null,
                    "title": "The waiter"
                  }
                ]
              },
              "edges": {
                "count": 1,
                "rows": [
                  {
                    "from": "task.second",
                    "to": "task.first",
                    "kind": "spec"
                  }
                ]
              }
            }),
        );
        let payload = ok(&mut storage, &["--json", "graph"]);
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(
            parsed["edges"]["rows"],
            serde_json::json!([{"from": "task.second", "to": "task.first", "kind": "spec"}])
        );
    }

    /// A caller builds against a shape, so the document says which shape it
    /// is and which slice it answers.
    #[test]
    fn the_data_names_its_reading_and_the_slice_it_answers() {
        let payload = ok(
            &mut a_chain(),
            &["--json", "graph", "--match", "blocker", "--tag", "parser"],
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&payload).unwrap_or_else(|_| panic!("not JSON: {payload}"));

        assert_eq!(parsed["v"], 4);
        assert_eq!(parsed["slice"]["match"], "blocker");
        assert_eq!(parsed["slice"]["tag"], serde_json::json!(["parser"]));
        assert_eq!(parsed["slice"]["archive"], false);
        assert!(parsed["slice"].get("for").is_none(), "{payload}");
        assert_eq!(parsed["nodes"]["count"], 1);
        assert_eq!(parsed["nodes"]["rows"][0]["id"], "task.first");
        assert_eq!(parsed["nodes"]["rows"][0]["degree"], 0);
        assert_eq!(
            parsed["edges"]["count"], 0,
            "the waiter is off the map, so the line to it is not drawn"
        );
    }

    /// A record's envelope and body are most of its bytes and none of the
    /// graph, so they travel only when a caller asks for them.
    #[test]
    fn a_record_travels_whole_only_when_it_is_asked_for() {
        let bare = ok(&mut a_chain(), &["--json", "graph"]);
        let bare: serde_json::Value = serde_json::from_str(&bare).unwrap();
        assert!(bare["nodes"]["rows"][0].get("body").is_none(), "{bare}");

        let whole = ok(&mut a_chain(), &["--json", "graph", "--full"]);
        let whole: serde_json::Value = serde_json::from_str(&whole).unwrap();
        let carried = &whole["nodes"]["rows"][0];
        assert!(carried.get("body").is_some(), "{whole}");
        assert_eq!(
            carried["fields"]["rows"][0],
            serde_json::json!(["id", "task.first"])
        );
    }

    /// The same ask, printed: the envelope and the body under the tiles
    /// they belong to.
    #[test]
    fn the_printed_graph_carries_the_records_under_the_tiles() {
        let printed = ok(
            &mut a_chain(),
            &["graph", "--focus", "task.stray", "--full"],
        );
        assert_reply(
            printed,
            serde_json::json!({
              "nodes": {
                "count": 1,
                "rows": [
                  {
                    "id": "task.stray",
                    "type": "task",
                    "state": "open",
                    "ready": true,
                    "archived": false,
                    "degree": 0,
                    "priority": null,
                    "created": "2026-08-24",
                    "epic": null,
                    "title": "Another line of work",
                    "fields": {
                      "rows": [
                        [
                          "id",
                          "task.stray"
                        ],
                        [
                          "type",
                          "task"
                        ],
                        [
                          "state",
                          "open"
                        ],
                        [
                          "title",
                          "Another line of work"
                        ],
                        [
                          "created",
                          "2026-08-24"
                        ],
                        [
                          "updated",
                          "2026-08-25"
                        ]
                      ]
                    }
                  }
                ]
              },
              "edges": {
                "count": 0,
                "rows": []
              }
            }),
        );
    }

    /// A slice narrows the graph itself, so the same flags answer the same
    /// tiles whether the caller reads them or looks at them.
    #[test]
    fn every_slice_flag_narrows_what_the_verb_answers() {
        let mut storage = a_chain();
        for filter in [
            ["graph", "--tag", "parser"],
            ["graph", "--focus", "task.first"],
        ] {
            assert_reply(
                ok(&mut storage, &filter),
                serde_json::json!({
                  "nodes": {
                    "count": 2,
                    "rows": [
                      {
                        "id": "task.first",
                        "type": "task",
                        "state": "open",
                        "ready": true,
                        "archived": false,
                        "degree": 1,
                        "priority": null,
                        "created": "2026-08-24",
                        "epic": null,
                        "title": "The blocker"
                      },
                      {
                        "id": "task.second",
                        "type": "task",
                        "state": "open",
                        "ready": false,
                        "archived": false,
                        "degree": 1,
                        "priority": null,
                        "created": "2026-08-24",
                        "epic": null,
                        "title": "The waiter"
                      }
                    ]
                  },
                  "edges": {
                    "count": 1,
                    "rows": [
                      {
                        "from": "task.first",
                        "to": "task.second",
                        "kind": "waits"
                      }
                    ]
                  }
                }),
            );
        }
    }

    #[test]
    fn graph_membership_does_not_claim_an_external_prerequisite() {
        let mut storage = a_chain();
        let graph = reply_value(ok(&mut storage, &["graph", "--for", "task.second"]));
        assert_eq!(graph["nodes"]["count"], 1);
        assert_eq!(graph["nodes"]["rows"][0]["id"], "task.second");
        assert_eq!(graph["nodes"]["rows"][0]["ready"], false);
        assert_eq!(graph["edges"]["count"], 0);
    }

    /// Omitting structural rows would describe a different graph.
    #[test]
    fn both_formats_carry_the_complete_graph() {
        let mut storage = many_open_tasks(anb_core::encode::ROW_BOUND + 2);

        let printed = ok(&mut storage, &["graph"]);
        let toon = reply_value(printed);
        assert_eq!(
            toon["nodes"]["rows"].as_array().unwrap().len(),
            anb_core::encode::ROW_BOUND + 2
        );

        let payload = ok(&mut storage, &["--json", "graph"]);
        let parsed: serde_json::Value =
            serde_json::from_str(&payload).unwrap_or_else(|_| panic!("not JSON: {payload}"));
        let counted = anb_core::encode::ROW_BOUND + 2;
        assert_eq!(toon, parsed, "one complete graph in both formats");
        assert_eq!(parsed["nodes"]["count"], counted);
        assert_eq!(
            parsed["nodes"]["rows"].as_array().expect("rows").len(),
            counted,
            "and the document carries all of them"
        );
    }

    /// A body naming a record is an edge, so the notebook's own prose is
    /// part of its graph and not only the fields an envelope declares.
    #[test]
    fn a_record_named_in_another_s_prose_is_an_edge() {
        let mut storage = storage_with(&[
            (
                "tasks/task.one.md".to_owned(),
                record_file(
                    "task.one",
                    "task",
                    "open",
                    "The one that names",
                    &[],
                    "The reasoning lives in task.two.",
                ),
            ),
            open_task("task.two", "The one that is named", &[]),
        ]);

        let printed = ok(&mut storage, &["graph"]);
        assert!(
            printed.contains("task.one,task.two,mentions"),
            "the prose edge is drawn: {printed}"
        );
    }

    /// A pair of records is one edge however many times the notebook says
    /// so. A blocker whose own prose names the record waiting on it states
    /// one relation twice, and two edges would weigh that pair twice in
    /// every degree derived from the graph.
    #[test]
    fn a_relation_stated_twice_is_still_one_edge() {
        let mut storage = storage_with(&[
            (
                "tasks/task.blocker.md".to_owned(),
                record_file(
                    "task.blocker",
                    "task",
                    "open",
                    "The blocker",
                    &[],
                    "This one clears the way for task.waiter.",
                ),
            ),
            open_task("task.waiter", "The waiter", &["blocked-by: task.blocker"]),
        ]);

        let printed = ok(&mut storage, &["graph"]);
        assert_reply(
            printed,
            serde_json::json!({"edges":{"count":1,"omitted":0,"rows":[{"from":"task.blocker","to":"task.waiter","kind":"waits"}]}}),
        );
    }

    /// A kind the notebook has no word for is refused by the flag that
    /// carried it, rather than answered with a graph of nothing — which
    /// reads exactly like a notebook holding no such work.
    #[test]
    fn a_type_that_is_no_kind_of_record_is_refused_by_the_flag() {
        let Err(refused) = Cli::try_parse_from(["anb", "graph", "--type", "tsk"]) else {
            panic!("a kind the notebook cannot name must not parse");
        };
        let refusal = refused.to_string();

        assert!(refusal.contains("tsk"), "{refusal}");
        assert!(
            refusal.contains("task, decision, note, question"),
            "and it names the kinds there are: {refusal}"
        );
    }

    /// A slice a document does not name reads as a notebook that holds
    /// nothing else, which is a true-sounding answer to a question the
    /// caller never asked.
    #[test]
    fn the_document_names_the_kinds_it_was_narrowed_to() {
        let payload = ok(
            &mut a_chain(),
            &["--json", "graph", "--type", "task,decision"],
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&payload).unwrap_or_else(|_| panic!("not JSON: {payload}"));

        assert_eq!(
            parsed["slice"]["type"],
            serde_json::json!(["task", "decision"])
        );
    }

    /// A hint that cut a slice has to lift that same slice, or it names a
    /// different graph than the one the reader is looking at.
    #[test]
    fn the_truncation_hint_lifts_the_slice_it_cut() {
        let mut storage = many_open_tasks(anb_core::encode::ROW_BOUND + 1);
        let printed = ok(&mut storage, &["graph", "--archive"]);
        assert!(
            printed.contains("anb graph --archive --all"),
            "the hint carries the slice: {printed}"
        );
    }
}

mod unknown_verbs {
    use super::*;

    #[test]
    fn a_near_miss_verb_answers_the_recovery_payload_with_the_nearest_command() {
        let Err(error) = Cli::try_parse_from(["anb", "archiv"]) else {
            panic!("an unknown verb must not parse");
        };
        let recovery =
            anb::recovery::parse_recovery(&error).expect("an unknown verb joins the catalog");
        assert_reply(
            text::render_recovery(&recovery),
            serde_json::json!({
              "error": "unknown-command",
              "message": "`archiv` is not an anb command",
              "try": [
                "anb archive --help",
                "anb --help"
              ]
            }),
        );
    }

    #[test]
    fn a_verb_near_nothing_still_points_at_help() {
        let Err(error) = Cli::try_parse_from(["anb", "zzz"]) else {
            panic!("an unknown verb must not parse");
        };
        let recovery = anb::recovery::parse_recovery(&error).unwrap();
        assert_reply(
            text::render_recovery(&recovery),
            serde_json::json!({
              "error": "unknown-command",
              "message": "`zzz` is not an anb command",
              "try": [
                "anb --help"
              ]
            }),
        );
    }

    /// `--help` is what an agent reads before it types anything, so a flag
    /// carrying no help is a flag only the source explains.
    #[test]
    fn every_verb_and_every_flag_it_takes_says_what_it_is_for() {
        use clap::CommandFactory;
        let command = Cli::command();
        for verb in command.get_subcommands() {
            assert!(
                verb.get_about().is_some(),
                "`{}` has no description",
                verb.get_name()
            );
            for flag in verb.get_arguments().filter(|arg| arg.get_long().is_some()) {
                assert!(
                    flag.get_help().is_some(),
                    "`{} --{}` has no help",
                    verb.get_name(),
                    flag.get_long().unwrap_or_default()
                );
            }
        }
    }

    #[test]
    fn help_stays_claps() {
        let Err(help) = Cli::try_parse_from(["anb", "--help"]) else {
            panic!("--help renders through clap's error path");
        };
        assert!(anb::recovery::parse_recovery(&help).is_none());
    }

    #[test]
    fn a_missing_argument_has_a_structured_recovery() {
        let Err(missing_arg) = Cli::try_parse_from(["anb", "block", "task.demo"]) else {
            panic!("a missing argument must not parse");
        };
        let recovery = anb::recovery::parse_recovery(&missing_arg).unwrap();
        assert_eq!(recovery.code, "invalid-argument");
        assert!(recovery.tries.contains(&"anb --help".to_owned()));
    }

    #[test]
    fn an_unknown_flag_suggests_hyphen_safe_prose_inputs_only_for_prose_commands() {
        let error = Cli::try_parse_from(["anb", "edit", "task.demo", "--bdy"])
            .err()
            .unwrap();
        let recovery = anb::recovery::parse_recovery(&error).unwrap();
        assert_eq!(recovery.code, "invalid-argument");
        assert!(
            recovery
                .tries
                .iter()
                .any(|command| command.contains("--body="))
        );
        assert_eq!(recovery.tries[0], "anb edit --help");
        let error = Cli::try_parse_from(["anb", "comment", "task.demo", "entry", "--unknown"])
            .err()
            .unwrap();
        let recovery = anb::recovery::parse_recovery(&error).unwrap();
        assert!(
            recovery
                .tries
                .iter()
                .any(|command| command.contains(" -- \""))
        );
        for arguments in [
            vec!["anb", "status", "--all"],
            vec!["anb", "show", "task.demo", "--body"],
            vec!["anb", "ready", "--body"],
        ] {
            let error = Cli::try_parse_from(&arguments).err().unwrap();
            let recovery = anb::recovery::parse_recovery(&error).unwrap();
            assert_eq!(recovery.tries, [format!("anb {} --help", arguments[1])]);
        }
    }
}

mod check_bounds {
    use super::*;

    fn many_broken_records(count: usize) -> MemoryStorage {
        let files: Vec<(String, String)> = (0..count)
            .map(|n| {
                let id = format!("task.b{n:02}");
                (
                    format!("tasks/{id}.md"),
                    record_file(&id, "task", "cancelled", "A demo record", &[], ""),
                )
            })
            .collect();
        storage_with(&files)
    }

    #[test]
    fn the_findings_table_is_bounded_with_the_restore_hint() {
        let mut storage = many_broken_records(22);
        let out = ok(&mut storage, &["check"]);
        assert_reply(
            &out,
            serde_json::json!({"count":22,"omitted":2,"more":"anb check --all"}),
        );
        assert_eq!(reply_value(out)["findings"].as_array().unwrap().len(), 20);
    }

    /// The same bound in the rendering an agent parses rather than reads.
    #[test]
    fn the_json_findings_are_bounded_like_the_table() {
        let mut storage = many_broken_records(22);
        let checked: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["check", "--json"])).unwrap();
        assert_eq!(checked["count"], serde_json::json!(22));
        assert_eq!(checked["findings"].as_array().unwrap().len(), 20);
    }
}

/// The law every reply obeys: what a command names in passing — who was
/// unblocked, who cites a record, what blocks a removal, how a cycle runs —
/// is stated as a count and a bounded head, so no answer grows with the
/// notebook.
mod bounded_consequences {
    use super::*;

    /// One past the twenty a reply shows, so the count and the remainder
    /// are both wrong under any off-by-one.
    const MANY: usize = 21;

    fn hub_with_dependents(count: usize) -> MemoryStorage {
        let mut files = vec![(
            "tasks/task.hub.md".to_owned(),
            record_file("task.hub", "task", "active", "The hub", &[], ""),
        )];
        files.extend((0..count).map(|n| {
            open_task(
                &format!("task.d{n:02}"),
                "A demo record",
                &["blocked-by: task.hub"],
            )
        }));
        storage_with(&files)
    }

    fn citers_of(id: &str, count: usize) -> MemoryStorage {
        let mut files = vec![(
            format!("notes/{id}.md"),
            record_file(id, "note", "active", "The cited note", &["kind: fact"], ""),
        )];
        files.extend((0..count).map(|n| {
            (
                format!("tasks/task.c{n:02}.md"),
                record_file(
                    &format!("task.c{n:02}"),
                    "task",
                    "open",
                    "A demo record",
                    &[],
                    &format!("It follows {id}.\n"),
                ),
            )
        }));
        storage_with(&files)
    }

    #[test]
    fn a_close_counts_what_it_unblocked_and_names_a_bounded_head() {
        let mut storage = hub_with_dependents(MANY);
        let out = ok(
            &mut storage,
            &["close", "task.hub", "--body", "Completed and checked."],
        );
        assert_reply(
            out,
            serde_json::json!({"unblocked":{"count":21,"omitted":1,"rows":(0..20).map(|n|format!("task.d{n:02}")).collect::<Vec<_>>()}}),
        );
    }

    #[test]
    fn a_close_json_carries_the_count_beside_the_bounded_rows() {
        let mut storage = hub_with_dependents(MANY);
        let out = ok(
            &mut storage,
            &[
                "close",
                "task.hub",
                "--body",
                "Completed and checked.",
                "--json",
            ],
        );
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["unblocked"]["count"], serde_json::json!(MANY));
        assert_eq!(
            value["unblocked"]["rows"],
            serde_json::json!(
                (0..20)
                    .map(|n| format!("task.d{n:02}"))
                    .collect::<Vec<String>>()
            )
        );
    }

    #[test]
    fn a_view_bounds_who_cites_the_record_and_says_what_lifts_the_bound() {
        let mut storage = citers_of("note.magnet", MANY);
        assert_reply(
            ok(&mut storage, &["show", "note.magnet"]),
            serde_json::json!({"mentioned-by":{"count":21,"omitted":1,"rows":(0..20).map(|n|format!("task.c{n:02}")).collect::<Vec<_>>()},"more":"anb show note.magnet --all"}),
        );
    }

    #[test]
    fn a_view_all_names_every_record_that_cites_this_one() {
        let mut storage = citers_of("note.magnet", MANY);
        assert_reply(
            ok(&mut storage, &["show", "note.magnet", "--all"]),
            serde_json::json!({"mentioned-by":{"count":21,"omitted":0,"rows":(0..21).map(|n|format!("task.c{n:02}")).collect::<Vec<_>>()}}),
        );
    }

    #[test]
    fn a_refusal_names_the_detail_lines_it_cut_in_its_own_units() {
        // Each record holds the id twice — through its edge and through its
        // body — so the records the message counts and the lines the
        // details name are two different totals.
        let mut files = vec![(
            "tasks/task.hub.md".to_owned(),
            record_file("task.hub", "task", "active", "The hub", &[], ""),
        )];
        files.extend((0..11).map(|n| {
            (
                format!("tasks/task.c{n:02}.md"),
                record_file(
                    &format!("task.c{n:02}"),
                    "task",
                    "open",
                    "A demo record",
                    &["blocked-by: task.hub"],
                    "It also follows task.hub in prose.\n",
                ),
            )
        }));
        let mut storage = storage_with(&files);
        let out = refused(&mut storage, &["delete", "task.hub"]);
        assert_reply(
            &out,
            serde_json::json!({"error":"still-referenced","message":"`task.hub` is still referenced by 11 records","count":22,"omitted":2}),
        );
        let value = reply_value(out);
        assert_eq!(value["findings"].as_array().unwrap().len(), 20);
        assert_eq!(value["try"].as_array().unwrap().len(), 11);
    }

    #[test]
    fn a_cycle_finding_bounds_the_walk_it_names() {
        let files: Vec<(String, String)> = (0..MANY)
            .map(|n| {
                open_task(
                    &format!("task.r{n:02}"),
                    "A demo record",
                    &[&format!("blocked-by: task.r{:02}", (n + 1) % MANY)],
                )
            })
            .collect();
        let mut storage = storage_with(&files);
        let out = ok(&mut storage, &["check"]);
        let value = reply_value(out);
        let finding = value["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["code"] == "block-cycle")
            .unwrap()["message"]
            .as_str()
            .unwrap();
        assert!(
            finding.contains("closes the cycle task.r00 → task.r01 → "),
            "{finding}"
        );
        // The walk repeats its first id, so a 21-task ring is 22 steps.
        assert!(finding.ends_with("task.r19 → … 2 more"), "{finding}");
    }
}

mod json_maintenance_surface {
    use super::*;

    /// Every verb answers in JSON too, and an agent keying on `ok` and the
    /// fields beside it gets the same reply the text carries.
    #[test]
    fn every_edge_and_hold_verb_carries_its_own_json_shape() {
        let mut storage = storage_with(&[
            open_task("task.demo", "A demo record", &[]),
            open_task("task.other", "Another record", &[]),
            (
                "questions/question.doubt.md".to_owned(),
                record_file("question.doubt", "question", "open", "A doubt", &[], ""),
            ),
        ]);
        for (line, expected) in [
            (
                vec!["block", "task.demo", "task.other", "--json"],
                r#"{"ok":"block","id":"task.demo","on":"task.other","already":false}"#,
            ),
            (
                vec!["unblock", "task.demo", "task.other", "--json"],
                r#"{"ok":"unblock","id":"task.demo","on":"task.other","already":false}"#,
            ),
            (
                vec![
                    "hold",
                    "task.demo",
                    "--reason",
                    "waiting on the owner",
                    "--until",
                    "2026-09-09",
                    "--json",
                ],
                r#"{"ok":"hold","id":"task.demo","already":false,"until":"2026-09-09"}"#,
            ),
            (
                vec!["unhold", "task.demo", "--json"],
                r#"{"ok":"unhold","id":"task.demo","already":false}"#,
            ),
            (
                vec![
                    "close",
                    "question.doubt",
                    "--reason",
                    "the ground it stood on is gone",
                    "--json",
                ],
                r#"{"ok":"close","id":"question.doubt","from":"open","to":"closed","already":false,"unblocked":{"count":0,"rows":[]},"open-questions":{"count":0,"rows":[]}}"#,
            ),
            (
                vec!["delete", "task.other", "--json"],
                r#"{"ok":"delete","id":"task.other","paths":["tasks/task.other.md"]}"#,
            ),
        ] {
            assert_reply(
                ok(&mut storage, &line),
                serde_json::from_str(expected).unwrap(),
            );
        }
    }

    #[test]
    fn check_carries_findings_with_the_line_omitted_when_absent() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "cancelled", "A demo record", &[], ""),
        )]);
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["check", "--json"])).unwrap();
        assert_eq!(value["count"], serde_json::json!(1));
        let finding = &value["findings"][0];
        assert_eq!(finding["file"], serde_json::json!("tasks/task.demo.md"));
        assert_eq!(finding["line"], serde_json::json!(4));
        assert_eq!(finding["severity"], serde_json::json!("error"));
        assert_eq!(finding["code"], serde_json::json!("bad-value"));
        assert!(
            finding.get("repair").is_none(),
            "a state no command writes has no repair: {finding}"
        );

        let mut duplicated = storage_with(&[
            (
                "tasks/task.demo.md".to_owned(),
                record_file("task.demo", "task", "open", "A demo record", &[], ""),
            ),
            (
                "notes/note.twin.md".to_owned(),
                record_file("task.demo", "note", "active", "A twin claim", &[], ""),
            ),
        ]);
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut duplicated, &["check", "--json"])).unwrap();
        let whole_file = value["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["code"] == serde_json::json!("duplicate-id"))
            .expect("two files claim one id");
        assert!(
            whole_file.get("line").is_none(),
            "a whole-file finding carries no line: {whole_file}"
        );
    }

    #[test]
    fn archiving_a_task_leaves_linked_knowledge_live_in_json() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md".to_owned(),
                record_file(
                    "task.demo",
                    "task",
                    "closed",
                    "A demo record",
                    &["link: note note.report"],
                    "",
                ),
            ),
            (
                "notes/note.report.md".to_owned(),
                record_file(
                    "note.report",
                    "note",
                    "active",
                    "The report",
                    &["from: task.demo"],
                    "",
                ),
            ),
        ]);
        assert_reply(
            ok(&mut storage, &["archive", "task.demo", "--json"]),
            serde_json::json!({
              "ok": "archive",
              "id": "task.demo",
              "from": "tasks/task.demo.md",
              "to": "archive/tasks/task.demo.md",
              "already": false
            }),
        );
    }

    #[test]
    fn archive_confirms_the_move() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "closed", "A demo record", &[], ""),
        )]);
        assert_reply(
            ok(&mut storage, &["archive", "task.demo", "--json"]),
            serde_json::json!({
              "ok": "archive",
              "id": "task.demo",
              "from": "tasks/task.demo.md",
              "to": "archive/tasks/task.demo.md",
              "already": false
            }),
        );
    }

    #[test]
    fn archiving_a_task_preserves_the_linked_notes_bytes() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md".to_owned(),
                record_file(
                    "task.demo",
                    "task",
                    "closed",
                    "A demo record",
                    &["link: note note.report"],
                    "",
                ),
            ),
            (
                "notes/note.report.md".to_owned(),
                record_file(
                    "note.report",
                    "note",
                    "active",
                    "The report",
                    &["from: task.demo"],
                    "",
                ),
            ),
        ]);
        let knowledge = anb_core::Storage::read(&storage, "notes/note.report.md").unwrap();
        assert_reply(
            ok(&mut storage, &["archive", "task.demo"]),
            serde_json::json!({"ok":"archive","id":"task.demo","from":"tasks/task.demo.md","to":"archive/tasks/task.demo.md"}),
        );
        assert_eq!(
            anb_core::Storage::read(&storage, "notes/note.report.md").unwrap(),
            knowledge
        );
        assert!(
            anb_core::Storage::list(&storage, "archive/notes")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn restore_confirms_the_move_back() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "closed", "A demo record", &[], ""),
        )]);
        assert_reply(
            ok(&mut storage, &["restore", "task.demo", "--json"]),
            serde_json::json!({
              "ok": "restore",
              "id": "task.demo",
              "from": "archive/tasks/task.demo.md",
              "to": "tasks/task.demo.md",
              "already": false
            }),
        );
    }

    #[test]
    fn edit_names_what_changed_and_the_nudge() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(
                &mut storage,
                &[
                    "edit",
                    "task.demo",
                    "--title",
                    "Sharper",
                    "--body",
                    "Blocked by task.ghost.",
                    "--json",
                ],
            ),
            serde_json::json!({
              "ok": "edit",
              "id": "task.demo",
              "changed": [
                "title",
                "body"
              ],
              "already": false,
              "dangling-mention": {
                "count": 1,
                "rows": [
                  "task.ghost"
                ]
              }
            }),
        );
    }

    #[test]
    fn a_narrowed_listing_answers_the_same_rows_as_the_whole() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_reply(
            ok(&mut storage, &["list", "--match", "demo", "--json"]),
            serde_json::json!({
              "count": 1,
              "records": [
                {
                  "id": "task.demo",
                  "state": "open",
                  "title": "A demo record"
                }
              ]
            }),
        );
    }

    #[test]
    fn debt_answers_the_signals_as_rows() {
        let mut storage = storage_with(&[(
            "notes/note.cites.md".to_owned(),
            record_file(
                "note.cites",
                "note",
                "active",
                "A demo record",
                &[],
                "cites task.gone here.\n",
            ),
        )]);
        assert_reply(
            ok(&mut storage, &["debt", "--json"]),
            serde_json::json!({
              "count": 1,
              "debt": [
                {
                  "code": "dangling-mention",
                  "id": "note.cites",
                  "target": "task.gone",
                  "line": "dangling-mention: note.cites -> task.gone"
                }
              ]
            }),
        );
    }
}

/// Which verbs the user's notebook accepts. It holds knowledge that
/// outlives a repository, so it holds decisions and notes and nothing to
/// work on.
mod the_global_scope {
    use super::*;
    use std::collections::BTreeSet;

    /// The verbs the Global Notebook exists for: recording knowledge and
    /// reading it back.
    const NAMED: &[&[&str]] = &[
        &["add", "decision", "A rule"],
        &["add", "note", "A fact"],
        &["retire", "note.demo"],
        &["show", "note.demo"],
        &["list"],
        &["list", "--match", "rule"],
    ];

    /// The verbs the rule admits beyond those, each for a reason the
    /// Global Notebook would be crippled without: knowledge corrected in
    /// place and verified, settled knowledge filed and brought back,
    /// mistakes erased, and the whole-notebook reads. Their rows are empty
    /// of tasks and questions rather than absent, so the output contract
    /// is one contract.
    const ALSO_ADMITTED: &[&[&str]] = &[
        &["edit", "note.demo", "--title", "A sharper fact"],
        &["comment", "note.demo", "--body", "The conclusion"],
        &["check"],
        &["archive", "note.demo"],
        &["restore", "note.demo"],
        &["delete", "note.demo"],
        &["status"],
        &["ready"],
        &["graph"],
        &["debt"],
        &["skill"],
        &["recall"],
        &["hook"],
        &["import", "records.json"],
        &["migrate"],
    ];

    /// The verb refused the user's notebook for a reason of its own: setup
    /// installs where a session starts, and no session starts in the
    /// user's home notebook.
    const NO_HOME: &[&[&str]] = &[&["setup"]];

    /// Every verb that creates or moves a task or a question.
    const WORK: &[&[&str]] = &[
        &["add", "task", "A task"],
        &["start", "task.demo"],
        &["submit", "task.demo"],
        &["close", "task.demo", "--body", "Completed and checked."],
        &["reopen", "task.demo"],
        &["hold", "task.demo", "--reason", "waiting"],
        &["unhold", "task.demo"],
        &["block", "task.demo", "task.other"],
        &["unblock", "task.demo", "task.other"],
        &["add", "question", "A doubt"],
        &["close", "question.demo", "--reason", "moot"],
    ];

    fn parsed(line: &[&str]) -> Command {
        let mut args = vec!["anb"];
        args.extend_from_slice(line);
        Cli::try_parse_from(args)
            .unwrap_or_else(|_| panic!("the test drives a well-formed command line: {line:?}"))
            .command
    }

    fn accepts(lines: &[&[&str]]) {
        for line in lines {
            assert!(
                anb::scope::refused_privately(&parsed(line), true).is_none(),
                "`anb {} --global` carries knowledge and belongs in either scope",
                line.join(" ")
            );
        }
    }

    #[test]
    fn the_six_verbs_the_scope_was_asked_for_reach_the_users_notebook() {
        accepts(NAMED);
    }

    #[test]
    fn the_verbs_the_rule_admits_beyond_them_reach_it_too() {
        accepts(ALSO_ADMITTED);
    }

    #[test]
    fn a_verb_that_writes_work_is_refused_the_users_notebook() {
        for line in WORK.iter().chain(NO_HOME) {
            let refused = anb::scope::refused_privately(&parsed(line), true)
                .unwrap_or_else(|| panic!("`anb {} --global` must be refused", line.join(" ")));
            assert_eq!(refused.code(), "invalid-argument");
            assert!(
                refused.to_string().starts_with(line[0]),
                "the refusal names the verb that was refused: {refused}"
            );
        }
    }

    /// The refusal is the scope's, never the verb's: without the flag the
    /// same command line is the project's to serve.
    #[test]
    fn no_verb_is_refused_the_project() {
        for line in WORK.iter().chain(NAMED).chain(ALSO_ADMITTED).chain(NO_HOME) {
            assert!(
                anb::scope::refused_privately(&parsed(line), false).is_none(),
                "`anb {}` names no scope and must not be refused one",
                line.join(" ")
            );
        }
    }

    /// A verb missing from the tables above would be judged by neither, so
    /// the three are held against the whole command surface.
    #[test]
    fn every_verb_is_placed_on_one_side_of_the_rule() {
        let placed: BTreeSet<&str> = NAMED
            .iter()
            .chain(ALSO_ADMITTED)
            .chain(WORK)
            .chain(NO_HOME)
            .map(|line| line[0])
            .collect();
        let surface = Cli::command();
        let surface: BTreeSet<&str> = surface
            .get_subcommands()
            .map(clap::Command::get_name)
            .collect();
        assert_eq!(placed, surface);
    }
}

/// The user's notebook stands behind every surface of the project's, not
/// only its Status: what it holds is no dangling citation in a reply and
/// no dangling link under the gate.
mod the_users_notebook_behind_every_surface {
    use super::*;

    fn users_notebook() -> MemoryStorage {
        storage_with(&[(
            "decisions/decision.tabs.md".to_owned(),
            record_file(
                "decision.tabs",
                "decision",
                "active",
                "Tabs",
                &["by: Reader"],
                "",
            ),
        )])
    }

    #[test]
    fn recording_the_rule_that_shadows_a_global_one_is_nudged_about_nothing() {
        let user = users_notebook();
        let mut storage = MemoryStorage::new();
        let reply = run_behind(
            &mut storage,
            Some(&user),
            &[
                "add",
                "decision",
                "Spaces here",
                "--body",
                "This repository decides otherwise, against decision.tabs.",
            ],
            &missing_report,
        )
        .expect("the command must succeed");
        assert_reply(
            reply,
            serde_json::json!({
              "ok": "add",
              "id": "decision.spaces-here",
              "path": "decisions/decision.spaces-here.md"
            }),
        );
    }

    #[test]
    fn a_personal_rule_does_not_validate_a_shared_link() {
        let user = users_notebook();
        let mut storage = storage_with(&[(
            "decisions/decision.spaces.md".to_owned(),
            record_file(
                "decision.spaces",
                "decision",
                "active",
                "Spaces here",
                &["link: against decision.tabs"],
                "",
            ),
        )]);
        let output = run_behind(
            &mut storage,
            Some(&user),
            &["check", "--json"],
            &missing_report,
        )
        .expect("check returns its findings");
        let checked: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(checked["count"], 1);
        assert_eq!(checked["findings"][0]["code"], "dangling-ref");
    }
}
