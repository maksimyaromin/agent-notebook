//! The thin e2e pass over golden outputs: a command line in, the rendered
//! reply out, over an in-memory notebook. Every expected text derives from
//! the output contract, never from running the code.

use anb::cli::{Cli, Command};
use anb::reply::{Host, execute};
use anb::{json, text};
use anb_core::MemoryStorage;
use anb_core::StorageError;
use clap::{CommandFactory as _, Parser};
use insta::assert_snapshot;
use std::fmt::Write as _;

const TODAY: &str = "2026-08-28";
const GIT_IDENTITY: &str = "Maks";

/// Parse and run one command line; `Ok` is stdout, `Err` is the payload a
/// failure prints.
fn run(storage: &mut MemoryStorage, line: &[&str]) -> Result<String, String> {
    run_reading(storage, line, &[])
}

/// [`run`] with files the shell may read by path — the report `--note`
/// ingests lives outside the notebook, so no Storage serves it. A path the
/// list does not name is a file that is not there.
fn run_reading(
    storage: &mut MemoryStorage,
    line: &[&str],
    reports: &[(&str, &str)],
) -> Result<String, String> {
    run_with(storage, line, &|path: &str| {
        reports
            .iter()
            .find(|(named, _)| *named == path)
            .map(|(_, text)| (*text).to_owned())
            .ok_or_else(|| StorageError::NotFound {
                path: path.to_owned(),
            })
    })
}

/// One command line against a host whose one reach outside the notebook —
/// the report it reads — the case chooses.
fn run_with(
    storage: &mut MemoryStorage,
    line: &[&str],
    read_report: &dyn Fn(&str) -> Result<String, StorageError>,
) -> Result<String, String> {
    run_behind(storage, None, line, read_report)
}

/// [`run_with`] against a project whose user's notebook, when one is given,
/// stands behind it.
fn run_behind(
    storage: &mut MemoryStorage,
    user_notebook: Option<&dyn anb_core::Storage>,
    line: &[&str],
    read_report: &dyn Fn(&str) -> Result<String, StorageError>,
) -> Result<String, String> {
    let mut args = vec!["anb"];
    args.extend_from_slice(line);
    let cli = Cli::try_parse_from(args).expect("the test drives a well-formed command line");
    let subject = anb::recovery::subject(&cli.command);
    let wants_json = cli.json;
    let host = Host {
        git_by: || Some(GIT_IDENTITY.to_owned()),
        read_report,
        lost_proofs: &nothing_lost,
        user_notebook,
        today: TODAY,
    };
    match execute(cli.command, storage, host) {
        Ok(reply) => Ok(if wants_json {
            json::render(&reply)
        } else {
            text::render(&reply, TODAY)
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
        git_by: || None,
        read_report: &missing_report,
        lost_proofs: &nothing_lost,
        user_notebook: None,
        today: "not-a-date",
    }
}

/// The world of a case that is not about reconciliation: it still holds
/// every proof the notebook cites.
fn nothing_lost(_: &[anb_core::CitedProof]) -> Vec<anb_core::CitedProof> {
    Vec::new()
}

/// The reader for tests that never pass `--note`: every path is absent.
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

        assert_snapshot!(
            run_with(&mut storage, &["close", "task.demo", "--note", "report.md"], &unreadable)
                .expect_err("bytes outside UTF-8 are no proof"),
            @r#"
        error[invalid-argument]: note: `report.md` is not UTF-8
        try: anb close task.demo --note <path>
        try: anb close task.demo --no-proof
        try: anb close task.demo --reason "<why>"
        "#
        );
        assert!(
            ok(&mut storage, &["show", "task.demo"]).contains("state: active"),
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
        assert_eq!(
            output,
            "ok: add task.grammar-parser-accepts-fenced-envelopes — tasks/task.grammar-parser-accepts-fenced-envelopes.md\n"
        );
        let written = storage
            .read("tasks/task.grammar-parser-accepts-fenced-envelopes.md")
            .unwrap();
        assert!(
            written.contains("\nby: Maks\n"),
            "git identity fills `by` when no flag names one: {written}"
        );
    }

    /// Whether the value is an integer is the command line\'s question;
    /// whether it is a priority is the notebook\'s, and both answers reach
    /// the caller in one shape.
    #[test]
    fn a_priority_outside_the_scale_is_refused_the_same_way_at_any_size() {
        for out_of_range in ["9", "300"] {
            assert_eq!(
                refused(
                    &mut MemoryStorage::new(),
                    &["add", "task", "A triaged task", "--priority", out_of_range]
                ),
                format!(
                    "error[invalid-argument]: priority: {out_of_range} is not 0\u{2013}4\ntry: anb add task \"<title>\"\n"
                )
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
                "title: yields an empty id \u{2014} pass an explicit id",
            ),
        ] {
            let mut storage = MemoryStorage::new();
            let refusal = refused(&mut storage, &line);
            assert!(
                refusal.starts_with(&format!("error[invalid-argument]: {reason}\n")),
                "`anb {}` answered {refusal}",
                line.join(" ")
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
        assert_eq!(
            refused(
                &mut storage,
                &[
                    "hold",
                    "task.demo",
                    "--reason",
                    "waiting",
                    "--until",
                    "soon"
                ]
            ),
            "error[invalid-argument]: hold-until: `soon` is not `YYYY-MM-DD` or an RFC 3339 timestamp\ntry: anb hold task.demo --reason \"<why>\"\n"
        );
    }

    #[test]
    fn start_answers_the_transition() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_eq!(
            ok(&mut storage, &["start", "task.demo"]),
            "ok: start task.demo — open\u{2192}active\n"
        );
    }

    #[test]
    fn a_replay_names_the_standing_state() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        ok(&mut storage, &["start", "task.demo"]);
        assert_eq!(
            ok(&mut storage, &["start", "task.demo"]),
            "ok: start task.demo — active (already)\n"
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
        assert_snapshot!(
            ok(&mut storage, &["close", "task.demo", "--pr", "https://example.com/pull/7"]),
            @r"
        ok: close task.demo — active→closed
        unblocked[1]: task.waiting
        open-questions[1]: question.doubt
        "
        );
    }

    #[test]
    fn close_with_a_note_ingests_the_report_and_names_it() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_snapshot!(
            run_reading(
                &mut storage,
                &["close", "task.demo", "--note", "reports/demo.md"],
                &[("reports/demo.md", "# What shipped\n")],
            )
            .expect("the command must succeed"),
            @r"
        ok: close task.demo — active→closed
        report: note.report-a-demo-record
        "
        );
        // The Core owns the Note's shape; the shell's own contribution is
        // the identity it resolved from git.
        assert!(
            storage
                .read("notes/note.report-a-demo-record.md")
                .unwrap()
                .contains("\nby: Maks\n"),
            "the report is signed by whoever closed the task"
        );
    }

    #[test]
    fn close_with_a_note_naming_no_file_is_a_recovery_payload() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_snapshot!(
            run_reading(&mut storage, &["close", "task.demo", "--note", "gone.md"], &[])
                .expect_err("the command must be refused"),
            @r#"
        error[invalid-argument]: note: no file at `gone.md`
        try: anb close task.demo --note <path>
        try: anb close task.demo --no-proof
        try: anb close task.demo --reason "<why>"
        "#
        );
        assert_eq!(
            storage.read("tasks/task.demo.md").unwrap(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
            "an unreadable report closes nothing"
        );
    }

    #[test]
    fn close_with_two_proofs_names_the_conflict() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_snapshot!(
            refused(&mut storage, &["close", "task.demo", "--sha", "f00d", "--no-proof"]),
            @r#"
        error[invalid-argument]: close: pass exactly one of --note <path>, --pr <url>, --sha <sha>, --report <path>, --no-proof, --reason "<why>", or --resolved-by <id>
        try: anb close task.demo --note <path>
        try: anb close task.demo --no-proof
        try: anb close task.demo --reason "<why>"
        "#
        );
    }

    #[test]
    fn close_without_a_proof_is_a_recovery_payload() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_snapshot!(
            refused(&mut storage, &["close", "task.demo"]),
            @r#"
        error[invalid-argument]: close: pass one of --note <path>, --pr <url>, --sha <sha>, --report <path>, --no-proof, --reason "<why>", or --resolved-by <id>
        try: anb close task.demo --note <path>
        try: anb close task.demo --no-proof
        try: anb close task.demo --reason "<why>"
        "#
        );
    }

    #[test]
    fn an_invalid_transition_lists_the_valid_commands() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            refused(&mut storage, &["close", "task.demo", "--no-proof"]),
            @r#"
        error[invalid-transition]: `task.demo` is open; valid: start, close --reason
        try: anb start task.demo
        try: anb close task.demo --reason "<why>"
        "#
        );
    }

    #[test]
    fn a_close_proof_shows_among_the_valid_commands() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "review", "A demo record", &[], ""),
        )]);
        assert_snapshot!(
            refused(&mut storage, &["reopen", "task.demo"]),
            @r#"
        error[invalid-transition]: `task.demo` is review; valid: start, close, close --reason
        try: anb start task.demo
        try: anb close task.demo --note <path>
        try: anb close task.demo --no-proof
        try: anb close task.demo --reason "<why>"
        "#
        );
    }

    #[test]
    fn an_edge_that_would_cycle_walks_the_chain() {
        let mut storage = storage_with(&[
            open_task("task.a", "The first", &["blocked-by: task.b"]),
            open_task("task.b", "The second", &[]),
        ]);
        assert_snapshot!(
            refused(&mut storage, &["block", "task.b", "task.a"]),
            @r"
        error[would-cycle]: the edge would close a dependency cycle: task.b → task.a → task.b
        try: anb unblock task.a task.b
        "
        );
    }

    #[test]
    fn block_and_unblock_answer_the_edge() {
        let mut storage = storage_with(&[
            open_task("task.a", "The first", &[]),
            open_task("task.b", "The second", &[]),
        ]);
        assert_eq!(
            ok(&mut storage, &["block", "task.a", "task.b"]),
            "ok: block task.a — waits on task.b\n"
        );
        assert_eq!(
            ok(&mut storage, &["unblock", "task.a", "task.b"]),
            "ok: unblock task.a — edge on task.b erased\n"
        );
    }

    #[test]
    fn a_replayed_unblock_marks_already() {
        let mut storage = storage_with(&[open_task("task.a", "The first", &[])]);
        assert_eq!(
            ok(&mut storage, &["unblock", "task.a", "task.b"]),
            "ok: unblock task.a — edge on task.b erased (already)\n"
        );
    }

    #[test]
    fn a_citation_into_nothing_is_nudged_under_the_ok_line() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            ok(
                &mut storage,
                &["comment", "task.demo", "waits on task.ghost and task.wraith"],
            ),
            @r"
        ok: comment task.demo — logged
        dangling-mention[2]: task.ghost, task.wraith — backtick to quote, or create the record
        "
        );
    }

    #[test]
    fn an_add_body_citing_nothing_is_nudged_the_same_way() {
        let mut storage = MemoryStorage::new();
        assert_snapshot!(
            ok(
                &mut storage,
                &["add", "task", "A demo record", "--body", "Blocked by task.ghost."],
            ),
            @r"
        ok: add task.a-demo-record — tasks/task.a-demo-record.md
        dangling-mention[1]: task.ghost — backtick to quote, or create the record
        "
        );
    }

    #[test]
    fn an_invalid_record_names_its_findings() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "cancelled", "A demo record", &[], ""),
        )]);
        assert_snapshot!(
            refused(&mut storage, &["start", "task.demo"]),
            @r"
        error[invalid-record]: tasks/task.demo.md is invalid (1 findings)
          line 4: bad-value state: `cancelled` is not one of open, active, review, closed for a task
        try: anb show task.demo
        "
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
        assert_snapshot!(
            refused(&mut storage, &["start", "task.done"]),
            @r"
        error[archived]: `task.done` is archived
        try: anb show task.done
        try: anb restore task.done
        "
        );
    }

    #[test]
    fn a_wrong_typed_target_suggests_viewing_it() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            refused(&mut storage, &["block", "task.demo", "decision.d"]),
            @r"
        error[wrong-type]: `decision.d` is not a task
        try: anb show decision.d
        "
        );
    }

    #[test]
    fn block_on_a_missing_task_suggests_creating_it() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            refused(&mut storage, &["block", "task.demo", "task.ghost"]),
            @r#"
        error[dangling-ref]: blocked-by: `task.ghost` names no record
        try: anb add task "<title>" --id task.ghost
        try: anb list
        "#
        );
    }

    #[test]
    fn comment_logs_with_the_acting_hand() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_eq!(
            ok(
                &mut storage,
                &[
                    "comment",
                    "task.demo",
                    "parser done, tests next",
                    "--via",
                    "claude-code"
                ],
            ),
            "ok: comment task.demo — logged\n"
        );
        let written = anb_core::Storage::read(&storage, "tasks/task.demo.md").unwrap();
        assert!(
            written.ends_with("- 2026-08-28 claude-code: parser done, tests next\n"),
            "{written}"
        );
    }

    #[test]
    fn a_comment_without_via_signs_as_the_git_identity() {
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
        assert_eq!(
            ok(
                &mut storage,
                &[
                    "hold",
                    "task.demo",
                    "--reason",
                    "waiting for the release",
                    "--until",
                    "2026-09-01"
                ],
            ),
            "ok: hold task.demo — held until 2026-09-01\n"
        );
        assert_eq!(
            ok(&mut storage, &["unhold", "task.demo"]),
            "ok: unhold task.demo — unheld\n"
        );
    }

    #[test]
    fn hold_without_a_reason_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            refused(&mut storage, &["hold", "task.demo"]),
            @r#"
        error[invalid-argument]: hold: the reason must not be empty
        try: anb hold task.demo --reason "<why>"
        "#
        );
    }

    #[test]
    fn a_taken_id_suggests_the_next_commands() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            refused(&mut storage, &["add", "task", "Another demo", "--id", "task.demo"]),
            @r#"
        error[duplicate-id]: `task.demo` already exists at tasks/task.demo.md
        try: anb show task.demo
        try: anb add task "<title>"
        "#
        );
    }

    #[test]
    fn an_unknown_id_is_a_recovery_payload() {
        let mut storage = MemoryStorage::new();
        assert_snapshot!(
            refused(&mut storage, &["start", "task.absent"]),
            @r"
        error[unknown-id]: no record `task.absent`
        try: anb list
        "
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
        assert_eq!(
            ok(
                &mut storage,
                &["add", "note", "The new note", "--supersedes", "note.old"],
            ),
            "ok: add note.the-new-note — notes/note.the-new-note.md\n\
             superseded: note.old\n"
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
        assert_snapshot!(
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
            @r#"
        error[invalid-argument]: close: pass exactly one of --note <path>, --pr <url>, --sha <sha>, --report <path>, --no-proof, --reason "<why>", or --resolved-by <id>
        try: anb close question.doubt --resolved-by <id>
        try: anb close question.doubt --reason "<why>"
        "#
        );
        assert!(
            ok(&mut storage, &["show", "question.doubt"]).contains("state: open"),
            "a refused close closes nothing"
        );
    }
    use anb_core::Storage as _;

    #[test]
    fn add_decision_records_a_decision_and_answers_the_path() {
        let mut storage = MemoryStorage::new();
        assert_eq!(
            ok(
                &mut storage,
                &["add", "decision", "No mise toml", "--kind", "rule"]
            ),
            "ok: add decision.no-mise-toml — decisions/decision.no-mise-toml.md\n"
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
        assert_eq!(
            ok(
                &mut storage,
                &[
                    "add",
                    "decision",
                    "Rust for the CLI",
                    "--supersedes",
                    "decision.go-for-the-cli"
                ],
            ),
            "ok: add decision.rust-for-the-cli — decisions/decision.rust-for-the-cli.md\n\
             superseded: decision.go-for-the-cli\n"
        );
    }

    #[test]
    fn an_undeclared_conflict_is_nudged_not_blocked() {
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
        assert_snapshot!(
            ok(
                &mut storage,
                &[
                "add",
                "decision",
                    "Fences stay",
                    "--tag",
                    "parser",
                    "--tag",
                    "grammar"
                ],
            ),
            @r"
        ok: add decision.fences-stay — decisions/decision.fences-stay.md
        may-conflict[1]: decision.first (supolka/claude-code)
        "
        );
        assert!(
            storage.read("decisions/decision.fences-stay.md").is_ok(),
            "the nudge is a consequence in the reply, never a block"
        );
    }

    #[test]
    fn add_note_records_a_term() {
        let mut storage = MemoryStorage::new();
        assert_eq!(
            ok(&mut storage, &["add", "note", "Record", "--kind", "term"]),
            "ok: add note.record — notes/note.record.md\n"
        );
        let written = storage.read("notes/note.record.md").unwrap();
        assert!(written.contains("\nkind: term\n"), "{written}");
    }

    #[test]
    fn a_kind_outside_the_types_enum_is_a_recovery_payload() {
        let mut storage = MemoryStorage::new();
        assert_snapshot!(
            refused(&mut storage, &["add", "note", "A fact", "--kind", "law"]),
            @r#"
        error[invalid-argument]: kind: `law` is not one of fact, term, guide for a note
        try: anb add note "<title>"
        "#
        );
    }

    #[test]
    fn add_question_files_a_question_with_its_origin() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_eq!(
            ok(
                &mut storage,
                &[
                    "add",
                    "question",
                    "Does the parser keep fences?",
                    "--from",
                    "task.demo"
                ],
            ),
            "ok: add question.does-the-parser-keep-fences — questions/question.does-the-parser-keep-fences.md\n"
        );
        let written = storage
            .read("questions/question.does-the-parser-keep-fences.md")
            .unwrap();
        assert!(written.contains("\nstate: open\n"), "{written}");
        assert!(written.contains("\nfrom: task.demo\n"), "{written}");
    }

    #[test]
    fn add_from_a_missing_origin_is_a_recovery_payload() {
        let mut storage = MemoryStorage::new();
        assert_snapshot!(
            refused(&mut storage, &["add", "question", "A doubt", "--from", "task.ghost"]),
            @r#"
        error[dangling-ref]: from: `task.ghost` names no record
        try: anb add task "<title>" --id task.ghost
        try: anb list
        "#
        );
    }

    /// A dangling reference to a record no `add` shape is offered for gets
    /// no creating command; only a missing Task is worth minting on the spot.
    #[test]
    fn a_dangling_reference_to_another_type_offers_no_creating_command() {
        let mut storage = MemoryStorage::new();
        assert_snapshot!(
            refused(&mut storage, &["add", "question", "A doubt", "--from", "note.ghost"]),
            @r"
        error[dangling-ref]: from: `note.ghost` names no record
        try: anb list
        "
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
        assert_eq!(
            ok(
                &mut storage,
                &[
                    "close",
                    "question.doubt",
                    "--resolved-by",
                    "decision.ruling"
                ],
            ),
            "ok: close question.doubt — open\u{2192}closed\nresolved-by: decision.ruling\n"
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
        assert_eq!(
            ok(
                &mut storage,
                &[
                    "close",
                    "question.doubt",
                    "--reason",
                    "overtaken by the rewrite"
                ],
            ),
            "ok: close question.doubt — open\u{2192}closed\n"
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
        assert_snapshot!(
            refused(&mut storage, &["close", "question.doubt"]),
            @r#"
        error[invalid-argument]: close: pass one of --note <path>, --pr <url>, --sha <sha>, --report <path>, --no-proof, --reason "<why>", or --resolved-by <id>
        try: anb close question.doubt --resolved-by <id>
        try: anb close question.doubt --reason "<why>"
        "#
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
        assert_eq!(
            ok(&mut storage, &["retire", "decision.old-rule"]),
            "ok: retire decision.old-rule — active\u{2192}retired\n"
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
        assert_snapshot!(
            ok(&mut worked_example(), &["ready"]),
            @r#"
        ready[3]{id,priority,age,title}:
          task.parser-fences,1,2d,Grammar parser accepts fenced envelopes
          task.check-corpus,2,9d,Negative corpus wired into CI
          task.status-budget,2,4d,"Status degrades sections, keeps Budget"
        "#
        );
    }

    #[test]
    fn a_bounded_list_hints_its_truncation() {
        let mut storage = many_open_tasks(22);

        let bounded = ok(&mut storage, &["ready"]);
        assert!(
            bounded.starts_with("ready[22]{"),
            "the header counts the queue, not the rows it affords: {bounded}"
        );
        assert!(
            bounded.ends_with("  \u{2026} 2 more: anb ready --all\n"),
            "{bounded}"
        );

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
        assert_snapshot!(
            ok(&mut storage, &["list"]),
            @r#"
        records[3]{id,state,priority,title}:
          task.demo,open,-,A demo record
          decision.why-rust,active,-,Rust for the CLI
          question.doubt,open,-,"What, exactly?"
        "#
        );
    }
}

mod single_record {
    use super::*;

    /// A filed record answers a read and refuses every state verb, so the
    /// reply has to say which of the two it is.
    #[test]
    fn viewing_an_archived_record_names_it_as_history() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.done.md".to_owned(),
            record_file("task.done", "task", "closed", "A finished task", &[], ""),
        )]);
        assert!(
            ok(&mut storage, &["show", "task.done"]).contains("archived: true"),
            "a filed record must not read like a live one"
        );
    }

    /// A record's envelope grows with the notebook — an epic hub carries a
    /// `blocked-by` line per child — and a value in it is as long as the
    /// hand that wrote it. `show` bounds both, and `--all` restores the
    /// record whole.
    #[test]
    fn show_bounds_a_long_envelope_and_a_long_value() {
        let edges: Vec<String> = (0..80)
            .map(|n| format!("blocked-by: task.c{n:03}"))
            .collect();
        let lines: Vec<&str> = edges.iter().map(String::as_str).collect();
        let long_title = "word ".repeat(400);
        let mut storage = storage_with(&[(
            "tasks/task.hub.md".to_owned(),
            record_file("task.hub", "task", "open", &long_title, &lines, ""),
        )]);

        let bounded = ok(&mut storage, &["show", "task.hub"]);
        assert!(
            bounded.contains("more: anb show task.hub --all"),
            "the envelope names what it left out: {bounded}"
        );
        assert!(
            !bounded.contains(&long_title),
            "a value as long as a hand wrote it is cut: {bounded}"
        );

        let whole = ok(&mut storage, &["show", "task.hub", "--all"]);
        assert!(whole.contains("blocked-by: task.c079"), "{whole}");
        assert!(whole.contains(long_title.trim_end()), "{whole}");
        assert!(
            whole.len() > bounded.len() * 4,
            "the bound is what makes the default reply small: {} vs {}",
            bounded.len(),
            whole.len()
        );
    }

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

    #[test]
    fn show_prints_the_envelope_the_body_and_the_blocks() {
        assert_snapshot!(
            ok(&mut viewed_storage(), &["show", "task.demo"]),
            @r"
        id: task.demo
        type: task
        state: active
        title: A demo record
        priority: 1
        created: 2026-08-24
        updated: 2026-08-25
        body: |
          The plan follows decision.chosen.

          - 2026-08-25 Maks: started
        mentions[1]: decision.chosen
        mentioned-by[1]: decision.chosen
        "
        );
    }

    /// A Task's log grows for as long as the work does, and `show` is how a
    /// session resumes it, so the reply must not grow with the trail.
    #[test]
    fn show_prints_a_long_body_by_its_ends_and_names_what_it_dropped() {
        let mut storage = logged_task(60);
        let output = ok(&mut storage, &["show", "task.long"]);
        let body: Vec<&str> = output
            .lines()
            .skip_while(|line| *line != "body: |")
            .skip(1)
            .collect();
        assert_eq!(body.first(), Some(&"  - entry 1"));
        assert_eq!(body.last(), Some(&"  - entry 60"));
        assert_eq!(
            body[20],
            "  \u{2026} 20 more lines: anb show task.long --all"
        );
        assert_eq!(body.len(), 41, "twenty lines each end, and the elision");
    }

    /// Both renderings cut the body through the same encoder, and the data
    /// one is what an agent parses without ever reading it.
    #[test]
    fn the_json_body_is_bounded_like_the_text() {
        let mut storage = logged_task(60);
        let view: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["show", "task.long", "--json"])).unwrap();
        assert_eq!(view["body"]["lines"], serde_json::json!(60));
        assert_eq!(view["body"]["head"].as_str().unwrap().lines().count(), 20);
        assert_eq!(view["body"]["tail"].as_str().unwrap().lines().count(), 20);
    }

    #[test]
    fn show_all_prints_every_line_of_a_long_body() {
        let mut storage = logged_task(60);
        let output = ok(&mut storage, &["show", "task.long", "--all"]);
        assert!(output.contains("  - entry 30"), "{output}");
        assert!(!output.contains("more lines"), "{output}");
    }

    /// The elision costs a line of its own, so a body it could not shorten
    /// is left whole.
    #[test]
    fn show_prints_a_body_at_the_bound_whole() {
        let mut storage = logged_task(41);
        let output = ok(&mut storage, &["show", "task.long"]);
        assert!(output.contains("  - entry 21"), "{output}");
        assert!(!output.contains("more lines"), "{output}");
    }

    fn logged_task(entries: usize) -> MemoryStorage {
        let mut body = String::new();
        for entry in 1..=entries {
            let _ = writeln!(body, "- entry {entry}");
        }
        storage_with(&[(
            "tasks/task.long.md".to_owned(),
            record_file("task.long", "task", "active", "A demo record", &[], &body),
        )])
    }

    #[test]
    fn show_json_carries_the_fields_in_envelope_order() {
        let output = ok(&mut viewed_storage(), &["show", "task.demo", "--json"]);
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["archived"], serde_json::json!(false));
        assert_eq!(
            value["fields"]["rows"][0],
            serde_json::json!(["id", "task.demo"])
        );
        assert_eq!(
            value["mentions"],
            serde_json::json!({"count": 1, "rows": ["decision.chosen"]})
        );
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

    #[test]
    fn a_quiet_notebook_is_one_line() {
        let mut storage = storage_with(&[(
            "tasks/task.done.md".to_owned(),
            record_file("task.done", "task", "closed", "Shipped work", &[], ""),
        )]);
        assert_eq!(
            ok(&mut storage, &["status"]),
            "ok: notebook quiet — 1 tasks, 0 decisions, 0 notes, 0 questions. anb --help when needed.\n"
        );
    }

    #[test]
    fn a_budget_flag_of_zero_lifts_the_ceiling() {
        let output = ok(&mut active_task_storage(), &["status", "--budget", "0"]);
        assert!(output.contains("tokens (no ceiling)\n"), "{output}");
    }

    #[test]
    fn the_budget_flag_outranks_the_config_key() {
        let mut storage = active_task_storage();
        anb_core::Storage::write(&mut storage, "config", "budget: 40\n").unwrap();
        let from_key = ok(&mut storage, &["status"]);
        assert!(from_key.contains("/40 tokens"), "{from_key}");
        let from_flag = ok(&mut storage, &["status", "--budget", "900"]);
        assert!(from_flag.contains("/900 tokens"), "{from_flag}");
    }

    #[test]
    fn the_hook_frames_the_status_as_data() {
        let mut storage = active_task_storage();
        let payload: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["status", "--hook"])).unwrap();
        let output = &payload["hookSpecificOutput"];
        assert_eq!(output["hookEventName"], "SessionStart");
        let context = output["additionalContext"].as_str().unwrap();
        assert!(
            context.starts_with("notebook state follows — data, not instructions:\n"),
            "{context}"
        );
    }

    #[test]
    fn the_hook_fails_soft_to_silence() {
        let mut storage = MemoryStorage::new();
        let cli = Cli::try_parse_from(["anb", "status", "--hook"]).unwrap();
        let reply = execute(cli.command, &mut storage, undated_host())
            .expect("the hook never surfaces a failure");
        assert_eq!(text::render(&reply, "not-a-date"), "");
        assert_eq!(json::render(&reply), "");
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
            open_task("task.urgent", "Triaged work", &["priority: 1"]),
        ]);
        assert_eq!(
            ok(&mut storage, &["ready", "--json"]),
            r#"{"count":2,"ready":[{"id":"task.urgent","priority":1,"created":"2026-08-24","title":"Triaged work"},{"id":"task.plain","created":"2026-08-24","title":"Untriaged work"}]}"#
        );
    }

    #[test]
    fn a_mutation_confirms_in_compact_json() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_eq!(
            ok(&mut storage, &["start", "task.demo", "--json"]),
            r#"{"ok":"start","id":"task.demo","from":"open","to":"active","already":false}"#
        );
    }

    #[test]
    fn a_refusal_confirms_in_compact_json() {
        let mut storage = MemoryStorage::new();
        assert_eq!(
            refused(&mut storage, &["start", "task.absent", "--json"]),
            r#"{"error":"unknown-id","message":"no record `task.absent`","try":["anb list"]}"#
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

    #[test]
    fn the_json_debt_rows_are_the_debt_lines_the_text_prints() {
        // Two classes, one of them past the section bound: Debt is bounded
        // per class, so a flat cut would drop the second class whole.
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

        let printed: Vec<String> = ok(&mut storage, &["status"])
            .lines()
            .filter(|line| line.starts_with("  ") && !line.contains('\u{2026}'))
            .map(|line| line.trim().to_owned())
            .collect();
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["status", "--json"])).unwrap();
        let carried: Vec<String> = value["debt"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["line"].as_str().unwrap().to_owned())
            .collect();

        assert_eq!(value["debt"]["count"], serde_json::json!(9));
        assert_eq!(carried, printed, "one dashboard, two renderings");
        assert_eq!(carried.len(), 8, "five of one class, three of the other");
    }

    #[test]
    fn a_decide_reply_carries_the_nudge() {
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
        assert_eq!(
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
                    "--json"
                ],
            ),
            r#"{"ok":"add","id":"decision.fences-stay","path":"decisions/decision.fences-stay.md","may-conflict":{"count":1,"rows":[{"id":"decision.first","by":"supolka"}]}}"#
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
        assert_eq!(
            ok(
                &mut storage,
                &[
                    "close",
                    "question.doubt",
                    "--resolved-by",
                    "decision.ruling",
                    "--json"
                ],
            ),
            r#"{"ok":"close","id":"question.doubt","from":"open","to":"closed","already":false,"resolved-by":"decision.ruling","unblocked":{"count":0,"rows":[]},"open-questions":{"count":0,"rows":[]}}"#
        );
    }

    #[test]
    fn a_comment_reply_carries_only_the_citations_into_nothing() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_eq!(
            ok(
                &mut storage,
                &[
                    "comment",
                    "task.demo",
                    "waits on task.ghost, not the `task.quoted` case",
                    "--json"
                ],
            ),
            r#"{"ok":"comment","id":"task.demo","already":false,"dangling-mention":{"count":1,"rows":["task.ghost"]}}"#
        );
        assert_eq!(
            ok(
                &mut storage,
                &["comment", "task.demo", "plain text", "--json"]
            ),
            r#"{"ok":"comment","id":"task.demo","already":false}"#,
            "an absent nudge is omitted, like every absent field"
        );
    }

    #[test]
    fn a_close_reply_carries_its_consequences() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_eq!(
            ok(
                &mut storage,
                &["close", "task.demo", "--no-proof", "--json"]
            ),
            r#"{"ok":"close","id":"task.demo","from":"active","to":"closed","already":false,"unblocked":{"count":0,"rows":[]},"open-questions":{"count":0,"rows":[]}}"#
        );
    }

    #[test]
    fn an_ingested_report_is_named_in_json_too() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_eq!(
            run_reading(
                &mut storage,
                &["close", "task.demo", "--note", "r.md", "--json"],
                &[("r.md", "# What shipped\n")],
            )
            .expect("the command must succeed"),
            r#"{"ok":"close","id":"task.demo","from":"active","to":"closed","already":false,"report":"note.report-a-demo-record","unblocked":{"count":0,"rows":[]},"open-questions":{"count":0,"rows":[]}}"#
        );
    }
}

/// The surface itself: every verb of the task cycle parses, so a rename in
/// the clap tree cannot slip out silently.
#[test]
fn the_command_vocabulary_parses() {
    for line in [
        vec!["anb", "add", "task", "A title"],
        vec!["anb", "start", "task.x"],
        vec!["anb", "submit", "task.x"],
        vec!["anb", "close", "task.x", "--no-proof"],
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
        vec!["anb", "status", "--budget", "0", "--hook"],
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
        vec!["anb", "graph", "--for", "task.epic", "--ready"],
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
        vec!["anb", "search", "parser", "--all"],
        vec!["anb", "overview"],
    ] {
        assert!(Cli::try_parse_from(&line).is_ok(), "must parse: {line:?}");
    }
    assert!(matches!(
        Cli::try_parse_from(["anb", "status", "--hook"])
            .unwrap()
            .command,
        Command::Status { hook: true, .. }
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
        anb_core::Repair::Unhold,
        anb_core::Repair::Archive,
        anb_core::Repair::Restore,
    ];
    for repair in &repairs {
        match repair {
            anb_core::Repair::Clear(_)
            | anb_core::Repair::Unblock(_)
            | anb_core::Repair::Unhold
            | anb_core::Repair::Archive
            | anb_core::Repair::Restore => {}
        }
    }
    let command = Cli::command();
    let offered = command
        .get_subcommands()
        .filter_map(|verb| anb::recovery::runnable(verb.get_name(), Some("task.demo")))
        .flatten()
        .chain(
            repairs
                .iter()
                .map(|repair| anb::reply::repair_command(repair, "tasks/task.demo.md")),
        );

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

/// A printed command line back into its words: a double-quoted run is one
/// word, however many spaces it holds.
fn shell_words(line: &str) -> impl Iterator<Item = String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    for character in line.chars() {
        match character {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
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

    fn closed_task(id: &str) -> (String, String) {
        (
            format!("tasks/{id}.md"),
            record_file(id, "task", "closed", "A demo record", &[], ""),
        )
    }

    #[test]
    fn a_clean_check_answers_count_zero() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(ok(&mut storage, &["check"]), @"count: 0");
    }

    #[test]
    fn check_names_file_line_severity_code_repair_and_reason() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "cancelled", "A demo record", &[], ""),
        )]);
        assert_snapshot!(
            ok(&mut storage, &["check"]),
            @r#"
        findings[1]{file,line,severity,code,repair,message}:
          tasks/task.demo.md,4,error,bad-value,-,"state: `cancelled` is not one of open, active, review, closed for a task"
        "#
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
        assert_snapshot!(ok(&mut storage, &["check"]), @"count: 0");
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
                git_by: || None,
                read_report: &missing_report,
                lost_proofs: &nothing_lost,
                user_notebook: None,
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
                git_by: || None,
                read_report: &missing_report,
                lost_proofs: &nothing_lost,
                user_notebook: None,
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
        assert_snapshot!(
            ok(&mut storage, &["archive", "task.demo"]),
            @"ok: archive task.demo — archived (already)"
        );
    }

    #[test]
    fn a_restore_is_the_archive_move_made_back() {
        let mut storage = storage_with(&[closed_task("task.demo")]);
        ok(&mut storage, &["archive", "task.demo"]);
        assert_snapshot!(
            ok(&mut storage, &["restore", "task.demo"]),
            @"ok: restore task.demo — archive/tasks/task.demo.md→tasks/task.demo.md"
        );
    }

    #[test]
    fn a_replayed_restore_answers_already() {
        let mut storage = storage_with(&[closed_task("task.demo")]);
        assert_snapshot!(
            ok(&mut storage, &["restore", "task.demo"]),
            @"ok: restore task.demo — live (already)"
        );
    }

    #[test]
    fn archiving_a_live_task_names_the_settling_command() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "active", "A demo record", &[], ""),
        )]);
        assert_snapshot!(
            refused(&mut storage, &["archive", "task.demo"]),
            @r#"
        error[invalid-transition]: `task.demo` is active; valid: close
        try: anb close task.demo --note <path>
        try: anb close task.demo --no-proof
        try: anb close task.demo --reason "<why>"
        "#
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
        assert_eq!(
            ok(&mut storage, &["delete", "note.mistake"]),
            "ok: delete note.mistake — notes/note.mistake.md removed\n"
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
        assert_snapshot!(
            refused(&mut storage, &["delete", "note.mistake"]),
            @r"
        error[still-referenced]: `note.mistake` is still referenced by 2 records
          task.born — from
          task.citing — body
        try: anb show task.born
        try: anb show task.citing
        "
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
        assert_snapshot!(
            refused(&mut storage, &["delete", "note.mistake"]),
            @r"
        error[still-referenced]: `note.mistake` is still referenced by 1 record
          task.holder — from
          task.holder — body
        try: anb show task.holder
        "
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
    fn the_epic_block_names_progress_and_what_to_pick_up() {
        assert_snapshot!(
            ok(&mut an_epic(), &["overview"]),
            @r"
        notebook: 3 tasks, 0 decisions, 0 notes, 0 questions
        epics[1]:
          task.epic-auth: 1/2 closed, next: task.auth-tokens
        tasks[3]{id,state,priority,title}:
          task.auth-login,closed,-,The login screen
          task.auth-tokens,open,-,Token rotation
          task.epic-auth,open,-,Auth end to end
        "
        );
    }

    /// One epic per row, and a notebook can carry more epics than any reply
    /// shows.
    fn many_epics(count: usize) -> MemoryStorage {
        let mut files = Vec::new();
        for nth in 0..count {
            let hub = format!("task.epic-{nth:02}");
            let child = format!("task.child-{nth:02}");
            files.push((
                format!("tasks/{hub}.md"),
                record_file(
                    &hub,
                    "task",
                    "open",
                    "An epic",
                    &[&format!("blocked-by: {child}")],
                    "",
                ),
            ));
            files.push((
                format!("tasks/{child}.md"),
                record_file(
                    &child,
                    "task",
                    "open",
                    "A child",
                    &[&format!("from: {hub}")],
                    "",
                ),
            ));
        }
        storage_with(&files)
    }

    #[test]
    fn the_epic_block_is_bounded_like_every_other_listing() {
        let shown = ok(&mut many_epics(22), &["overview"]);
        assert!(
            shown.contains("epics[22]:")
                && shown.matches("closed, next:").count() == 20
                && shown.contains("\u{2026} 2 more: anb overview --all"),
            "the count is the notebook\'s, the rows are the reply\'s: {shown}"
        );
        assert_eq!(
            ok(&mut many_epics(22), &["overview", "--all"])
                .matches("closed, next:")
                .count(),
            22,
            "and --all lifts the bound here as everywhere"
        );
    }

    #[test]
    fn the_epic_block_is_a_shape_the_json_carries_too() {
        assert_eq!(
            ok(&mut an_epic(), &["overview", "--json"]),
            r#"{"live":{"tasks":3,"decisions":0,"notes":0,"questions":0},"epics":{"count":1,"rows":[{"id":"task.epic-auth","closed":1,"total":2,"next":"task.auth-tokens"}]},"tasks":{"count":3,"rows":[{"id":"task.auth-login","state":"closed","title":"The login screen"},{"id":"task.auth-tokens","state":"open","title":"Token rotation"},{"id":"task.epic-auth","state":"open","title":"Auth end to end"}]},"decisions":{"count":0,"rows":[]},"notes":{"count":0,"rows":[]},"questions":{"count":0,"rows":[]},"archive":{"tasks":0,"decisions":0,"notes":0,"questions":0}}"#
        );
    }

    #[test]
    fn a_scoped_queue_answers_only_the_epic_it_was_asked_about() {
        let mut storage = an_epic();
        ok(&mut storage, &["add", "task", "Something else entirely"]);
        assert_snapshot!(
            ok(&mut storage, &["ready", "--for", "task.epic-auth"]),
            @r"
        ready[1]{id,priority,age,title}:
          task.auth-tokens,-,4d,Token rotation
        "
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
            (
                "list",
                "  \u{2026} 6 more: anb list --for task.hub --all",
                25 + 1,
            ),
            (
                "ready",
                "  \u{2026} 5 more: anb ready --for task.hub --all",
                25,
            ),
        ] {
            let out = ok(&mut storage, &[verb, "--for", "task.hub"]);
            assert_eq!(out.lines().last().unwrap(), hint, "{out}");

            // The hint is a command, so running it must answer what it
            // promises: every row of the same scope, and nothing left to hint.
            let lifted = ok(&mut storage, &[verb, "--for", "task.hub", "--all"]);
            assert_eq!(lifted.lines().count(), rows_in_scope + 1, "{lifted}");
            assert!(!lifted.contains("more:"), "{lifted}");
        }
    }

    #[test]
    fn a_scope_named_by_no_record_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            refused(&mut storage, &["ready", "--for", "task.no-such-epic"]),
            @r"
        error[unknown-id]: no record `task.no-such-epic`
        try: anb list
        "
        );
    }

    #[test]
    fn a_repeated_clear_flag_names_every_field_it_erased() {
        let mut storage = storage_with(&[open_task(
            "task.demo",
            "A demo record",
            &["from: task.parent", "priority: 2"],
        )]);
        assert_snapshot!(
            ok(
                &mut storage,
                &["edit", "task.demo", "--clear", "from", "--clear", "priority"],
            ),
            @"ok: edit task.demo — from, priority"
        );
    }

    #[test]
    fn a_field_with_no_eraser_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            refused(&mut storage, &["edit", "task.demo", "--clear", "state"]),
            @r#"
        error[invalid-argument]: clear: `state` is not an erasable field — from, priority, review-by
        try: anb edit task.demo --title "<title>"
        "#
        );
    }

    #[test]
    fn an_edit_changing_nothing_answers_already() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            ok(&mut storage, &["edit", "task.demo", "--title", "A demo record"]),
            @"ok: edit task.demo — unchanged (already)"
        );
    }

    #[test]
    fn an_edited_body_carries_the_dangling_mention_nudge() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            ok(
                &mut storage,
                &["edit", "task.demo", "--body", "Blocked by task.ghost."],
            ),
            @r"
        ok: edit task.demo — body
        dangling-mention[1]: task.ghost — backtick to quote, or create the record
        "
        );
    }

    #[test]
    fn an_edit_requesting_nothing_is_a_recovery_payload() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(
            refused(&mut storage, &["edit", "task.demo"]),
            @r#"
        error[invalid-argument]: edit: nothing to change — pass --title, --body, --tag, --untag, --from, --priority, --review-by, or --clear
        try: anb edit task.demo --title "<title>"
        "#
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
        assert_snapshot!(
            text::render_error(&error, &subject),
            @r"
        error[not-utf8]: not UTF-8: tasks/task.demo.md
        try: anb check
        "
        );
    }
}

mod search_replies {
    use super::*;

    #[test]
    fn matches_share_the_listing_shape() {
        let mut storage = storage_with(&[
            open_task("task.parser", "Grammar parser work", &[]),
            (
                "archive/tasks/task.spike.md".to_owned(),
                record_file("task.spike", "task", "closed", "Parser spike", &[], ""),
            ),
        ]);
        assert_snapshot!(
            ok(&mut storage, &["search", "parser"]),
            @r"
        matches[2]{id,state,priority,title}:
          task.parser,open,-,Grammar parser work
          task.spike,closed,-,Parser spike
        "
        );
    }

    /// The hint is meant to be typed back, so the query it carries must
    /// survive the shell — including the quote that would end the word.
    #[test]
    fn the_truncation_hint_carries_the_query_as_one_shell_word() {
        let mut storage = many_open_tasks(22);
        assert_snapshot!(
            ok(&mut storage, &["search", "demo record"]).lines().last().unwrap(),
            @r"  … 2 more: anb search 'demo record' --all"
        );
        let files: Vec<(String, String)> = (0..22)
            .map(|n| open_task(&format!("task.q{n:02}"), "Won't fix, won't file", &[]))
            .collect();
        assert_snapshot!(
            ok(&mut storage_with(&files), &["search", "won't"]).lines().last().unwrap(),
            @r"  … 2 more: anb search 'won'\''t' --all"
        );
    }

    #[test]
    fn a_match_of_nothing_answers_count_zero() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_snapshot!(ok(&mut storage, &["search", "zeppelin"]), @"count: 0");
    }
}

mod overview_reply {
    use super::*;

    #[test]
    fn the_page_groups_types_and_counts_the_archive() {
        let mut storage = storage_with(&[
            open_task("task.a", "A demo record", &[]),
            (
                "decisions/decision.d.md".to_owned(),
                record_file("decision.d", "decision", "active", "A ruling", &[], ""),
            ),
            (
                "archive/tasks/task.done.md".to_owned(),
                record_file("task.done", "task", "closed", "Shipped", &[], ""),
            ),
        ]);
        assert_snapshot!(
            ok(&mut storage, &["overview"]),
            @r"
        notebook: 1 tasks, 1 decisions, 0 notes, 0 questions
        tasks[1]{id,state,priority,title}:
          task.a,open,-,A demo record
        decisions[1]{id,state,priority,title}:
          decision.d,active,-,A ruling
        archive: 1 tasks, 0 decisions, 0 notes, 0 questions
        "
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
            open_task("task.first", "The blocker", &[]),
            open_task("task.second", "The waiter", &["blocked-by: task.first"]),
            open_task("task.stray", "Another line of work", &[]),
        ])
    }

    /// Every other verb answers an agent, and a picture answers nobody
    /// without a browser. So the graph itself is what the verb prints.
    #[test]
    fn the_verb_prints_the_graph_itself() {
        assert_snapshot!(ok(&mut a_chain(), &["graph"]), @r"
        nodes[3]{id,type,state,ready,archived,degree,priority,created,epic,title}:
          task.first,task,open,yes,no,1,-,2026-08-24,-,The blocker
          task.second,task,open,no,no,1,-,2026-08-24,-,The waiter
          task.stray,task,open,yes,no,0,-,2026-08-24,-,Another line of work
        edges[1]{from,to,kind}:
          task.first,task.second,waits
        ");
    }

    /// A caller builds against a shape, so the document says which shape it
    /// is and which slice it answers.
    #[test]
    fn the_data_names_its_reading_and_the_slice_it_answers() {
        let payload = ok(&mut a_chain(), &["--json", "graph", "--ready"]);
        let parsed: serde_json::Value =
            serde_json::from_str(&payload).unwrap_or_else(|_| panic!("not JSON: {payload}"));

        assert_eq!(parsed["v"], 2);
        assert_eq!(parsed["slice"]["ready"], true);
        assert_eq!(parsed["slice"]["archive"], false);
        assert_eq!(parsed["nodes"]["count"], 2);
        assert_eq!(parsed["nodes"]["rows"][0]["id"], "task.first");
        assert_eq!(parsed["nodes"]["rows"][0]["degree"], 0);
        assert_eq!(
            parsed["edges"]["count"], 0,
            "the ready lens holds only what waits on nothing live"
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
        assert_snapshot!(printed, @r#"
        nodes[1]{id,type,state,ready,archived,degree,priority,created,epic,title}:
          task.stray,task,open,yes,no,0,-,2026-08-24,-,Another line of work
        edges[0]{from,to,kind}:
        fields[6]{id,key,value}:
          task.stray,id,task.stray
          task.stray,type,task
          task.stray,state,open
          task.stray,title,Another line of work
          task.stray,created,2026-08-24
          task.stray,updated,2026-08-25
        bodies[0]{id,text}:
        "#);
    }

    /// A slice narrows the graph itself, so the same flags answer the same
    /// tiles whether the caller reads them or looks at them.
    #[test]
    fn every_slice_flag_narrows_what_the_verb_answers() {
        let mut storage = a_chain();
        assert_snapshot!(ok(&mut storage, &["graph", "--ready"]), @r"
        nodes[2]{id,type,state,ready,archived,degree,priority,created,epic,title}:
          task.first,task,open,yes,no,0,-,2026-08-24,-,The blocker
          task.stray,task,open,yes,no,0,-,2026-08-24,-,Another line of work
        edges[0]{from,to,kind}:
        ");
        assert_snapshot!(ok(&mut storage, &["graph", "--for", "task.second"]), @r"
        nodes[2]{id,type,state,ready,archived,degree,priority,created,epic,title}:
          task.first,task,open,yes,no,1,-,2026-08-24,-,The blocker
          task.second,task,open,no,no,1,-,2026-08-24,-,The waiter
        edges[1]{from,to,kind}:
          task.first,task.second,waits
        ");
        assert_snapshot!(ok(&mut storage, &["graph", "--focus", "task.first"]), @r"
        nodes[2]{id,type,state,ready,archived,degree,priority,created,epic,title}:
          task.first,task,open,yes,no,1,-,2026-08-24,-,The blocker
          task.second,task,open,no,no,1,-,2026-08-24,-,The waiter
        edges[1]{from,to,kind}:
          task.first,task.second,waits
        ");
    }

    /// The plain text is bounded because a reader asked a question; the
    /// document is not, because what reads it draws from it, and a drawing
    /// made from some of the edges is not a smaller picture of this
    /// notebook but a picture of one that does not exist.
    #[test]
    fn the_document_carries_every_row_the_text_bounds() {
        let mut storage = many_open_tasks(anb_core::encode::ROW_BOUND + 2);

        let printed = ok(&mut storage, &["graph"]);
        let rows = printed
            .lines()
            .filter(|line| line.starts_with("  task."))
            .count();
        assert_eq!(rows, anb_core::encode::ROW_BOUND, "the text is bounded");

        let payload = ok(&mut storage, &["--json", "graph"]);
        let parsed: serde_json::Value =
            serde_json::from_str(&payload).unwrap_or_else(|_| panic!("not JSON: {payload}"));
        let counted = anb_core::encode::ROW_BOUND + 2;
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
        let between = printed
            .lines()
            .filter(|line| line.starts_with("  task.blocker,task.waiter,"))
            .collect::<Vec<&str>>();
        assert_eq!(
            between,
            vec!["  task.blocker,task.waiter,waits"],
            "the declared word survives and the mention does not repeat it: {printed}"
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
        let recovery = anb::recovery::unknown_command_recovery(&error)
            .expect("an unknown verb joins the catalog");
        assert_snapshot!(
            text::render_recovery(&recovery),
            @r"
        error[unknown-command]: `archiv` is not an anb command
        try: anb search --help
        try: anb archive --help
        try: anb --help
        "
        );
    }

    #[test]
    fn a_verb_near_nothing_still_points_at_help() {
        let Err(error) = Cli::try_parse_from(["anb", "zzz"]) else {
            panic!("an unknown verb must not parse");
        };
        let recovery = anb::recovery::unknown_command_recovery(&error).unwrap();
        assert_snapshot!(
            text::render_recovery(&recovery),
            @r"
        error[unknown-command]: `zzz` is not an anb command
        try: anb --help
        "
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
        assert!(anb::recovery::unknown_command_recovery(&help).is_none());
    }

    #[test]
    fn a_missing_argument_stays_claps() {
        let Err(missing_arg) = Cli::try_parse_from(["anb", "start"]) else {
            panic!("a missing argument must not parse");
        };
        assert!(anb::recovery::unknown_command_recovery(&missing_arg).is_none());
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
        assert!(out.starts_with("findings[22]{"), "got: {out}");
        assert_eq!(out.lines().last().unwrap(), "  … 2 more: anb check --all");
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
        let out = ok(&mut storage, &["close", "task.hub", "--no-proof"]);
        let line = out
            .lines()
            .find(|line| line.starts_with("unblocked["))
            .unwrap_or_else(|| panic!("no unblocked line in: {out}"));
        assert!(line.starts_with("unblocked[21]: task.d00, "), "{line}");
        assert!(line.ends_with("task.d19, … 1 more"), "{line}");
    }

    #[test]
    fn a_close_json_carries_the_count_beside_the_bounded_rows() {
        let mut storage = hub_with_dependents(MANY);
        let out = ok(&mut storage, &["close", "task.hub", "--no-proof", "--json"]);
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
        let line = cited_line(&ok(&mut storage, &["show", "note.magnet"]));
        assert!(line.starts_with("mentioned-by[21]: task.c00, "), "{line}");
        assert!(
            line.ends_with("task.c19, … 1 more: anb show note.magnet --all"),
            "{line}"
        );
    }

    #[test]
    fn a_view_all_names_every_record_that_cites_this_one() {
        let mut storage = citers_of("note.magnet", MANY);
        let line = cited_line(&ok(&mut storage, &["show", "note.magnet", "--all"]));
        assert!(line.starts_with("mentioned-by[21]: task.c00, "), "{line}");
        assert!(line.ends_with("task.c20"), "{line}");
    }

    fn cited_line(out: &str) -> String {
        out.lines()
            .find(|line| line.starts_with("mentioned-by["))
            .unwrap_or_else(|| panic!("no mentioned-by line in: {out}"))
            .to_owned()
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
        assert!(
            out.starts_with(
                "error[still-referenced]: `task.hub` is still referenced by 11 records\n"
            ),
            "{out}"
        );
        let details: Vec<&str> = out.lines().filter(|line| line.starts_with("  ")).collect();
        assert_eq!(details.len(), 21, "twenty blockers and the cut: {out}");
        assert_eq!(details[20], "  … 2 more");
        assert_eq!(
            out.lines().filter(|line| line.starts_with("try: ")).count(),
            11,
            "one retry per record holding the id: {out}"
        );
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
        let finding = out
            .lines()
            .find(|line| line.contains("block-cycle"))
            .unwrap_or_else(|| panic!("no block-cycle finding in: {out}"));
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
            assert_eq!(ok(&mut storage, &line), expected);
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
    fn archive_carries_its_reports_in_the_json_too() {
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
        assert_eq!(
            ok(&mut storage, &["archive", "task.demo", "--json"]),
            r#"{"ok":"archive","id":"task.demo","from":"tasks/task.demo.md","to":"archive/tasks/task.demo.md","carried":{"count":1,"rows":["note.report"]},"already":false}"#
        );
    }

    #[test]
    /// `carried` is the one key an archive reply omits when it is empty:
    /// nothing was taken along, and a reader is told nothing rather than
    /// an empty list.
    fn archive_confirms_the_move() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "closed", "A demo record", &[], ""),
        )]);
        assert_eq!(
            ok(&mut storage, &["archive", "task.demo", "--json"]),
            r#"{"ok":"archive","id":"task.demo","from":"tasks/task.demo.md","to":"archive/tasks/task.demo.md","already":false}"#
        );
    }

    #[test]
    fn archive_names_the_report_it_carried() {
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
        assert_eq!(
            ok(&mut storage, &["archive", "task.demo"]),
            "ok: archive task.demo — tasks/task.demo.md\u{2192}archive/tasks/task.demo.md\ncarried[1]: note.report\n"
        );
    }

    #[test]
    fn restore_confirms_the_move_back() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.demo.md".to_owned(),
            record_file("task.demo", "task", "closed", "A demo record", &[], ""),
        )]);
        assert_eq!(
            ok(&mut storage, &["restore", "task.demo", "--json"]),
            r#"{"ok":"restore","id":"task.demo","from":"archive/tasks/task.demo.md","to":"tasks/task.demo.md","already":false}"#
        );
    }

    #[test]
    fn edit_names_what_changed_and_the_nudge() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_eq!(
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
            r#"{"ok":"edit","id":"task.demo","changed":["title","body"],"already":false,"dangling-mention":{"count":1,"rows":["task.ghost"]}}"#
        );
    }

    #[test]
    fn search_answers_matches() {
        let mut storage = storage_with(&[open_task("task.demo", "A demo record", &[])]);
        assert_eq!(
            ok(&mut storage, &["search", "demo", "--json"]),
            r#"{"count":1,"matches":[{"id":"task.demo","state":"open","title":"A demo record"}]}"#
        );
    }

    #[test]
    fn overview_carries_the_tallies_and_the_sections() {
        let mut storage = storage_with(&[
            open_task("task.a", "A demo record", &[]),
            (
                "archive/tasks/task.done.md".to_owned(),
                record_file("task.done", "task", "closed", "Shipped", &[], ""),
            ),
        ]);
        let value: serde_json::Value =
            serde_json::from_str(&ok(&mut storage, &["overview", "--json"])).unwrap();
        assert_eq!(value["live"]["tasks"], serde_json::json!(1));
        assert_eq!(value["tasks"]["rows"][0]["id"], serde_json::json!("task.a"));
        assert_eq!(value["decisions"]["count"], serde_json::json!(0));
        assert_eq!(value["archive"]["tasks"], serde_json::json!(1));
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
        &["search", "rule"],
    ];

    /// The verbs the rule admits beyond those, each for a reason the
    /// Global Notebook would be crippled without: knowledge corrected in
    /// place and verified, settled knowledge filed and brought back,
    /// mistakes erased, and the whole-notebook reads. Their rows are empty
    /// of tasks and questions rather than absent, so the output contract
    /// is one contract.
    const ALSO_ADMITTED: &[&[&str]] = &[
        &["edit", "note.demo", "--title", "A sharper fact"],
        &["check"],
        &["archive", "note.demo"],
        &["restore", "note.demo"],
        &["delete", "note.demo"],
        &["overview"],
        &["status"],
        &["ready"],
        &["graph"],
    ];

    /// Every verb that creates or moves a task or a question.
    const WORK: &[&[&str]] = &[
        &["add", "task", "A task"],
        &["start", "task.demo"],
        &["submit", "task.demo"],
        &["close", "task.demo", "--no-proof"],
        &["reopen", "task.demo"],
        &["hold", "task.demo", "--reason", "waiting"],
        &["unhold", "task.demo"],
        &["block", "task.demo", "task.other"],
        &["unblock", "task.demo", "task.other"],
        &["comment", "task.demo", "a line"],
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
                anb::scope::refused_globally(&parsed(line), true).is_none(),
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
        for line in WORK {
            let refused = anb::scope::refused_globally(&parsed(line), true)
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
        for line in WORK.iter().chain(NAMED).chain(ALSO_ADMITTED) {
            assert!(
                anb::scope::refused_globally(&parsed(line), false).is_none(),
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
        assert_eq!(
            reply, "ok: add decision.spaces-here — decisions/decision.spaces-here.md\n",
            "the same fact Status names as a shadow is no dangling mention at the write"
        );
    }

    #[test]
    fn a_link_to_a_global_rule_passes_the_gate() {
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
        assert_eq!(
            run_behind(&mut storage, Some(&user), &["check"], &missing_report)
                .expect("the command must succeed"),
            "count: 0\n"
        );
    }
}
