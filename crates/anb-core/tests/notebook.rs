//! The record model's write-time invariants, specified at the Storage seam:
//! strings in, exact strings and returned models out. Expected file bytes
//! derive from the format's canonical form, never from running the code.

use anb_core::{
    Blocker, Budget, CitedProof, DebtSignal, Draft, Edit, FindingCode, Link, MemoryStorage,
    Notebook, NotebookError, Proof, RecordType, Storage, StorageError, Transitioned,
};

const TODAY: &str = "2026-08-27";

/// A Status asked with nothing to settle against the world outside the
/// notebook: the host finds every cited proof still there.
fn no_lost_proofs(_cited: &[CitedProof]) -> Vec<CitedProof> {
    Vec::new()
}

fn task_file(state: &str, extra_lines: &[&str]) -> String {
    record_file("task.demo", "task", state, extra_lines, "")
}

fn record_file(id: &str, type_word: &str, state: &str, extra_lines: &[&str], body: &str) -> String {
    let mut text =
        format!("---\nid: {id}\ntype: {type_word}\nstate: {state}\ntitle: A demo record\n");
    for line in extra_lines {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str("created: 2026-08-24\nupdated: 2026-08-25\n---\n");
    text.push_str(body);
    text
}

fn storage_with(files: &[(&str, &str)]) -> MemoryStorage {
    MemoryStorage::from_files(files.iter().map(|(path, text)| (*path, *text)))
}

fn moved(id: &str, from: &'static str, to: &'static str) -> Transitioned {
    Transitioned {
        id: id.to_owned(),
        from,
        to,
        already: false,
    }
}

mod task_cycle {
    use super::*;

    #[test]
    fn start_moves_an_open_task_to_active_touching_only_its_own_lines() {
        let quirky = "---\nid: task.demo\ntype:  task\nstate: open\ncustom: kept   \ntitle: A demo record\ncreated: 2026-08-24\n---\nbody\n";
        let mut storage = storage_with(&[("tasks/task.demo.md", quirky)]);
        let reply = Notebook::new(&mut storage)
            .start("task.demo", TODAY)
            .unwrap();
        assert_eq!(reply, moved("task.demo", "open", "active"));
        assert_eq!(
            storage.read("tasks/task.demo.md").unwrap(),
            "---\nid: task.demo\ntype:  task\nstate: active\ncustom: kept   \ntitle: A demo record\ncreated: 2026-08-24\nupdated: 2026-08-27\n---\nbody\n",
            "only the state line and the new updated line may change"
        );
    }

    #[test]
    fn a_replayed_start_answers_already_true_and_changes_no_byte() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        Notebook::new(&mut storage)
            .start("task.demo", TODAY)
            .unwrap();
        let after_first = storage.read("tasks/task.demo.md").unwrap();

        let replay = Notebook::new(&mut storage)
            .start("task.demo", TODAY)
            .unwrap();
        assert!(replay.already);
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), after_first);
    }

    #[test]
    fn an_invalid_transition_answers_with_the_valid_commands() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let error = Notebook::new(&mut storage)
            .close("task.demo", &Proof::Waived, TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::InvalidTransition {
                id: "task.demo".to_owned(),
                state: "open".to_owned(),
                valid: vec!["start"],
            }
        );
    }

    #[test]
    fn close_stamps_the_close_date_and_the_proof_link() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let closed = Notebook::new(&mut storage)
            .close(
                "task.demo",
                &Proof::Pr("https://example.com/pull/7".to_owned()),
                TODAY,
            )
            .unwrap();
        assert_eq!(closed.transition, moved("task.demo", "active", "closed"));
        let text = storage.read("tasks/task.demo.md").unwrap();
        assert!(text.contains("\nstate: closed\n"));
        assert!(text.contains("\nclosed: 2026-08-27\n"));
        assert!(text.contains("\nlink: pr https://example.com/pull/7\n"));
        assert!(text.contains("\nupdated: 2026-08-27\n"));
    }

    #[test]
    fn a_proof_with_an_empty_target_is_refused_before_any_byte_moves() {
        let text = task_file("active", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        let error = Notebook::new(&mut storage)
            .close("task.demo", &Proof::Pr(String::new()), TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidArgument { .. }));
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
    }

    #[test]
    fn a_proof_carrying_a_newline_cannot_inject_envelope_fields() {
        let text = task_file("active", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        let error = Notebook::new(&mut storage)
            .close(
                "task.demo",
                &Proof::Report("report.md\nhold: injected".to_owned()),
                TODAY,
            )
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidArgument { .. }));
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
    }

    #[test]
    fn a_return_on_a_task_never_submitted_is_invalid_not_a_replay() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let error = Notebook::new(&mut storage)
            .return_task("task.demo", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::InvalidTransition {
                id: "task.demo".to_owned(),
                state: "active".to_owned(),
                valid: vec!["submit", "close"],
            }
        );
    }

    #[test]
    fn a_waived_close_writes_no_proof_link() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        Notebook::new(&mut storage)
            .close("task.demo", &Proof::Waived, TODAY)
            .unwrap();
        assert!(
            !storage
                .read("tasks/task.demo.md")
                .unwrap()
                .contains("link:"),
            "an explicitly waived close carries no proof to lie about"
        );
    }

    #[test]
    fn a_replayed_close_answers_already_and_appends_no_second_proof() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let proof = Proof::Sha("f00dfeed".to_owned());
        Notebook::new(&mut storage)
            .close("task.demo", &proof, TODAY)
            .unwrap();
        let after_first = storage.read("tasks/task.demo.md").unwrap();

        let replay = Notebook::new(&mut storage)
            .close("task.demo", &Proof::Sha("0ther5ha".to_owned()), TODAY)
            .unwrap();
        assert!(replay.transition.already);
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), after_first);
    }

    const REPORT: &str = "# What shipped\n\nThe parser now accepts fenced envelopes.\n";

    #[test]
    fn a_close_with_a_report_lands_it_as_a_note_the_task_links() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let closed = Notebook::new(&mut storage)
            .close_with_report("task.demo", REPORT, None, TODAY)
            .unwrap();

        let note = closed.report_note.expect("the close ingested a report");
        let written = storage.read(&format!("notes/{note}.md")).unwrap();
        assert!(
            written.contains("\nfrom: task.demo\n"),
            "the Note is born from the Task it reports on: {written}"
        );
        assert!(
            written.ends_with(REPORT),
            "the report's text is the Note's body, verbatim: {written}"
        );
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .contains(&format!("\nlink: note {note}\n")),
            "a reader reaches the report through the notebook alone"
        );
    }

    #[test]
    fn a_replayed_report_close_answers_already_and_mints_no_second_note() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let mut notebook = Notebook::new(&mut storage);
        notebook
            .close_with_report("task.demo", REPORT, None, TODAY)
            .unwrap();
        let replay = notebook
            .close_with_report("task.demo", REPORT, None, TODAY)
            .unwrap();

        assert!(replay.transition.already);
        assert_eq!(
            storage.list("notes").unwrap().len(),
            1,
            "a replay creates nothing"
        );
    }

    #[test]
    fn a_report_close_resumed_after_the_note_landed_reuses_it() {
        // The crash window of the two writes: the Note is on disk, the Task
        // never reached `closed`.
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let orphan = Notebook::new(&mut storage)
            .create(
                &{
                    let mut draft = Draft::new(RecordType::Note, "Report: A demo record");
                    draft.from = Some("task.demo".to_owned());
                    draft.body = REPORT.to_owned();
                    draft
                },
                TODAY,
            )
            .unwrap()
            .id;

        let closed = Notebook::new(&mut storage)
            .close_with_report("task.demo", REPORT, None, TODAY)
            .unwrap();
        assert_eq!(closed.report_note.as_deref(), Some(orphan.as_str()));
        assert_eq!(
            storage.list("notes").unwrap().len(),
            1,
            "the interrupted call is resumed, not doubled"
        );
    }

    #[test]
    fn a_second_report_after_a_reopen_lands_whole_beside_the_first() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let mut notebook = Notebook::new(&mut storage);
        let first = notebook
            .close_with_report("task.demo", REPORT, None, TODAY)
            .unwrap()
            .report_note
            .unwrap();
        notebook.reopen("task.demo", TODAY).unwrap();
        notebook.start("task.demo", TODAY).unwrap();
        let second = notebook
            .close_with_report("task.demo", "# What shipped next\n", None, TODAY)
            .unwrap()
            .report_note
            .unwrap();

        assert_ne!(
            first, second,
            "a new report is a new Note, never the old one"
        );
        assert!(
            storage
                .read(&format!("notes/{second}.md"))
                .unwrap()
                .ends_with("# What shipped next\n"),
            "the report the caller passed is the one the notebook holds"
        );
    }

    #[test]
    fn an_archived_report_is_never_revived_as_a_fresh_proof() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let mut notebook = Notebook::new(&mut storage);
        let filed = notebook
            .close_with_report("task.demo", REPORT, None, TODAY)
            .unwrap()
            .report_note
            .unwrap();
        notebook.retire(&filed, TODAY).unwrap();
        notebook.archive(&filed).unwrap();
        notebook.reopen("task.demo", TODAY).unwrap();
        notebook.start("task.demo", TODAY).unwrap();

        let again = notebook
            .close_with_report("task.demo", REPORT, None, TODAY)
            .unwrap()
            .report_note
            .unwrap();
        assert_ne!(
            again, filed,
            "history stays history; a live close needs a live proof"
        );
        assert!(storage.read(&format!("notes/{again}.md")).is_ok());
    }

    #[test]
    fn an_empty_report_is_refused_before_anything_is_written() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let before = storage.read("tasks/task.demo.md").unwrap();
        Notebook::new(&mut storage)
            .close_with_report("task.demo", "  \n", None, TODAY)
            .unwrap_err();
        assert_eq!(storage.list("notes").unwrap(), Vec::<String>::new());
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), before);
    }

    #[test]
    fn a_report_citing_an_unwritten_id_carries_the_quotation_nudge() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let closed = Notebook::new(&mut storage)
            .close_with_report(
                "task.demo",
                "follows from task.never-written\n",
                None,
                TODAY,
            )
            .unwrap();
        assert_eq!(closed.dangling_mentions, vec!["task.never-written"]);
    }

    #[test]
    fn a_note_standing_at_the_derived_id_from_elsewhere_is_not_claimed() {
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("active", &[])),
            (
                "tasks/task.other.md",
                &record_file("task.other", "task", "closed", &[], ""),
            ),
            (
                "notes/note.report-a-demo-record.md",
                &record_file(
                    "note.report-a-demo-record",
                    "note",
                    "active",
                    &["from: task.other"],
                    "someone else's report\n",
                ),
            ),
        ]);
        let closed = Notebook::new(&mut storage)
            .close_with_report("task.demo", REPORT, None, TODAY)
            .unwrap();

        assert_ne!(
            closed.report_note.as_deref(),
            Some("note.report-a-demo-record"),
            "a Note born from another Task is a different record, not this call's"
        );
        assert_eq!(
            storage.read("notes/note.report-a-demo-record.md").unwrap(),
            record_file(
                "note.report-a-demo-record",
                "note",
                "active",
                &["from: task.other"],
                "someone else's report\n"
            ),
            "and its bytes are untouched"
        );
    }

    #[test]
    fn a_report_close_on_a_task_that_cannot_close_writes_no_note() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        Notebook::new(&mut storage)
            .close_with_report("task.demo", REPORT, None, TODAY)
            .unwrap_err();
        assert_eq!(
            storage.list("notes").unwrap(),
            Vec::<String>::new(),
            "a refused close leaves no orphan behind"
        );
    }

    #[test]
    fn close_names_the_still_open_questions_born_from_the_task() {
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("active", &[])),
            (
                "questions/question.open-doubt.md",
                &record_file(
                    "question.open-doubt",
                    "question",
                    "open",
                    &["from: task.demo"],
                    "",
                ),
            ),
            (
                "questions/question.already-routed.md",
                &record_file(
                    "question.already-routed",
                    "question",
                    "routed",
                    &["from: task.demo", "routed-to: task.demo"],
                    "",
                ),
            ),
            (
                "questions/question.elsewhere.md",
                &record_file("question.elsewhere", "question", "open", &[], ""),
            ),
        ]);
        let closed = Notebook::new(&mut storage)
            .close("task.demo", &Proof::Waived, TODAY)
            .unwrap();
        assert_eq!(closed.open_questions, vec!["question.open-doubt"]);
    }

    #[test]
    fn close_does_not_name_an_invalid_question() {
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("active", &[])),
            (
                "questions/question.corrupt.md",
                &record_file(
                    "question.corrupt",
                    "question",
                    "open",
                    &["from: task.demo", "routed-to: decision.never-written"],
                    "",
                ),
            ),
        ]);
        let closed = Notebook::new(&mut storage)
            .close("task.demo", &Proof::Waived, TODAY)
            .unwrap();
        assert_eq!(
            closed.open_questions,
            Vec::<String>::new(),
            "an invalid record is check's to name, as from every derived query"
        );
    }

    #[test]
    fn the_review_loop_submits_returns_and_closes_from_review() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        {
            let mut notebook = Notebook::new(&mut storage);
            assert_eq!(
                notebook.submit("task.demo", TODAY).unwrap(),
                moved("task.demo", "active", "review")
            );
            assert_eq!(
                notebook.return_task("task.demo", TODAY).unwrap(),
                moved("task.demo", "review", "active")
            );
            notebook.submit("task.demo", TODAY).unwrap();
            assert_eq!(
                notebook
                    .close("task.demo", &Proof::Waived, TODAY)
                    .unwrap()
                    .transition,
                moved("task.demo", "review", "closed")
            );
        }
    }

    #[test]
    fn reopen_reopens_a_closed_task_and_drops_the_close_date() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &task_file("closed", &["closed: 2026-08-25"]),
        )]);
        let reply = Notebook::new(&mut storage)
            .reopen("task.demo", TODAY)
            .unwrap();
        assert_eq!(reply, moved("task.demo", "closed", "open"));
        let text = storage.read("tasks/task.demo.md").unwrap();
        assert!(!text.contains("closed:"), "an open task has no close date");
    }

    #[test]
    fn a_task_command_on_another_type_names_the_expected_type() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file("note.demo", "note", "active", &[], ""),
        )]);
        let error = Notebook::new(&mut storage)
            .start("note.demo", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::WrongType {
                id: "note.demo".to_owned(),
                expected: "a task".to_owned(),
            }
        );
    }

    #[test]
    fn an_unknown_id_and_an_archived_id_are_told_apart() {
        let mut storage =
            storage_with(&[("archive/tasks/task.shipped.md", &task_file("closed", &[]))]);
        let mut notebook = Notebook::new(&mut storage);
        assert_eq!(
            notebook.start("task.absent", TODAY).unwrap_err(),
            NotebookError::UnknownId {
                id: "task.absent".to_owned()
            }
        );
        assert_eq!(
            notebook.start("task.shipped", TODAY).unwrap_err(),
            NotebookError::Archived {
                id: "task.shipped".to_owned()
            }
        );
    }

    #[test]
    fn a_record_carrying_an_error_finding_is_never_mutated() {
        let text = task_file("someday", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        let error = Notebook::new(&mut storage)
            .start("task.demo", TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidRecord { .. }));
        assert_eq!(
            storage.read("tasks/task.demo.md").unwrap(),
            text,
            "an invalid record's bytes are never rewritten"
        );
    }

    #[test]
    fn a_dangling_origin_excludes_the_record_from_mutation() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &task_file("open", &["from: question.never-written"]),
        )]);
        let error = Notebook::new(&mut storage)
            .start("task.demo", TODAY)
            .unwrap_err();
        let NotebookError::InvalidRecord { findings, .. } = error else {
            panic!("expected InvalidRecord, got {error:?}");
        };
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, FindingCode::DanglingRef);
    }

    #[test]
    fn mutating_one_record_leaves_every_other_records_bytes_unchanged() {
        let other = task_file("open", &[]);
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("open", &[])),
            (
                "tasks/task.other.md",
                &other.replace("task.demo", "task.other"),
            ),
        ]);
        Notebook::new(&mut storage)
            .start("task.demo", TODAY)
            .unwrap();
        assert_eq!(
            storage.read("tasks/task.other.md").unwrap(),
            other.replace("task.demo", "task.other")
        );
    }
}

mod hold {
    use super::*;

    #[test]
    fn hold_requires_a_reason() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let error = Notebook::new(&mut storage)
            .hold("task.demo", "  ", None, TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidArgument { .. }));
    }

    #[test]
    fn hold_writes_the_reason_and_the_until_date_keeping_the_state() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let reply = Notebook::new(&mut storage)
            .hold(
                "task.demo",
                "waiting for the 1.99 release",
                Some("2026-09-10"),
                TODAY,
            )
            .unwrap();
        assert!(!reply.already);
        let text = storage.read("tasks/task.demo.md").unwrap();
        assert!(text.contains("\nstate: active\n"), "a hold keeps the state");
        assert!(text.contains("\nhold: waiting for the 1.99 release\n"));
        assert!(text.contains("\nhold-until: 2026-09-10\n"));
    }

    #[test]
    fn a_replayed_hold_answers_already_and_changes_no_byte() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        Notebook::new(&mut storage)
            .hold("task.demo", "a reason", None, TODAY)
            .unwrap();
        let after_first = storage.read("tasks/task.demo.md").unwrap();

        let replay = Notebook::new(&mut storage)
            .hold("task.demo", "a reason", None, TODAY)
            .unwrap();
        assert!(replay.already);
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), after_first);
    }

    #[test]
    fn a_hold_with_a_new_reason_replaces_the_standing_one() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        Notebook::new(&mut storage)
            .hold("task.demo", "old reason", Some("2026-09-10"), TODAY)
            .unwrap();
        let reply = Notebook::new(&mut storage)
            .hold("task.demo", "new reason", None, TODAY)
            .unwrap();
        assert!(!reply.already);
        let text = storage.read("tasks/task.demo.md").unwrap();
        assert!(text.contains("hold: new reason"));
        assert!(
            !text.contains("hold-until"),
            "a hold given without a date carries none"
        );
    }

    #[test]
    fn unhold_clears_the_hold_and_its_date() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &task_file("active", &["hold: a reason", "hold-until: 2026-09-10"]),
        )]);
        let reply = Notebook::new(&mut storage)
            .unhold("task.demo", TODAY)
            .unwrap();
        assert!(!reply.already);
        let text = storage.read("tasks/task.demo.md").unwrap();
        assert!(!text.contains("\nhold:"));
        assert!(!text.contains("\nhold-until:"));
    }

    #[test]
    fn unhold_on_an_unheld_task_is_a_replay() {
        let text = task_file("active", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        let reply = Notebook::new(&mut storage)
            .unhold("task.demo", TODAY)
            .unwrap();
        assert!(reply.already);
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
    }
}

mod task_log {
    use super::*;

    #[test]
    fn comment_appends_a_dated_authored_entry_and_stamps_updated() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let reply = Notebook::new(&mut storage)
            .comment(
                "task.demo",
                Some("claude-code"),
                "parser done, tests next",
                TODAY,
            )
            .unwrap();
        assert!(!reply.already);
        assert_eq!(
            reply.entry,
            "- 2026-08-27 claude-code: parser done, tests next"
        );
        assert_eq!(
            storage.read("tasks/task.demo.md").unwrap(),
            "---\nid: task.demo\ntype: task\nstate: active\ntitle: A demo record\ncreated: 2026-08-24\nupdated: 2026-08-27\n---\n- 2026-08-27 claude-code: parser done, tests next\n"
        );
    }

    #[test]
    fn an_entry_without_an_acting_hand_shows_the_dash() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let reply = Notebook::new(&mut storage)
            .comment("task.demo", None, "stopped at the render step", TODAY)
            .unwrap();
        assert_eq!(reply.entry, "- 2026-08-27 -: stopped at the render step");
    }

    #[test]
    fn replaying_the_last_entry_answers_already_and_changes_no_byte() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        Notebook::new(&mut storage)
            .comment("task.demo", Some("claude-code"), "same note", TODAY)
            .unwrap();
        let after_first = storage.read("tasks/task.demo.md").unwrap();

        let replay = Notebook::new(&mut storage)
            .comment("task.demo", Some("claude-code"), "same note", TODAY)
            .unwrap();
        assert!(replay.already);
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), after_first);
    }

    #[test]
    fn a_repeated_text_behind_a_newer_entry_is_appended_again() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let mut notebook = Notebook::new(&mut storage);
        notebook
            .comment("task.demo", Some("claude-code"), "first note", TODAY)
            .unwrap();
        notebook
            .comment("task.demo", Some("claude-code"), "second note", TODAY)
            .unwrap();
        let reply = notebook
            .comment("task.demo", Some("claude-code"), "first note", TODAY)
            .unwrap();
        assert!(!reply.already, "the log is a trail, not a set");
        let text = storage.read("tasks/task.demo.md").unwrap();
        assert_eq!(text.matches("first note").count(), 2);
    }

    #[test]
    fn an_empty_comment_is_refused_before_any_byte_moves() {
        let text = task_file("active", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        let error = Notebook::new(&mut storage)
            .comment("task.demo", None, "   ", TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidArgument { .. }));
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
    }

    #[test]
    fn a_comment_carrying_a_newline_cannot_inject_log_lines() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let error = Notebook::new(&mut storage)
            .comment("task.demo", None, "one line\n- 2026-08-27 -: forged", TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidArgument { .. }));
    }

    #[test]
    fn the_state_does_not_gate_the_log() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("closed", &[]))]);
        let reply = Notebook::new(&mut storage)
            .comment(
                "task.demo",
                None,
                "closed because the epic absorbed it",
                TODAY,
            )
            .unwrap();
        assert!(!reply.already);
    }

    #[test]
    fn a_comment_on_a_non_task_is_wrong_type() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file("note.demo", "note", "active", &[], ""),
        )]);
        let error = Notebook::new(&mut storage)
            .comment("note.demo", None, "a note", TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::WrongType { .. }));
    }
}

mod dependency_graph {
    use super::*;
    use anb_core::Edged;

    fn task(id: &str, state: &str, extra_lines: &[&str]) -> String {
        record_file(id, "task", state, extra_lines, "")
    }

    fn edged(id: &str, on: &str, already: bool) -> Edged {
        Edged {
            id: id.to_owned(),
            on: on.to_owned(),
            already,
        }
    }

    #[test]
    fn block_writes_the_edge_at_its_canonical_place_and_stamps_updated() {
        let mut storage = storage_with(&[
            ("tasks/task.a.md", &task("task.a", "open", &[])),
            ("tasks/task.b.md", &task("task.b", "open", &[])),
        ]);
        let reply = Notebook::new(&mut storage)
            .block("task.a", "task.b", TODAY)
            .unwrap();
        assert_eq!(reply, edged("task.a", "task.b", false));
        assert_eq!(
            storage.read("tasks/task.a.md").unwrap(),
            "---\nid: task.a\ntype: task\nstate: open\ntitle: A demo record\nblocked-by: task.b\ncreated: 2026-08-24\nupdated: 2026-08-27\n---\n"
        );
    }

    #[test]
    fn a_replayed_block_answers_already_true_and_changes_no_byte() {
        let mut storage = storage_with(&[
            ("tasks/task.a.md", &task("task.a", "open", &[])),
            ("tasks/task.b.md", &task("task.b", "open", &[])),
        ]);
        Notebook::new(&mut storage)
            .block("task.a", "task.b", TODAY)
            .unwrap();
        let after_first = storage.read("tasks/task.a.md").unwrap();

        let replay = Notebook::new(&mut storage)
            .block("task.a", "task.b", TODAY)
            .unwrap();
        assert_eq!(replay, edged("task.a", "task.b", true));
        assert_eq!(storage.read("tasks/task.a.md").unwrap(), after_first);
    }

    #[test]
    fn blocking_a_task_on_itself_is_refused_as_the_shortest_cycle() {
        let text = task("task.a", "open", &[]);
        let mut storage = storage_with(&[("tasks/task.a.md", &text)]);
        let error = Notebook::new(&mut storage)
            .block("task.a", "task.a", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::WouldCycle {
                chain: vec!["task.a".to_owned(), "task.a".to_owned()],
            }
        );
        assert_eq!(storage.read("tasks/task.a.md").unwrap(), text);
    }

    #[test]
    fn an_edge_that_would_close_a_cycle_is_refused_with_the_cycle_walked() {
        let text = task("task.b", "open", &[]);
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &task("task.a", "open", &["blocked-by: task.b"]),
            ),
            ("tasks/task.b.md", &text),
        ]);
        let error = Notebook::new(&mut storage)
            .block("task.b", "task.a", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::WouldCycle {
                chain: vec![
                    "task.b".to_owned(),
                    "task.a".to_owned(),
                    "task.b".to_owned()
                ],
            }
        );
        assert_eq!(storage.read("tasks/task.b.md").unwrap(), text);
    }

    #[test]
    fn a_cycle_through_an_intermediate_task_is_still_refused() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &task("task.a", "open", &["blocked-by: task.b"]),
            ),
            (
                "tasks/task.b.md",
                &task("task.b", "open", &["blocked-by: task.c"]),
            ),
            ("tasks/task.c.md", &task("task.c", "open", &[])),
        ]);
        let error = Notebook::new(&mut storage)
            .block("task.c", "task.a", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::WouldCycle {
                chain: vec![
                    "task.c".to_owned(),
                    "task.a".to_owned(),
                    "task.b".to_owned(),
                    "task.c".to_owned(),
                ],
            }
        );
    }

    #[test]
    fn blocking_on_a_task_never_written_is_a_dangling_ref() {
        let mut storage = storage_with(&[("tasks/task.a.md", &task("task.a", "open", &[]))]);
        let error = Notebook::new(&mut storage)
            .block("task.a", "task.never-written", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::DanglingRef {
                field: "blocked-by",
                target: "task.never-written".to_owned(),
            }
        );
    }

    #[test]
    fn blocking_on_a_record_that_is_not_a_task_is_refused() {
        let mut storage = storage_with(&[("tasks/task.a.md", &task("task.a", "open", &[]))]);
        let error = Notebook::new(&mut storage)
            .block("task.a", "decision.a-ruling", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::WrongType {
                id: "decision.a-ruling".to_owned(),
                expected: "a task".to_owned(),
            }
        );
    }

    #[test]
    fn unblock_erases_the_edge_and_stamps_updated() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &task("task.a", "open", &["blocked-by: task.b"]),
            ),
            ("tasks/task.b.md", &task("task.b", "open", &[])),
        ]);
        let reply = Notebook::new(&mut storage)
            .unblock("task.a", "task.b", TODAY)
            .unwrap();
        assert_eq!(reply, edged("task.a", "task.b", false));
        assert_eq!(
            storage.read("tasks/task.a.md").unwrap(),
            "---\nid: task.a\ntype: task\nstate: open\ntitle: A demo record\ncreated: 2026-08-24\nupdated: 2026-08-27\n---\n"
        );
    }

    #[test]
    fn unblocking_an_edge_that_is_not_there_is_a_replay() {
        let text = task("task.a", "open", &[]);
        let mut storage = storage_with(&[("tasks/task.a.md", &text)]);
        let reply = Notebook::new(&mut storage)
            .unblock("task.a", "task.b", TODAY)
            .unwrap();
        assert_eq!(reply, edged("task.a", "task.b", true));
        assert_eq!(storage.read("tasks/task.a.md").unwrap(), text);
    }

    #[test]
    fn blocking_through_the_archive_is_still_refused_as_a_cycle() {
        let mut storage = storage_with(&[
            ("tasks/task.a.md", &task("task.a", "open", &[])),
            (
                "archive/tasks/task.done.md",
                &task("task.done", "closed", &["blocked-by: task.a"]),
            ),
        ]);
        let error = Notebook::new(&mut storage)
            .block("task.a", "task.done", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::WouldCycle {
                chain: vec![
                    "task.a".to_owned(),
                    "task.done".to_owned(),
                    "task.a".to_owned(),
                ],
            },
            "a closed Task can be reopened, so a latent cycle is a real one"
        );
    }

    #[test]
    fn unblock_repairs_a_task_waiting_on_itself() {
        let mut storage = storage_with(&[(
            "tasks/task.a.md",
            &task("task.a", "open", &["blocked-by: task.a"]),
        )]);
        let reply = Notebook::new(&mut storage)
            .unblock("task.a", "task.a", TODAY)
            .unwrap();
        assert_eq!(reply, edged("task.a", "task.a", false));
        assert!(
            !storage
                .read("tasks/task.a.md")
                .unwrap()
                .contains("blocked-by"),
        );
    }

    #[test]
    fn unblock_erases_an_edge_into_a_non_task() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &task("task.a", "open", &["blocked-by: note.a-fact"]),
            ),
            (
                "notes/note.a-fact.md",
                &record_file("note.a-fact", "note", "active", &[], ""),
            ),
        ]);
        let reply = Notebook::new(&mut storage)
            .unblock("task.a", "note.a-fact", TODAY)
            .unwrap();
        assert_eq!(reply, edged("task.a", "note.a-fact", false));
        assert!(
            !storage
                .read("tasks/task.a.md")
                .unwrap()
                .contains("blocked-by"),
        );
    }

    #[test]
    fn unblock_erases_a_dangling_edge() {
        let mut storage = storage_with(&[(
            "tasks/task.a.md",
            &task("task.a", "open", &["blocked-by: task.never-written"]),
        )]);
        let reply = Notebook::new(&mut storage)
            .unblock("task.a", "task.never-written", TODAY)
            .unwrap();
        assert_eq!(reply, edged("task.a", "task.never-written", false));
        assert!(
            !storage
                .read("tasks/task.a.md")
                .unwrap()
                .contains("blocked-by"),
        );
    }

    #[test]
    fn a_corrupted_edge_freezes_every_verb_but_unblock() {
        let text = task("task.a", "open", &["blocked-by: task.a"]);
        let mut storage = storage_with(&[("tasks/task.a.md", &text)]);
        let error = Notebook::new(&mut storage)
            .start("task.a", TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidRecord { .. }));
        assert_eq!(storage.read("tasks/task.a.md").unwrap(), text);
    }

    #[test]
    fn a_task_in_a_hand_edited_multi_file_cycle_still_moves() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &task("task.a", "open", &["blocked-by: task.b"]),
            ),
            (
                "tasks/task.b.md",
                &task("task.b", "open", &["blocked-by: task.a"]),
            ),
        ]);
        let reply = Notebook::new(&mut storage).start("task.a", TODAY).unwrap();
        assert!(
            !reply.already,
            "a finding that needs a second record is check's alone and freezes nothing"
        );
    }

    #[test]
    fn unblock_erases_a_hand_edited_duplicate_edge_whole() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &task(
                    "task.a",
                    "open",
                    &["blocked-by: task.b", "blocked-by: task.b"],
                ),
            ),
            ("tasks/task.b.md", &task("task.b", "open", &[])),
        ]);
        Notebook::new(&mut storage)
            .unblock("task.a", "task.b", TODAY)
            .unwrap();
        assert!(
            !storage
                .read("tasks/task.a.md")
                .unwrap()
                .contains("blocked-by"),
            "a half-erased edge would keep the task blocked and the replay false"
        );
    }

    #[test]
    fn close_names_the_open_tasks_whose_last_live_blocker_it_was() {
        let mut storage = storage_with(&[
            ("tasks/task.done.md", &task("task.done", "active", &[])),
            (
                "tasks/task.freed.md",
                &task("task.freed", "open", &["blocked-by: task.done"]),
            ),
            (
                "tasks/task.still-blocked.md",
                &task(
                    "task.still-blocked",
                    "open",
                    &["blocked-by: task.done", "blocked-by: task.other"],
                ),
            ),
            (
                "tasks/task.already-active.md",
                &task("task.already-active", "active", &["blocked-by: task.done"]),
            ),
            ("tasks/task.other.md", &task("task.other", "open", &[])),
        ]);
        let closed = Notebook::new(&mut storage)
            .close("task.done", &Proof::Waived, TODAY)
            .unwrap();
        assert_eq!(closed.unblocked, vec!["task.freed"]);
    }

    #[test]
    fn a_freed_task_on_hold_is_still_named_by_the_close() {
        let mut storage = storage_with(&[
            ("tasks/task.done.md", &task("task.done", "active", &[])),
            (
                "tasks/task.freed-but-held.md",
                &task(
                    "task.freed-but-held",
                    "open",
                    &["blocked-by: task.done", "hold: waiting on a decision"],
                ),
            ),
        ]);
        let closed = Notebook::new(&mut storage)
            .close("task.done", &Proof::Waived, TODAY)
            .unwrap();
        assert_eq!(
            closed.unblocked,
            vec!["task.freed-but-held"],
            "the hold gates `ready`, not the fact of unblocking"
        );
    }

    #[test]
    fn close_names_the_freed_tasks_in_ready_order() {
        let mut storage = storage_with(&[
            ("tasks/task.done.md", &task("task.done", "active", &[])),
            (
                "tasks/task.background.md",
                &task(
                    "task.background",
                    "open",
                    &["blocked-by: task.done", "priority: 4"],
                ),
            ),
            (
                "tasks/task.urgent.md",
                &task(
                    "task.urgent",
                    "open",
                    &["blocked-by: task.done", "priority: 0"],
                ),
            ),
        ]);
        let closed = Notebook::new(&mut storage)
            .close("task.done", &Proof::Waived, TODAY)
            .unwrap();
        assert_eq!(closed.unblocked, vec!["task.urgent", "task.background"]);
    }
}

mod ready_queue {
    use super::*;
    use anb_core::ReadyTask;

    fn task(id: &str, state: &str, extra_lines: &[&str]) -> String {
        record_file(id, "task", state, extra_lines, "")
    }

    fn ready_ids(storage: &mut MemoryStorage) -> Vec<String> {
        Notebook::new(storage)
            .ready()
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect()
    }

    fn task_created_on(id: &str, created: &str, extra_lines: &[&str]) -> String {
        task(id, "open", extra_lines).replace("created: 2026-08-24", &format!("created: {created}"))
    }

    #[test]
    fn ready_lists_only_open_unblocked_unheld_tasks() {
        let mut storage = storage_with(&[
            (
                "tasks/task.pickable.md",
                &task("task.pickable", "open", &[]),
            ),
            (
                "tasks/task.blocked.md",
                &task("task.blocked", "open", &["blocked-by: task.pickable"]),
            ),
            (
                "tasks/task.held.md",
                &task("task.held", "open", &["hold: parked for the release"]),
            ),
            ("tasks/task.active.md", &task("task.active", "active", &[])),
            ("tasks/task.closed.md", &task("task.closed", "closed", &[])),
        ]);
        assert_eq!(ready_ids(&mut storage), vec!["task.pickable"]);
    }

    #[test]
    fn a_closed_blocker_blocks_nothing() {
        let mut storage = storage_with(&[
            (
                "tasks/task.waited.md",
                &task("task.waited", "open", &["blocked-by: task.closed"]),
            ),
            ("tasks/task.closed.md", &task("task.closed", "closed", &[])),
        ]);
        assert_eq!(ready_ids(&mut storage), vec!["task.waited"]);
    }

    #[test]
    fn ready_ranks_priority_first_then_the_oldest_then_the_id() {
        let mut storage = storage_with(&[
            (
                "tasks/task.urgent.md",
                &task("task.urgent", "open", &["priority: 1"]),
            ),
            (
                "tasks/task.background.md",
                &task("task.background", "open", &["priority: 3"]),
            ),
            (
                "tasks/task.old.md",
                &task_created_on("task.old", "2026-08-01", &["priority: 2"]),
            ),
            (
                "tasks/task.same-day-b.md",
                &task("task.same-day-b", "open", &["priority: 2"]),
            ),
            (
                "tasks/task.same-day-a.md",
                &task("task.same-day-a", "open", &["priority: 2"]),
            ),
        ]);
        assert_eq!(
            ready_ids(&mut storage),
            vec![
                "task.urgent",
                "task.old",
                "task.same-day-a",
                "task.same-day-b",
                "task.background"
            ]
        );
    }

    #[test]
    fn an_unprioritized_task_ranks_at_the_neutral_middle() {
        let mut storage = storage_with(&[
            (
                "tasks/task.urgent.md",
                &task("task.urgent", "open", &["priority: 1"]),
            ),
            (
                "tasks/task.untriaged.md",
                &task("task.untriaged", "open", &[]),
            ),
            (
                "tasks/task.background.md",
                &task("task.background", "open", &["priority: 3"]),
            ),
        ]);
        assert_eq!(
            ready_ids(&mut storage),
            vec!["task.urgent", "task.untriaged", "task.background"]
        );
    }

    #[test]
    fn an_invalid_task_is_excluded_from_the_queue() {
        let mut storage = storage_with(&[
            (
                "tasks/task.dangling.md",
                &task("task.dangling", "open", &["blocked-by: task.never-written"]),
            ),
            ("tasks/task.sound.md", &task("task.sound", "open", &[])),
        ]);
        assert_eq!(
            ready_ids(&mut storage),
            vec!["task.sound"],
            "an invalid record is `check`'s to name, never a silent queue entry"
        );
    }

    #[test]
    fn a_row_carries_what_the_queue_prints() {
        let mut storage = storage_with(&[(
            "tasks/task.pickable.md",
            &task("task.pickable", "open", &["priority: 1"]),
        )]);
        let rows = Notebook::new(&mut storage).ready().unwrap();
        assert_eq!(
            rows,
            vec![ReadyTask {
                id: "task.pickable".to_owned(),
                priority: Some(1),
                created: "2026-08-24".to_owned(),
                title: "A demo record".to_owned(),
            }]
        );
    }
}

mod listing {
    use super::*;
    use anb_core::ListedRecord;

    #[test]
    fn list_shows_live_valid_records_of_every_type_in_type_major_order() {
        let mut storage = storage_with(&[
            (
                "questions/question.q.md",
                &record_file("question.q", "question", "open", &[], ""),
            ),
            (
                "tasks/task.b.md",
                &record_file("task.b", "task", "open", &["priority: 1"], ""),
            ),
            (
                "tasks/task.a.md",
                &record_file("task.a", "task", "active", &[], ""),
            ),
            (
                "decisions/decision.d.md",
                &record_file("decision.d", "decision", "active", &[], ""),
            ),
            (
                "archive/notes/note.gone.md",
                &record_file("note.gone", "note", "retired", &[], ""),
            ),
        ]);
        let rows = Notebook::new(&mut storage).list().unwrap();
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["task.a", "task.b", "decision.d", "question.q"],
            "tasks first in file order, then the other types; the archive is history"
        );
        assert_eq!(
            rows[1],
            ListedRecord {
                id: "task.b".into(),
                state: "open".into(),
                priority: Some(1),
                title: Some("A demo record".into()),
            }
        );
        assert_eq!(
            rows[2].priority, None,
            "a record without the field carries none"
        );
    }

    #[test]
    fn an_invalid_record_lists_as_state_invalid() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &record_file("task.a", "task", "open", &[], ""),
            ),
            (
                "tasks/task.broken.md",
                &record_file("task.broken", "task", "cancelled", &[], ""),
            ),
        ]);
        let rows = Notebook::new(&mut storage).list().unwrap();
        let states: Vec<(&str, &str)> = rows
            .iter()
            .map(|row| (row.id.as_str(), row.state.as_str()))
            .collect();
        assert_eq!(
            states,
            vec![("task.a", "open"), ("task.broken", "invalid")],
            "visible as invalid, never silently dropped; check names the findings"
        );
    }
}

mod record_view {
    use super::*;

    #[test]
    fn view_shows_the_envelope_as_it_stands_and_the_body_verbatim() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &record_file(
                "task.demo",
                "task",
                "active",
                &["custom: kept"],
                "The plan.\n\nStep one.\n",
            ),
        )]);
        let view = Notebook::new(&mut storage).view("task.demo").unwrap();
        assert_eq!(view.path, "tasks/task.demo.md");
        assert!(!view.archived);
        assert_eq!(
            view.fields,
            vec![
                ("id".to_owned(), "task.demo".to_owned()),
                ("type".to_owned(), "task".to_owned()),
                ("state".to_owned(), "active".to_owned()),
                ("title".to_owned(), "A demo record".to_owned()),
                ("custom".to_owned(), "kept".to_owned()),
                ("created".to_owned(), "2026-08-24".to_owned()),
                ("updated".to_owned(), "2026-08-25".to_owned()),
            ]
        );
        assert_eq!(view.body, "The plan.\n\nStep one.\n");
    }

    #[test]
    fn the_mention_blocks_cite_and_are_cited_by() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &record_file(
                    "task.demo",
                    "task",
                    "active",
                    &[],
                    "Follows decision.chosen; question.missing is still open.\n",
                ),
            ),
            (
                "decisions/decision.chosen.md",
                &record_file("decision.chosen", "decision", "active", &[], ""),
            ),
            (
                "notes/note.citing.md",
                &record_file(
                    "note.citing",
                    "note",
                    "active",
                    &[],
                    "Background for task.demo.\n",
                ),
            ),
        ]);
        let view = Notebook::new(&mut storage).view("task.demo").unwrap();
        assert_eq!(
            view.mentions,
            vec!["decision.chosen", "question.missing"],
            "a citation is a hint, so a dangling one still shows"
        );
        assert_eq!(view.mentioned_by, vec!["note.citing"]);
    }

    #[test]
    fn a_quoted_id_stays_out_of_the_mention_blocks() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &record_file(
                    "task.demo",
                    "task",
                    "active",
                    &[],
                    "Rename `decision.chosen` before question.missing settles.\n",
                ),
            ),
            (
                "decisions/decision.chosen.md",
                &record_file("decision.chosen", "decision", "active", &[], ""),
            ),
        ]);
        let view = Notebook::new(&mut storage).view("task.demo").unwrap();
        assert_eq!(view.mentions, vec!["question.missing"]);
        let quoted = Notebook::new(&mut storage).view("decision.chosen").unwrap();
        assert_eq!(quoted.mentioned_by, Vec::<String>::new());
    }

    #[test]
    fn a_record_never_enters_its_own_mention_blocks() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &record_file(
                "task.demo",
                "task",
                "active",
                &[],
                "This body cites task.demo itself.\n",
            ),
        )]);
        let view = Notebook::new(&mut storage).view("task.demo").unwrap();
        assert_eq!(view.mentions, Vec::<String>::new());
        assert_eq!(view.mentioned_by, Vec::<String>::new());
    }

    #[test]
    fn mentioned_by_reads_only_live_records() {
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("active", &[])),
            (
                "archive/notes/note.gone.md",
                &record_file(
                    "note.gone",
                    "note",
                    "retired",
                    &[],
                    "Once cited task.demo.\n",
                ),
            ),
        ]);
        let view = Notebook::new(&mut storage).view("task.demo").unwrap();
        assert_eq!(view.mentioned_by, Vec::<String>::new());
    }

    #[test]
    fn view_of_an_archived_record_marks_it_archived() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.done.md",
            &record_file("task.done", "task", "closed", &[], ""),
        )]);
        let view = Notebook::new(&mut storage).view("task.done").unwrap();
        assert!(view.archived);
        assert_eq!(view.path, "archive/tasks/task.done.md");
    }

    #[test]
    fn view_of_an_unknown_id_is_unknown_id() {
        let mut storage = storage_with(&[]);
        let error = Notebook::new(&mut storage).view("task.absent").unwrap_err();
        assert!(matches!(error, NotebookError::UnknownId { .. }));
    }
}

mod routing {
    use super::*;

    fn question_notebook() -> MemoryStorage {
        storage_with(&[
            (
                "questions/question.demo.md",
                &record_file("question.demo", "question", "open", &[], ""),
            ),
            (
                "decisions/decision.the-answer.md",
                &record_file("decision.the-answer", "decision", "active", &[], ""),
            ),
        ])
    }

    #[test]
    fn route_closes_the_question_into_what_its_answer_became() {
        let mut storage = question_notebook();
        let reply = Notebook::new(&mut storage)
            .route("question.demo", "decision.the-answer", TODAY)
            .unwrap();
        assert_eq!(reply, moved("question.demo", "open", "routed"));
        let text = storage.read("questions/question.demo.md").unwrap();
        assert!(text.contains("\nstate: routed\n"));
        assert!(text.contains("\nrouted-to: decision.the-answer\n"));
    }

    #[test]
    fn a_replayed_route_to_the_same_target_answers_already() {
        let mut storage = question_notebook();
        Notebook::new(&mut storage)
            .route("question.demo", "decision.the-answer", TODAY)
            .unwrap();
        let after_first = storage.read("questions/question.demo.md").unwrap();

        let replay = Notebook::new(&mut storage)
            .route("question.demo", "decision.the-answer", TODAY)
            .unwrap();
        assert!(replay.already);
        assert_eq!(
            storage.read("questions/question.demo.md").unwrap(),
            after_first
        );
    }

    #[test]
    fn a_routed_question_cannot_be_rerouted() {
        let mut storage = question_notebook();
        storage
            .write(
                "tasks/task.other-answer.md",
                &record_file("task.other-answer", "task", "open", &[], ""),
            )
            .unwrap();
        Notebook::new(&mut storage)
            .route("question.demo", "decision.the-answer", TODAY)
            .unwrap();
        let error = Notebook::new(&mut storage)
            .route("question.demo", "task.other-answer", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::InvalidTransition {
                id: "question.demo".to_owned(),
                state: "routed".to_owned(),
                valid: vec![],
            }
        );
    }

    #[test]
    fn a_question_routes_only_into_a_decision_or_a_task() {
        let mut storage = question_notebook();
        storage
            .write(
                "notes/note.a-fact.md",
                &record_file("note.a-fact", "note", "active", &[], ""),
            )
            .unwrap();
        let error = Notebook::new(&mut storage)
            .route("question.demo", "note.a-fact", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::WrongType {
                id: "note.a-fact".to_owned(),
                expected: "a decision or a task".to_owned(),
            }
        );
    }

    #[test]
    fn a_route_into_nothing_is_a_dangling_ref() {
        let mut storage = question_notebook();
        let error = Notebook::new(&mut storage)
            .route("question.demo", "decision.never-written", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::DanglingRef {
                field: "routed-to",
                target: "decision.never-written".to_owned(),
            }
        );
    }

    #[test]
    fn drop_closes_the_question_with_its_stated_reason_in_the_body() {
        let mut storage = question_notebook();
        let reply = Notebook::new(&mut storage)
            .drop_question("question.demo", "overtaken by a newer decision", TODAY)
            .unwrap();
        assert_eq!(reply.transition, moved("question.demo", "open", "dropped"));
        let text = storage.read("questions/question.demo.md").unwrap();
        assert!(text.contains("\nstate: dropped\n"));
        assert!(text.ends_with("---\nDropped 2026-08-27: overtaken by a newer decision\n"));
    }

    #[test]
    fn a_replayed_drop_appends_no_second_reason_line() {
        let mut storage = question_notebook();
        Notebook::new(&mut storage)
            .drop_question("question.demo", "a reason", TODAY)
            .unwrap();
        let after_first = storage.read("questions/question.demo.md").unwrap();

        let replay = Notebook::new(&mut storage)
            .drop_question("question.demo", "a reason", TODAY)
            .unwrap();
        assert!(replay.transition.already);
        assert_eq!(
            storage.read("questions/question.demo.md").unwrap(),
            after_first
        );
    }

    #[test]
    fn a_routed_question_with_a_dangling_thread_is_broken_routing_on_every_surface() {
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
            &record_file(
                "question.demo",
                "question",
                "routed",
                &["routed-to: decision.gone"],
                "",
            ),
        )]);
        let error = Notebook::new(&mut storage)
            .drop_question("question.demo", "a reason", TODAY)
            .unwrap_err();
        let NotebookError::InvalidRecord { findings, .. } = error else {
            panic!("expected InvalidRecord, got {error:?}");
        };
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].code,
            FindingCode::BrokenRouting,
            "check and the mutation guard must name one condition with one code"
        );
    }

    #[test]
    fn drop_requires_a_reason() {
        let mut storage = question_notebook();
        let error = Notebook::new(&mut storage)
            .drop_question("question.demo", " ", TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidArgument { .. }));
    }
}

mod retirement {
    use super::*;

    #[test]
    fn retire_ends_an_active_decision_without_a_successor() {
        let mut storage = storage_with(&[(
            "decisions/decision.demo.md",
            &record_file("decision.demo", "decision", "active", &[], ""),
        )]);
        let reply = Notebook::new(&mut storage)
            .retire("decision.demo", TODAY)
            .unwrap();
        assert_eq!(reply, moved("decision.demo", "active", "retired"));
    }

    #[test]
    fn a_superseded_decision_is_settled_and_cannot_retire() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.demo.md",
                &record_file(
                    "decision.demo",
                    "decision",
                    "superseded",
                    &["superseded-by: decision.newer"],
                    "",
                ),
            ),
            (
                "decisions/decision.newer.md",
                &record_file(
                    "decision.newer",
                    "decision",
                    "active",
                    &["supersedes: decision.demo"],
                    "",
                ),
            ),
        ]);
        let error = Notebook::new(&mut storage)
            .retire("decision.demo", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::InvalidTransition {
                id: "decision.demo".to_owned(),
                state: "superseded".to_owned(),
                valid: vec![],
            }
        );
    }

    #[test]
    fn retire_acts_only_on_a_decision_or_a_note() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let error = Notebook::new(&mut storage)
            .retire("task.demo", TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::WrongType {
                id: "task.demo".to_owned(),
                expected: "a decision or a note".to_owned(),
            }
        );
    }
}

mod creation {
    use super::*;

    #[test]
    fn create_mints_the_id_from_the_title_and_writes_the_canonical_file() {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Task, "Grammar parser accepts fenced envelopes");
        draft.by = Some("supolka".to_owned());
        draft.via = Some("claude-code".to_owned());
        draft.tags = vec!["core".to_owned(), "parser".to_owned()];
        draft.priority = Some(1);
        draft.body = "The why before the what.".to_owned();

        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.id, "task.grammar-parser-accepts-fenced-envelopes");
        assert_eq!(created.superseded, None);
        assert_eq!(
            storage.read(&created.path).unwrap(),
            "---\n\
             id: task.grammar-parser-accepts-fenced-envelopes\n\
             type: task\n\
             state: open\n\
             title: Grammar parser accepts fenced envelopes\n\
             by: supolka\n\
             via: claude-code\n\
             tags: core, parser\n\
             priority: 1\n\
             created: 2026-08-27\n\
             updated: 2026-08-27\n\
             ---\n\
             \n\
             The why before the what.\n"
        );
    }

    #[test]
    fn a_caller_supplied_id_that_is_taken_names_its_holder() {
        let mut storage =
            storage_with(&[("archive/tasks/task.demo.md", &task_file("closed", &[]))]);
        let mut draft = Draft::new(RecordType::Task, "Another demo");
        draft.id = Some("task.demo".to_owned());
        let error = Notebook::new(&mut storage)
            .create(&draft, TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::DuplicateId {
                id: "task.demo".to_owned(),
                holder: "archive/tasks/task.demo.md".to_owned(),
            },
            "ids are never reused, archive included"
        );
    }

    /// The id `create` mints for `title`, with nothing else in the way.
    fn minted_from(title: &str) -> String {
        let mut storage = MemoryStorage::new();
        Notebook::new(&mut storage)
            .create(&Draft::new(RecordType::Task, title), TODAY)
            .unwrap()
            .id
    }

    #[test]
    fn a_long_title_is_cut_at_a_word_boundary_so_every_kept_word_survives() {
        assert_eq!(
            minted_from("GitHub dev flow: Actions CI, fmt + clippy + tests"),
            "task.github-dev-flow-actions-ci-fmt-clippy"
        );
        assert_eq!(
            minted_from("Epic pattern: scoped queries + Status hub grouping"),
            "task.epic-pattern-scoped-queries-status-hub"
        );
    }

    #[test]
    fn a_title_that_fits_keeps_every_word_it_has() {
        assert_eq!(
            minted_from("Grammar parser accepts fenced envelopes"),
            "task.grammar-parser-accepts-fenced-envelopes"
        );
    }

    #[test]
    fn a_boundary_landing_on_the_cap_keeps_the_word_before_it() {
        // Forty characters of whole words, then one more word: the cut has
        // a boundary to take at the cap itself.
        let title = "aaaa bbbb cccc dddd eeee ffff gggg hhhhi jjjj";
        assert_eq!(
            minted_from(title),
            "task.aaaa-bbbb-cccc-dddd-eeee-ffff-gggg-hhhhi"
        );
    }

    #[test]
    fn a_first_word_longer_than_the_cap_is_cut_short_for_want_of_a_boundary() {
        let id = minted_from(&"z".repeat(60));
        assert_eq!(id, format!("task.{}", "z".repeat(40)));
    }

    #[test]
    fn a_mint_collision_retries_with_a_two_character_suffix() {
        let mut storage = MemoryStorage::new();
        let draft = Draft::new(RecordType::Task, "A demo record");
        let first = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        let second = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(first.id, "task.a-demo-record");
        assert_eq!(second.id.len(), first.id.len() + 3);
        assert!(second.id.starts_with("task.a-demo-record-"));
        assert!(storage.read(&second.path).is_ok());
    }

    #[test]
    fn a_decision_created_with_supersedes_flips_its_victim_in_the_same_move() {
        let mut storage = storage_with(&[(
            "decisions/decision.go-for-the-cli.md",
            &record_file(
                "decision.go-for-the-cli",
                "decision",
                "active",
                &[],
                "Go.\n",
            ),
        )]);
        let mut draft = Draft::new(RecordType::Decision, "Rust for the CLI");
        draft.kind = Some("shape".to_owned());
        draft.supersedes = Some("decision.go-for-the-cli".to_owned());

        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(
            created.superseded,
            Some("decision.go-for-the-cli".to_owned())
        );
        assert_eq!(
            storage
                .read("decisions/decision.go-for-the-cli.md")
                .unwrap(),
            "---\nid: decision.go-for-the-cli\ntype: decision\nstate: superseded\ntitle: A demo record\nsuperseded-by: decision.rust-for-the-cli\ncreated: 2026-08-24\nupdated: 2026-08-27\n---\nGo.\n",
            "the victim gains the back-pointer and can never again read as live"
        );
        assert!(
            storage
                .read(&created.path)
                .unwrap()
                .contains("supersedes: decision.go-for-the-cli\n")
        );
    }

    #[test]
    fn a_note_superseded_by_its_successor_retires() {
        let mut storage = storage_with(&[(
            "notes/note.old-fact.md",
            &record_file("note.old-fact", "note", "active", &[], ""),
        )]);
        let mut draft = Draft::new(RecordType::Note, "The corrected fact");
        draft.kind = Some("fact".to_owned());
        draft.supersedes = Some("note.old-fact".to_owned());
        Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert!(
            storage
                .read("notes/note.old-fact.md")
                .unwrap()
                .contains("state: retired")
        );
    }

    #[test]
    fn a_victim_already_superseded_names_its_standing_superseder() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.demo.md",
                &record_file(
                    "decision.demo",
                    "decision",
                    "superseded",
                    &["superseded-by: decision.newer"],
                    "",
                ),
            ),
            (
                "decisions/decision.newer.md",
                &record_file(
                    "decision.newer",
                    "decision",
                    "active",
                    &["supersedes: decision.demo"],
                    "",
                ),
            ),
        ]);
        let mut draft = Draft::new(RecordType::Decision, "A third ruling");
        draft.supersedes = Some("decision.demo".to_owned());
        let error = Notebook::new(&mut storage)
            .create(&draft, TODAY)
            .unwrap_err();
        let NotebookError::CannotSupersede { id, reason } = error else {
            panic!("expected CannotSupersede, got {error:?}");
        };
        assert_eq!(id, "decision.demo");
        assert!(reason.contains("decision.newer"));
    }

    #[test]
    fn a_victim_excluded_from_mutation_cannot_be_superseded_either() {
        let victim = record_file(
            "decision.demo",
            "decision",
            "active",
            &["from: task.never-written"],
            "",
        );
        let mut storage = storage_with(&[("decisions/decision.demo.md", &victim)]);
        let mut draft = Draft::new(RecordType::Decision, "A newer ruling");
        draft.supersedes = Some("decision.demo".to_owned());
        let error = Notebook::new(&mut storage)
            .create(&draft, TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidRecord { .. }));
        assert_eq!(
            storage.read("decisions/decision.demo.md").unwrap(),
            victim,
            "a record excluded from mutation is never rewritten"
        );
        assert!(
            storage
                .read("decisions/decision.a-newer-ruling.md")
                .is_err(),
            "a refused supersession creates nothing"
        );
    }

    #[test]
    fn a_task_cannot_be_superseded() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let mut draft = Draft::new(RecordType::Decision, "A ruling over a task");
        draft.supersedes = Some("task.demo".to_owned());
        let error = Notebook::new(&mut storage)
            .create(&draft, TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::CannotSupersede { .. }));
    }

    #[test]
    fn a_draft_of_a_type_that_cannot_die_by_supersession_cannot_declare_it() {
        let mut draft = Draft::new(RecordType::Task, "A task claiming supersession");
        draft.supersedes = Some("task.demo".to_owned());
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let error = Notebook::new(&mut storage)
            .create(&draft, TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidArgument { .. }));
    }

    #[test]
    fn an_origin_that_does_not_exist_is_a_dangling_ref() {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Question, "A doubt from nowhere");
        draft.from = Some("task.never-written".to_owned());
        let error = Notebook::new(&mut storage)
            .create(&draft, TODAY)
            .unwrap_err();
        assert_eq!(
            error,
            NotebookError::DanglingRef {
                field: "from",
                target: "task.never-written".to_owned(),
            }
        );
    }

    #[test]
    fn an_archived_origin_still_counts_as_existing() {
        let mut storage =
            storage_with(&[("archive/tasks/task.shipped.md", &task_file("closed", &[]))]);
        let mut draft = Draft::new(RecordType::Note, "Knowledge born from shipped work");
        draft.kind = Some("fact".to_owned());
        draft.from = Some("task.shipped".to_owned());
        assert!(Notebook::new(&mut storage).create(&draft, TODAY).is_ok());
    }

    #[test]
    fn a_kind_outside_the_types_enum_is_refused_naming_the_set() {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Decision, "A ruling");
        draft.kind = Some("law".to_owned());
        let error = Notebook::new(&mut storage)
            .create(&draft, TODAY)
            .unwrap_err();
        let NotebookError::InvalidArgument { reason } = error else {
            panic!("expected InvalidArgument, got {error:?}");
        };
        assert!(reason.contains("rule, shape, drift"));
    }

    #[test]
    fn a_kind_on_a_kindless_type_is_refused() {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Task, "A task");
        draft.kind = Some("feature".to_owned());
        assert!(matches!(
            Notebook::new(&mut storage)
                .create(&draft, TODAY)
                .unwrap_err(),
            NotebookError::InvalidArgument { .. }
        ));
    }

    #[test]
    fn links_render_as_kind_target_lines() {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Task, "A linked task");
        draft.links = vec![Link {
            kind: "doc".to_owned(),
            target: "docs/format.md".to_owned(),
        }];
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert!(
            storage
                .read(&created.path)
                .unwrap()
                .contains("link: doc docs/format.md\n")
        );
    }
}

mod conflict_nudge {
    use super::*;
    use anb_core::Cited;

    fn decision_draft(tags: &[&str]) -> Draft {
        let mut draft = Draft::new(RecordType::Decision, "A second ruling");
        draft.tags = tags.iter().map(|tag| (*tag).to_owned()).collect();
        draft
    }

    fn standing_decision(extra_lines: &[&str]) -> (&'static str, String) {
        (
            "decisions/decision.first.md",
            record_file("decision.first", "decision", "active", extra_lines, ""),
        )
    }

    #[test]
    fn a_decision_sharing_two_tags_with_the_draft_is_named_with_its_authors() {
        let (path, text) =
            standing_decision(&["by: supolka", "via: claude-code", "tags: parser, grammar"]);
        let mut storage = storage_with(&[(path, &text)]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["grammar", "cli", "parser"]), TODAY)
            .unwrap();
        assert_eq!(
            created.may_conflict,
            vec![Cited {
                id: "decision.first".to_owned(),
                by: Some("supolka".to_owned()),
                via: Some("claude-code".to_owned()),
            }]
        );
    }

    #[test]
    fn a_decision_cited_in_the_draft_body_is_named() {
        let (path, text) = standing_decision(&[]);
        let mut storage = storage_with(&[(path, &text)]);
        let mut draft = decision_draft(&[]);
        draft.body = "Refines decision.first for fenced blocks.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.may_conflict.len(), 1);
        assert_eq!(created.may_conflict[0].id, "decision.first");
    }

    #[test]
    fn a_quoted_decision_id_is_not_a_conflict_hint() {
        let (path, text) = standing_decision(&[]);
        let mut storage = storage_with(&[(path, &text)]);
        let mut draft = decision_draft(&[]);
        draft.body = "Renames the `decision.first` rule file.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn one_shared_tag_is_not_a_conflict_hint() {
        let (path, text) = standing_decision(&["tags: parser, grammar"]);
        let mut storage = storage_with(&[(path, &text)]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser", "cli"]), TODAY)
            .unwrap();
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn two_copies_of_one_tag_are_one_shared_tag() {
        let (path, text) = standing_decision(&["tags: parser, grammar"]);
        let mut storage = storage_with(&[(path, &text)]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser", "parser"]), TODAY)
            .unwrap();
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn a_tag_repeated_in_the_standing_list_counts_once() {
        let (path, text) = standing_decision(&["tags: parser, parser"]);
        let mut storage = storage_with(&[(path, &text)]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser"]), TODAY)
            .unwrap();
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn a_declared_supersession_carries_no_nudge() {
        let (path, text) = standing_decision(&["tags: parser, grammar"]);
        let mut storage = storage_with(&[(path, &text)]);
        let mut draft = decision_draft(&["parser", "grammar"]);
        draft.supersedes = Some("decision.first".to_owned());
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.superseded, Some("decision.first".to_owned()));
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn only_a_live_decision_is_named() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.retired.md",
                &record_file(
                    "decision.retired",
                    "decision",
                    "retired",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
            (
                "archive/decisions/decision.archived.md",
                &record_file(
                    "decision.archived",
                    "decision",
                    "active",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
            (
                "decisions/decision.live.md",
                &record_file(
                    "decision.live",
                    "decision",
                    "active",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
        ]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser", "grammar"]), TODAY)
            .unwrap();
        assert_eq!(created.may_conflict.len(), 1);
        assert_eq!(created.may_conflict[0].id, "decision.live");
    }

    #[test]
    fn a_decision_excluded_from_derived_queries_is_not_named() {
        let (path, text) =
            standing_decision(&["from: task.never-written", "tags: parser, grammar"]);
        let mut storage = storage_with(&[(path, &text)]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser", "grammar"]), TODAY)
            .unwrap();
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn a_note_draft_is_never_nudged() {
        let (path, text) = standing_decision(&["tags: parser, grammar"]);
        let mut storage = storage_with(&[(path, &text)]);
        let mut draft = Draft::new(RecordType::Note, "A fact beside the rulings");
        draft.kind = Some("fact".to_owned());
        draft.tags = vec!["parser".to_owned(), "grammar".to_owned()];
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn only_decisions_are_candidates() {
        let mut storage = storage_with(&[(
            "notes/note.first.md",
            &record_file(
                "note.first",
                "note",
                "active",
                &["tags: parser, grammar"],
                "",
            ),
        )]);
        let mut draft = decision_draft(&["parser", "grammar"]);
        draft.body = "See note.first.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn candidates_arrive_oldest_first() {
        let older = record_file(
            "decision.z-old",
            "decision",
            "active",
            &["tags: parser, grammar"],
            "",
        )
        .replace("created: 2026-08-24", "created: 2026-08-20");
        let mut storage = storage_with(&[
            ("decisions/decision.z-old.md", &older),
            (
                "decisions/decision.a-new.md",
                &record_file(
                    "decision.a-new",
                    "decision",
                    "active",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
        ]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser", "grammar"]), TODAY)
            .unwrap();
        let named: Vec<&str> = created
            .may_conflict
            .iter()
            .map(|cited| cited.id.as_str())
            .collect();
        assert_eq!(named, vec!["decision.z-old", "decision.a-new"]);
    }

    #[test]
    fn candidates_laid_down_the_same_day_order_by_id() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.second.md",
                &record_file(
                    "decision.second",
                    "decision",
                    "active",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
            (
                "decisions/decision.first.md",
                &record_file(
                    "decision.first",
                    "decision",
                    "active",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
        ]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser", "grammar"]), TODAY)
            .unwrap();
        let named: Vec<&str> = created
            .may_conflict
            .iter()
            .map(|cited| cited.id.as_str())
            .collect();
        assert_eq!(named, vec!["decision.first", "decision.second"]);
    }
}

mod mention_nudge {
    use super::*;

    #[test]
    fn a_body_citing_no_record_warns_in_the_reply_and_still_lands() {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Task, "A demo record");
        draft.body = "Blocked by task.ghost until the spike lands.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.dangling_mentions, vec!["task.ghost"]);
        assert!(
            storage.read(&created.path).is_ok(),
            "a forward reference is legal, so the record is written"
        );
    }

    #[test]
    fn a_quoted_unknown_id_warns_nothing() {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Task, "A demo record");
        draft.body = "The corpus case `task.ghost` is prose.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.dangling_mentions, Vec::<String>::new());
    }

    #[test]
    fn a_citation_that_resolves_live_or_archived_warns_nothing() {
        let mut storage = storage_with(&[
            ("tasks/task.live.md", &task_file("open", &[])),
            (
                "archive/tasks/task.done.md",
                &record_file("task.done", "task", "closed", &[], ""),
            ),
        ]);
        let mut draft = Draft::new(RecordType::Task, "A demo record");
        draft.body = "Follows task.live and task.done.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.dangling_mentions, Vec::<String>::new());
    }

    #[test]
    fn a_body_citing_the_record_it_creates_warns_nothing() {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Task, "A demo record");
        draft.id = Some("task.selfware".to_owned());
        draft.body = "task.selfware tracks its own scope.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.dangling_mentions, Vec::<String>::new());
    }

    #[test]
    fn a_comment_citing_no_record_warns_in_the_reply_and_still_logs() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let reply = Notebook::new(&mut storage)
            .comment("task.demo", None, "waits on task.ghost", TODAY)
            .unwrap();
        assert_eq!(reply.dangling_mentions, vec!["task.ghost"]);
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .contains("waits on task.ghost"),
            "a warning is not a rejection"
        );
    }

    #[test]
    fn a_quoted_id_in_a_comment_warns_nothing() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let reply = Notebook::new(&mut storage)
            .comment("task.demo", None, "renamed the `task.ghost` case", TODAY)
            .unwrap();
        assert_eq!(reply.dangling_mentions, Vec::<String>::new());
    }

    #[test]
    fn a_replayed_comment_carries_the_same_nudge() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        Notebook::new(&mut storage)
            .comment("task.demo", None, "waits on task.ghost", TODAY)
            .unwrap();
        let replay = Notebook::new(&mut storage)
            .comment("task.demo", None, "waits on task.ghost", TODAY)
            .unwrap();
        assert!(replay.already);
        assert_eq!(
            replay.dangling_mentions,
            vec!["task.ghost"],
            "the entry stands as the trail's tail, so its citations stand too"
        );
    }

    #[test]
    fn a_drop_reason_citing_no_record_warns_in_the_reply() {
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
            &record_file("question.demo", "question", "open", &[], ""),
        )]);
        let dropped = Notebook::new(&mut storage)
            .drop_question("question.demo", "absorbed into task.ghost", TODAY)
            .unwrap();
        assert_eq!(dropped.dangling_mentions, vec!["task.ghost"]);
    }

    #[test]
    fn a_replayed_drop_carries_no_nudge_for_a_reason_that_never_landed() {
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
            &record_file("question.demo", "question", "open", &[], ""),
        )]);
        Notebook::new(&mut storage)
            .drop_question("question.demo", "a plain reason", TODAY)
            .unwrap();
        let replay = Notebook::new(&mut storage)
            .drop_question("question.demo", "absorbed into task.ghost", TODAY)
            .unwrap();
        assert!(replay.transition.already);
        assert_eq!(
            replay.dangling_mentions,
            Vec::<String>::new(),
            "the replay's reason wrote nothing, so it cites nothing"
        );
    }
}

mod check {
    use super::*;

    fn findings_for(storage: &mut MemoryStorage) -> Vec<(String, FindingCode)> {
        Notebook::new(storage)
            .check()
            .unwrap()
            .into_iter()
            .map(|located| (located.path, located.finding.code))
            .collect()
    }

    #[test]
    fn a_clean_notebook_checks_empty() {
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("open", &[])),
            (
                "notes/note.demo.md",
                &record_file("note.demo", "note", "active", &[], ""),
            ),
        ]);
        assert_eq!(findings_for(&mut storage), vec![]);
    }

    #[test]
    fn errors_lead_the_report_however_late_their_file_sorts() {
        // The reply prints a bounded head, and the settling verbs leave
        // warnings behind by the dozen: on file order alone an error in a
        // late-sorting file would fall off the end of a real notebook.
        let mut storage = storage_with(&[
            (
                "tasks/task.aaa.md",
                &record_file("task.aaa", "task", "closed", &[], ""),
            ),
            (
                "tasks/task.zzz.md",
                &record_file("task.zzz", "task", "open", &["blocked-by: task.nobody"], ""),
            ),
        ]);
        assert_eq!(
            findings_for(&mut storage),
            vec![
                ("tasks/task.zzz.md".to_owned(), FindingCode::DanglingRef),
                (
                    "tasks/task.aaa.md".to_owned(),
                    FindingCode::UnarchivedSettledRecord
                ),
            ]
        );
    }

    #[test]
    fn a_task_stranded_in_the_archive_is_named_against_its_own_file() {
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("open", &[])),
            (
                "archive/tasks/task.stranded.md",
                &record_file("task.stranded", "task", "open", &[], ""),
            ),
        ]);
        assert_eq!(
            findings_for(&mut storage),
            vec![(
                "archive/tasks/task.stranded.md".to_owned(),
                FindingCode::ArchivedLiveRecord
            )],
            "the finding lands on the stranded file, not on the live record that is fine"
        );
    }

    #[test]
    fn two_files_claiming_one_id_are_both_named() {
        // The shape an archive move leaves when it dies between its write
        // and its remove: identical settled bytes in both homes.
        let text = task_file("closed", &[]);
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &text),
            ("archive/tasks/task.demo.md", &text),
        ]);
        assert_eq!(
            findings_for(&mut storage),
            vec![
                (
                    "archive/tasks/task.demo.md".to_owned(),
                    FindingCode::DuplicateId
                ),
                ("tasks/task.demo.md".to_owned(), FindingCode::DuplicateId),
                (
                    "tasks/task.demo.md".to_owned(),
                    FindingCode::UnarchivedSettledRecord
                ),
            ]
        );
    }

    #[test]
    fn a_reference_into_nothing_is_a_dangling_ref_at_its_line() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &task_file("open", &["from: question.never-written"]),
        )]);
        let located = Notebook::new(&mut storage).check().unwrap();
        assert_eq!(located.len(), 1);
        assert_eq!(located[0].finding.code, FindingCode::DanglingRef);
        assert_eq!(located[0].finding.line, Some(6));
    }

    #[test]
    fn a_supersession_without_its_back_pointer_names_both_files() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.new.md",
                &record_file(
                    "decision.new",
                    "decision",
                    "active",
                    &["supersedes: decision.old"],
                    "",
                ),
            ),
            (
                "archive/decisions/decision.old.md",
                &record_file("decision.old", "decision", "retired", &[], ""),
            ),
        ]);
        assert_eq!(
            findings_for(&mut storage),
            vec![
                (
                    "archive/decisions/decision.old.md".to_owned(),
                    FindingCode::BrokenSupersession
                ),
                (
                    "decisions/decision.new.md".to_owned(),
                    FindingCode::BrokenSupersession
                ),
            ]
        );
    }

    #[test]
    fn a_record_marked_superseded_that_still_reads_live_is_broken() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.old.md",
                &record_file(
                    "decision.old",
                    "decision",
                    "active",
                    &["superseded-by: decision.new"],
                    "",
                ),
            ),
            (
                "decisions/decision.new.md",
                &record_file(
                    "decision.new",
                    "decision",
                    "active",
                    &["supersedes: decision.old"],
                    "",
                ),
            ),
        ]);
        assert_eq!(
            findings_for(&mut storage),
            vec![(
                "decisions/decision.old.md".to_owned(),
                FindingCode::BrokenSupersession
            )],
            "the pair is coherent; the victim's live state alone is the defect"
        );
    }

    #[test]
    fn a_hand_edited_cycle_is_named_on_every_member_at_its_edge_line() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &record_file("task.a", "task", "open", &["blocked-by: task.b"], ""),
            ),
            (
                "tasks/task.b.md",
                &record_file("task.b", "task", "open", &["blocked-by: task.a"], ""),
            ),
        ]);
        let located = Notebook::new(&mut storage).check().unwrap();
        assert_eq!(
            located
                .iter()
                .map(|found| (found.path.as_str(), found.finding.code, found.finding.line))
                .collect::<Vec<_>>(),
            vec![
                ("tasks/task.a.md", FindingCode::DepCycle, Some(6)),
                ("tasks/task.b.md", FindingCode::DepCycle, Some(6)),
            ]
        );
        assert!(
            located[0]
                .finding
                .message
                .contains("task.a → task.b → task.a"),
            "the message walks the whole cycle: {}",
            located[0].finding.message
        );
    }

    #[test]
    fn a_cycle_through_a_third_task_names_all_three_members() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &record_file("task.a", "task", "open", &["blocked-by: task.b"], ""),
            ),
            (
                "tasks/task.b.md",
                &record_file("task.b", "task", "open", &["blocked-by: task.c"], ""),
            ),
            (
                "tasks/task.c.md",
                &record_file("task.c", "task", "open", &["blocked-by: task.a"], ""),
            ),
        ]);
        assert_eq!(
            findings_for(&mut storage),
            vec![
                ("tasks/task.a.md".to_owned(), FindingCode::DepCycle),
                ("tasks/task.b.md".to_owned(), FindingCode::DepCycle),
                ("tasks/task.c.md".to_owned(), FindingCode::DepCycle),
            ]
        );
    }

    #[test]
    fn two_disjoint_cycles_are_both_named() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &record_file("task.a", "task", "open", &["blocked-by: task.b"], ""),
            ),
            (
                "tasks/task.b.md",
                &record_file("task.b", "task", "open", &["blocked-by: task.a"], ""),
            ),
            (
                "tasks/task.c.md",
                &record_file("task.c", "task", "open", &["blocked-by: task.d"], ""),
            ),
            (
                "tasks/task.d.md",
                &record_file("task.d", "task", "open", &["blocked-by: task.c"], ""),
            ),
        ]);
        let named: Vec<String> = Notebook::new(&mut storage)
            .check()
            .unwrap()
            .into_iter()
            .map(|located| located.path)
            .collect();
        assert_eq!(
            named,
            vec![
                "tasks/task.a.md",
                "tasks/task.b.md",
                "tasks/task.c.md",
                "tasks/task.d.md"
            ]
        );
    }

    #[test]
    fn repairing_a_named_cycle_surfaces_the_one_overlapping_it() {
        let entangled = [
            (
                "tasks/task.a.md",
                record_file(
                    "task.a",
                    "task",
                    "open",
                    &["blocked-by: task.b", "blocked-by: task.c"],
                    "",
                ),
            ),
            (
                "tasks/task.b.md",
                record_file("task.b", "task", "open", &["blocked-by: task.c"], ""),
            ),
            (
                "tasks/task.c.md",
                record_file("task.c", "task", "open", &["blocked-by: task.a"], ""),
            ),
        ];
        let mut storage = storage_with(
            &entangled
                .iter()
                .map(|(path, text)| (*path, text.as_str()))
                .collect::<Vec<_>>(),
        );
        let first_pass: Vec<FindingCode> = Notebook::new(&mut storage)
            .check()
            .unwrap()
            .into_iter()
            .map(|located| located.finding.code)
            .collect();
        assert_eq!(
            first_pass,
            vec![
                FindingCode::DepCycle,
                FindingCode::DepCycle,
                FindingCode::DepCycle
            ],
            "one cycle per back edge: the walk names task.a → task.b → task.c → task.a"
        );

        Notebook::new(&mut storage)
            .unblock("task.a", "task.b", TODAY)
            .unwrap();
        assert_eq!(
            findings_for(&mut storage),
            vec![
                ("tasks/task.a.md".to_owned(), FindingCode::DepCycle),
                ("tasks/task.c.md".to_owned(), FindingCode::DepCycle),
            ],
            "the cycle hidden behind the repaired one surfaces on the next walk"
        );
    }

    #[test]
    fn a_task_waiting_on_itself_is_a_dep_cycle_at_its_own_line() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &task_file("open", &["blocked-by: task.demo"]),
        )]);
        let located = Notebook::new(&mut storage).check().unwrap();
        assert_eq!(located.len(), 1);
        assert_eq!(located[0].finding.code, FindingCode::DepCycle);
        assert_eq!(located[0].finding.line, Some(6));
    }

    #[test]
    fn a_dependency_edge_into_a_non_task_is_a_bad_value() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &task_file("open", &["blocked-by: note.a-fact"]),
            ),
            (
                "notes/note.a-fact.md",
                &record_file("note.a-fact", "note", "active", &[], ""),
            ),
        ]);
        assert_eq!(
            findings_for(&mut storage),
            vec![("tasks/task.demo.md".to_owned(), FindingCode::BadValue)]
        );
    }

    #[test]
    fn a_routed_question_pointing_at_nothing_has_lost_its_thread() {
        let mut storage = storage_with(&[(
            "archive/questions/question.demo.md",
            &record_file(
                "question.demo",
                "question",
                "routed",
                &["routed-to: decision.never-written"],
                "",
            ),
        )]);
        assert_eq!(
            findings_for(&mut storage),
            vec![(
                "archive/questions/question.demo.md".to_owned(),
                FindingCode::BrokenRouting
            )]
        );
    }
}

mod status_dashboard {
    use super::*;

    fn status_text(storage: &mut MemoryStorage) -> String {
        Notebook::new(storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap()
            .text
    }

    #[test]
    fn a_quiet_notebook_answers_one_line_with_counts() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &task_file("closed", &["closed: 2026-08-25"]),
            ),
            (
                "notes/note.demo.md",
                &record_file("note.demo", "note", "retired", &[], ""),
            ),
        ]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert!(status.quiet);
        assert_eq!(status.text.lines().count(), 1);
        assert!(
            status
                .text
                .starts_with("ok: notebook quiet — 1 tasks, 0 decisions, 1 notes, 0 questions."),
            "unexpected quiet line: {}",
            status.text
        );
        assert!(status.text.contains("anb --help"));
    }

    #[test]
    fn an_active_task_is_the_in_flight_line_with_its_last_log_line() {
        let body = "Acceptance: the ladder holds.\n\n- 2026-08-25 claude: stopped at the ladder\n";
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &record_file("task.demo", "task", "active", &[], body),
        )]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("in-flight: task.demo \"A demo record\"\n"),
            "{text}"
        );
        assert!(
            text.contains("log: - 2026-08-25 claude: stopped at the ladder\n"),
            "{text}"
        );
    }

    #[test]
    fn the_dashboard_opens_on_ready_work_alone() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert!(!status.quiet);
        assert!(
            status.text.contains("ready[1]{id,priority,age,title}:\n"),
            "{}",
            status.text
        );
    }

    #[test]
    fn the_dashboard_opens_on_debt_alone() {
        // A routed question is settled; the open one aged past fourteen days.
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
            "---\nid: question.demo\ntype: question\nstate: open\ntitle: A demo record\ncreated: 2026-08-01\nupdated: 2026-08-01\n---\n",
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert!(!status.quiet);
        assert!(
            status.text.contains("question-age: question.demo (26d)"),
            "{}",
            status.text
        );
    }

    #[test]
    fn rules_alone_do_not_open_the_gate() {
        let mut storage = storage_with(&[(
            "decisions/decision.demo.md",
            &record_file("decision.demo", "decision", "active", &["kind: rule"], ""),
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert!(
            status.quiet,
            "a standing rule is not work in motion: {}",
            status.text
        );
    }

    #[test]
    fn a_ready_row_carries_priority_age_and_title() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            "---\nid: task.demo\ntype: task\nstate: open\ntitle: A demo record\npriority: 1\ncreated: 2026-08-24\n---\n",
        )]);
        let text = status_text(&mut storage);
        assert!(text.contains("  task.demo,1,3d,A demo record\n"), "{text}");
    }

    #[test]
    fn an_unprioritized_ready_row_shows_a_dash() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            "---\nid: task.demo\ntype: task\nstate: open\ntitle: A demo record\ncreated: 2026-08-27\n---\n",
        )]);
        let text = status_text(&mut storage);
        assert!(text.contains("  task.demo,-,0d,A demo record\n"), "{text}");
    }

    #[test]
    fn a_ready_title_with_a_comma_is_json_quoted() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            "---\nid: task.demo\ntype: task\nstate: open\ntitle: Sort, then trim\ncreated: 2026-08-27\n---\n",
        )]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("  task.demo,-,0d,\"Sort, then trim\"\n"),
            "{text}"
        );
    }

    #[test]
    fn more_ready_than_five_rows_shows_five_and_the_shorter_hint() {
        let files: Vec<(String, String)> = (0..7)
            .map(|index| {
                (
                    format!("tasks/task.t{index}.md"),
                    format!(
                        "---\nid: task.t{index}\ntype: task\nstate: open\ntitle: A demo record\ncreated: 2026-08-2{}\n---\n",
                        index % 8
                    ),
                )
            })
            .collect();
        let mut storage = MemoryStorage::from_files(files);
        let text = status_text(&mut storage);
        assert!(
            text.contains("ready[5]{id,priority,age,title}:\n"),
            "{text}"
        );
        assert_eq!(text.matches("\n  task.").count(), 5, "{text}");
        assert!(text.contains("  … 2 more: anb ready\n"), "{text}");
    }

    #[test]
    fn standing_rules_list_live_rule_decisions_only() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.rule.md",
                &record_file("decision.rule", "decision", "active", &["kind: rule"], ""),
            ),
            (
                "decisions/decision.shape.md",
                &record_file("decision.shape", "decision", "active", &["kind: shape"], ""),
            ),
            (
                "decisions/decision.dead.md",
                &record_file("decision.dead", "decision", "retired", &["kind: rule"], ""),
            ),
            ("tasks/task.demo.md", &task_file("open", &[])),
        ]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("rules[1]:\n  decision.rule: A demo record\n"),
            "{text}"
        );
        assert!(!text.contains("decision.shape"), "{text}");
        assert!(!text.contains("decision.dead"), "{text}");
    }

    #[test]
    fn a_review_task_is_named_waiting_on_a_human() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("review", &[]))]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("review[1]: task.demo — waiting on a human\n"),
            "{text}"
        );
    }

    #[test]
    fn an_in_flight_title_with_a_quote_is_escaped() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            "---\nid: task.demo\ntype: task\nstate: active\ntitle: Fix the \"quiet\" line\ncreated: 2026-08-24\n---\n",
        )]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("in-flight: task.demo \"Fix the \\\"quiet\\\" line\"\n"),
            "{text}"
        );
    }

    #[test]
    fn counts_span_live_records_of_every_type_and_skip_the_archive() {
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("open", &[])),
            (
                "decisions/decision.demo.md",
                &record_file("decision.demo", "decision", "active", &[], ""),
            ),
            (
                "notes/note.demo.md",
                &record_file("note.demo", "note", "active", &[], ""),
            ),
            (
                "questions/question.demo.md",
                &record_file("question.demo", "question", "open", &[], ""),
            ),
            (
                "archive/tasks/task.done.md",
                &record_file("task.done", "task", "closed", &[], ""),
            ),
        ]);
        let text = status_text(&mut storage);
        assert!(
            text.starts_with("ok: notebook — 1 tasks, 1 decisions, 1 notes, 1 questions\n"),
            "{text}"
        );
    }
}

mod debt_signals {
    use super::*;

    /// A record whose clock is under the test's control: `updated` is the
    /// override this module is about.
    fn aged(id: &str, type_word: &str, state: &str, updated: &str, extra: &[&str]) -> String {
        let mut lines = extra.join("\n");
        if !lines.is_empty() {
            lines.push('\n');
        }
        format!(
            "---\nid: {id}\ntype: {type_word}\nstate: {state}\ntitle: A demo record\n{lines}created: 2026-08-01\nupdated: {updated}\n---\n"
        )
    }

    fn debt_of(storage: &mut MemoryStorage) -> Vec<DebtSignal> {
        Notebook::new(storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap()
            .debt
    }

    #[test]
    fn an_active_task_untouched_for_seven_days_is_stale() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "active", "2026-08-20", &[]),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::TaskStale {
                id: "task.demo".into(),
                days: 7
            }]
        );
    }

    #[test]
    fn six_quiet_days_are_not_yet_stale() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "active", "2026-08-21", &[]),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_held_task_gone_quiet_is_hold_quiet_not_task_stale() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged(
                "task.demo",
                "task",
                "active",
                "2026-08-13",
                &["hold: waiting on the owner"],
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::HoldQuiet {
                id: "task.demo".into(),
                days: 14
            }]
        );
    }

    #[test]
    fn a_fresh_hold_is_not_debt_even_on_a_stale_clock_threshold() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged(
                "task.demo",
                "task",
                "active",
                "2026-08-20",
                &["hold: waiting on the owner"],
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_free_standing_question_ages_at_fourteen_days() {
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
            &aged("question.demo", "question", "open", "2026-08-13", &[]),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::QuestionAge {
                id: "question.demo".into(),
                days: 14
            }]
        );
    }

    #[test]
    fn a_task_born_question_ages_faster_than_a_free_standing_one() {
        let mut storage = storage_with(&[
            (
                "questions/question.parked.md",
                &aged(
                    "question.parked",
                    "question",
                    "open",
                    "2026-08-20",
                    &["from: task.origin"],
                ),
            ),
            (
                "questions/question.free.md",
                &aged("question.free", "question", "open", "2026-08-20", &[]),
            ),
            (
                "tasks/task.origin.md",
                &record_file("task.origin", "task", "active", &[], ""),
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::QuestionAge {
                id: "question.parked".into(),
                days: 7
            }]
        );
    }

    #[test]
    fn a_question_surfaces_the_moment_its_origin_task_closes() {
        let mut storage = storage_with(&[
            (
                "questions/question.parked.md",
                &aged(
                    "question.parked",
                    "question",
                    "open",
                    TODAY,
                    &["from: task.origin"],
                ),
            ),
            (
                "tasks/task.origin.md",
                &record_file("task.origin", "task", "closed", &["closed: 2026-08-26"], ""),
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::OriginClosed {
                id: "question.parked".into(),
                origin: "task.origin".into()
            }]
        );
    }

    #[test]
    fn a_review_task_waiting_seven_days_surfaces() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "review", "2026-08-20", &[]),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::ReviewWait {
                id: "task.demo".into(),
                days: 7
            }]
        );
    }

    #[test]
    fn a_review_by_date_surfaces_on_its_own_day_on_any_record() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &aged(
                "note.demo",
                "note",
                "active",
                TODAY,
                &[&format!("review-by: {TODAY}")],
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::ReviewDue {
                id: "note.demo".into(),
                date: TODAY.into()
            }]
        );
    }

    #[test]
    fn a_future_review_by_stays_silent() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &aged(
                "note.demo",
                "note",
                "active",
                TODAY,
                &["review-by: 2026-09-15"],
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_dangling_mention_names_the_citer_and_the_target() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "The fix waits on task.gone.\n",
            ),
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert_eq!(
            status.debt,
            vec![DebtSignal::DanglingMention {
                id: "note.demo".into(),
                target: "task.gone".into()
            }]
        );
        assert!(
            status
                .text
                .contains("  dangling-mention: note.demo -> task.gone\n"),
            "{}",
            status.text
        );
    }

    #[test]
    fn a_mention_of_an_existing_record_is_not_debt() {
        let mut storage = storage_with(&[
            (
                "notes/note.demo.md",
                &record_file(
                    "note.demo",
                    "note",
                    "active",
                    &[],
                    "See task.demo for the plan.\n",
                ),
            ),
            (
                "tasks/task.demo.md",
                &task_file("closed", &["closed: 2026-08-26"]),
            ),
        ]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_compound_word_does_not_cite() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "The subtask.gone helper and task.goneBar are prose, not citations.\n",
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_sentence_final_mention_cites_without_its_dot() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "Blocked by task.gone.\n",
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::DanglingMention {
                id: "note.demo".into(),
                target: "task.gone".into()
            }]
        );
    }

    #[test]
    fn a_repeated_mention_is_one_line() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "task.gone above, task.gone again.\n",
            ),
        )]);
        assert_eq!(debt_of(&mut storage).len(), 1);
    }

    #[test]
    fn two_live_decisions_citing_without_an_edge_are_one_pair_with_both_authors() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.a.md",
                &record_file(
                    "decision.a",
                    "decision",
                    "active",
                    &["by: supolka"],
                    "This tightens decision.b without replacing it.\n",
                ),
            ),
            (
                "decisions/decision.b.md",
                &record_file(
                    "decision.b",
                    "decision",
                    "active",
                    &["by: supolka", "via: claude-code"],
                    "And decision.a is the counterpart.\n",
                ),
            ),
        ]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        let pairs: Vec<&DebtSignal> = status
            .debt
            .iter()
            .filter(|signal| matches!(signal, DebtSignal::UndeclaredPair { .. }))
            .collect();
        assert_eq!(pairs.len(), 1, "both directions of citation are one pair");
        assert!(
            status.text.contains(
                "  undeclared-pair: decision.a (supolka) <-> decision.b (supolka/claude-code)\n"
            ),
            "{}",
            status.text
        );
    }

    #[test]
    fn a_declared_link_edge_silences_the_pair() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.a.md",
                &record_file(
                    "decision.a",
                    "decision",
                    "active",
                    &["link: see decision.b"],
                    "This tightens decision.b without replacing it.\n",
                ),
            ),
            (
                "decisions/decision.b.md",
                &record_file("decision.b", "decision", "active", &[], ""),
            ),
        ]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn an_invalid_record_is_a_debt_line_with_its_error_count_and_no_clock() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "wandering", "2026-08-01", &[]),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::Invalid {
                path: "tasks/task.demo.md".into(),
                errors: 1
            }]
        );
    }

    #[test]
    fn a_review_task_six_days_in_is_not_yet_debt() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "review", "2026-08-21", &[]),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn origin_closed_outranks_the_age_clock_when_both_would_fire() {
        let mut storage = storage_with(&[
            (
                "questions/question.parked.md",
                &aged(
                    "question.parked",
                    "question",
                    "open",
                    "2026-08-10",
                    &["from: task.origin"],
                ),
            ),
            (
                "tasks/task.origin.md",
                &record_file("task.origin", "task", "closed", &["closed: 2026-08-26"], ""),
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::OriginClosed {
                id: "question.parked".into(),
                origin: "task.origin".into()
            }]
        );
    }

    #[test]
    fn a_closed_task_still_carrying_a_hold_line_does_not_tick() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged(
                "task.demo",
                "task",
                "closed",
                "2026-08-01",
                &["hold: parked before the close", "closed: 2026-08-02"],
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_dead_lifecycle_state_silences_review_by() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.demo.md",
                &aged(
                    "decision.demo",
                    "decision",
                    "superseded",
                    "2026-08-01",
                    &["superseded-by: decision.next", "review-by: 2026-08-10"],
                ),
            ),
            (
                "decisions/decision.next.md",
                &record_file(
                    "decision.next",
                    "decision",
                    "active",
                    &["supersedes: decision.demo"],
                    "",
                ),
            ),
        ]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_record_in_the_wrong_directory_resolves_nothing() {
        // The referenced file exists only under `tasks/`; as a `note.*` id it
        // resolves nowhere, so the referrer is excluded from the clocks and
        // both files are listed invalid.
        let mut storage = storage_with(&[
            (
                "tasks/note.helper.md",
                &record_file("note.helper", "note", "active", &[], ""),
            ),
            (
                "tasks/task.demo.md",
                &aged(
                    "task.demo",
                    "task",
                    "active",
                    "2026-08-10",
                    &["from: note.helper"],
                ),
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![
                DebtSignal::Invalid {
                    path: "tasks/note.helper.md".into(),
                    errors: 1
                },
                DebtSignal::Invalid {
                    path: "tasks/task.demo.md".into(),
                    errors: 1
                },
            ]
        );
    }

    #[test]
    fn an_invalid_counterpart_cannot_enter_an_undeclared_pair() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.a.md",
                &record_file(
                    "decision.a",
                    "decision",
                    "active",
                    &[],
                    "This tightens decision.b without replacing it.\n",
                ),
            ),
            (
                "decisions/decision.b.md",
                "---\nid: decision.b\ntype: decision\nstate: active\ntitle: A demo record\ncreated: 2026-8-4\n---\n",
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::Invalid {
                path: "decisions/decision.b.md".into(),
                errors: 1
            }]
        );
    }

    #[test]
    fn pairs_rank_by_the_older_members_created_date() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.young.md",
                "---\nid: decision.young\ntype: decision\nstate: active\ntitle: A demo record\ncreated: 2026-08-20\n---\n\nSee decision.newish.\n",
            ),
            (
                "decisions/decision.newish.md",
                "---\nid: decision.newish\ntype: decision\nstate: active\ntitle: A demo record\ncreated: 2026-08-18\n---\n",
            ),
            (
                "decisions/decision.old.md",
                "---\nid: decision.old\ntype: decision\nstate: active\ntitle: A demo record\ncreated: 2026-08-02\n---\n\nSee decision.older.\n",
            ),
            (
                "decisions/decision.older.md",
                "---\nid: decision.older\ntype: decision\nstate: active\ntitle: A demo record\ncreated: 2026-08-01\n---\n",
            ),
        ]);
        let pair_firsts: Vec<String> = debt_of(&mut storage)
            .iter()
            .filter_map(|signal| match signal {
                DebtSignal::UndeclaredPair { first, .. } => Some(first.id.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            pair_firsts,
            vec!["decision.old".to_owned(), "decision.newish".to_owned()]
        );
    }

    #[test]
    fn a_config_stale_threshold_moves_the_task_clock() {
        let mut storage = storage_with(&[
            ("config", "debt-task-stale: 2\n"),
            (
                "tasks/task.demo.md",
                &aged("task.demo", "task", "active", "2026-08-25", &[]),
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::TaskStale {
                id: "task.demo".into(),
                days: 2
            }]
        );
    }

    #[test]
    fn a_trailing_hyphen_falls_off_a_mention() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "See task.gone- for why.\n",
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::DanglingMention {
                id: "note.demo".into(),
                target: "task.gone".into()
            }]
        );
    }

    #[test]
    fn a_dotted_chain_cites_up_to_its_first_stop() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "See task.foo.bar here.\n",
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::DanglingMention {
                id: "note.demo".into(),
                target: "task.foo".into()
            }]
        );
    }

    #[test]
    fn an_underscore_prefix_blocks_a_mention() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "The _task.gone symbol is code.\n",
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn an_id_past_sixty_four_bytes_is_not_a_mention() {
        let long_slug = "a".repeat(70);
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                &format!("See task.{long_slug} maybe.\n"),
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_backticked_id_is_a_quotation_beside_a_citing_bare_id() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "The corpus case `task.quoted` waits on task.gone.\n",
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::DanglingMention {
                id: "note.demo".into(),
                target: "task.gone".into()
            }],
            "the quoted id creates no edge; the bare one still cites"
        );
    }

    #[test]
    fn an_id_inside_a_code_fence_is_a_quotation_too() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "```\nanb view task.gone\n```\n",
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_stray_backtick_on_an_earlier_line_cannot_unquote_a_later_entry() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "- 2026-08-26 -: the ` character is special\n- 2026-08-27 -: renamed the `task.ghost` case\n",
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![],
            "span backticks pair within one line, never across entries"
        );
    }

    #[test]
    fn a_fence_closes_on_a_longer_run_too() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "```\nanb view task.gone\n````\n",
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn an_unclosed_fence_quotes_to_the_end() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "```\nanb view task.gone\n",
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn multi_byte_neighbors_leave_the_scan_intact() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "A café note — task.gone cites, «`task.quoted`» does not.\n",
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::DanglingMention {
                id: "note.demo".into(),
                target: "task.gone".into()
            }]
        );
    }

    #[test]
    fn an_unpaired_backtick_leaves_what_follows_citing() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "A stray ` backtick, then task.gone.\n",
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::DanglingMention {
                id: "note.demo".into(),
                target: "task.gone".into()
            }]
        );
    }

    #[test]
    fn a_code_span_closes_only_on_a_run_of_its_own_length() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "``a `task.inner` chain`` beside task.gone.\n",
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::DanglingMention {
                id: "note.demo".into(),
                target: "task.gone".into()
            }],
            "the single-backtick run is content of the double-run span"
        );
    }

    #[test]
    fn six_dangling_mentions_render_five_lines_and_a_hint() {
        let body = "task.gone0 task.gone1 task.gone2 task.gone3 task.gone4 task.gone5";
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file("note.demo", "note", "active", &[], &format!("{body}\n")),
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert_eq!(status.debt.len(), 6, "the model keeps every signal");
        assert_eq!(
            status.text.matches("dangling-mention:").count(),
            5,
            "{}",
            status.text
        );
        assert!(status.text.contains("  … 1 more\n"), "{}", status.text);
    }

    #[test]
    fn the_proofs_offered_for_the_world_to_settle_are_the_live_records_claims() {
        let mut storage = storage_with(&[
            (
                "tasks/task.shipped.md",
                &record_file(
                    "task.shipped",
                    "task",
                    "closed",
                    &[
                        "link: sha f00dfeed",
                        "link: report notes/report.md",
                        "link: pr https://example.com/pull/7",
                        "link: note note.elsewhere",
                    ],
                    "",
                ),
            ),
            (
                "notes/note.elsewhere.md",
                &record_file("note.elsewhere", "note", "active", &[], ""),
            ),
            (
                "archive/tasks/task.filed.md",
                &record_file("task.filed", "task", "closed", &["link: sha deadbeef"], ""),
            ),
        ]);
        let mut offered = Vec::new();
        Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, |cited| {
                offered = cited.to_vec();
                Vec::new()
            })
            .unwrap();
        assert_eq!(
            offered,
            vec![
                CitedProof {
                    record: "task.shipped".to_owned(),
                    kind: "sha".to_owned(),
                    target: "f00dfeed".to_owned(),
                },
                CitedProof {
                    record: "task.shipped".to_owned(),
                    kind: "report".to_owned(),
                    target: "notes/report.md".to_owned(),
                },
            ],
            "a commit and a file are claims about the world; a pull request \
             is not ours to reach, a note resolves inside the notebook, and \
             an archived record's claim is history no verb can settle"
        );
    }

    #[test]
    fn a_proof_the_world_no_longer_holds_is_named_on_the_dashboard() {
        let mut storage = storage_with(&[(
            "tasks/task.shipped.md",
            &record_file(
                "task.shipped",
                "task",
                "closed",
                &["link: sha f00dfeed"],
                "",
            ),
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, <[CitedProof]>::to_vec)
            .unwrap();
        assert_eq!(
            status.debt,
            vec![DebtSignal::LostProof {
                id: "task.shipped".to_owned(),
                proof: "sha f00dfeed".to_owned(),
            }],
            "a claim the world cannot answer for is the record's while it is live"
        );
        assert!(
            status
                .text
                .contains("  lost-proof: task.shipped -> sha f00dfeed\n"),
            "{}",
            status.text
        );
    }

    #[test]
    fn a_proof_the_world_still_holds_raises_nothing() {
        let mut storage = storage_with(&[(
            "tasks/task.shipped.md",
            &record_file(
                "task.shipped",
                "task",
                "closed",
                &["link: sha f00dfeed"],
                "",
            ),
        )]);
        // The host settled this one and found it: nothing reaches the Core.
        assert!(
            Notebook::new(&mut storage)
                .status(TODAY, Budget::Unbounded, no_lost_proofs)
                .unwrap()
                .debt
                .iter()
                .all(|signal| !matches!(signal, DebtSignal::LostProof { .. }))
        );
    }

    #[test]
    fn a_record_its_own_errors_already_condemn_is_left_to_check() {
        let mut storage = storage_with(&[(
            "tasks/task.broken.md",
            &record_file(
                "task.broken",
                "task",
                "closed",
                &["from: task.never-written", "link: sha f00dfeed"],
                "",
            ),
        )]);
        let lost = vec![CitedProof {
            record: "task.broken".to_owned(),
            kind: "sha".to_owned(),
            target: "f00dfeed".to_owned(),
        }];
        assert!(
            Notebook::new(&mut storage)
                .status(TODAY, Budget::Unbounded, |_| lost.clone())
                .unwrap()
                .debt
                .iter()
                .all(|signal| !matches!(signal, DebtSignal::LostProof { .. })),
            "one broken file, one signal — the invalid line already names it"
        );
    }

    #[test]
    fn the_archive_raises_no_debt_of_its_own() {
        // A settled record, valid where it sits, carrying the one signal
        // that outlives a settled state: a body citing an id nobody holds.
        // Only residence can silence it.
        let mut storage = storage_with(&[(
            "archive/tasks/task.demo.md",
            &record_file(
                "task.demo",
                "task",
                "closed",
                &[],
                "the work task.ghost asked for\n",
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![],
            "history is over: it neither ages nor asks anything of the reader"
        );
    }

    #[test]
    fn a_corrupt_file_in_the_archive_is_checks_to_name_and_not_the_dashboards() {
        let mut storage = storage_with(&[(
            "archive/questions/question.demo.md",
            &aged("question.demo", "question", "open", "2026-06-01", &[]),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![],
            "a session start reads the live notebook, not every file history holds"
        );
        assert_eq!(
            Notebook::new(&mut storage)
                .check()
                .unwrap()
                .iter()
                .map(|located| located.path.clone())
                .collect::<Vec<String>>(),
            vec!["archive/questions/question.demo.md"],
            "nothing is lost: the surface that reads every file still names it"
        );
    }

    #[test]
    fn debt_orders_classes_by_the_clock_table_and_oldest_first_within() {
        let mut storage = storage_with(&[
            (
                "tasks/task.older.md",
                &aged("task.older", "task", "active", "2026-08-10", &[]),
            ),
            (
                "tasks/task.newer.md",
                &aged("task.newer", "task", "active", "2026-08-19", &[]),
            ),
            (
                "questions/question.demo.md",
                &aged("question.demo", "question", "open", "2026-08-01", &[]),
            ),
        ]);
        let codes_and_ids: Vec<(String, String)> = debt_of(&mut storage)
            .iter()
            .map(|signal| match signal {
                DebtSignal::TaskStale { id, .. } => ("task-stale".into(), id.clone()),
                DebtSignal::QuestionAge { id, .. } => ("question-age".into(), id.clone()),
                other => panic!("unexpected signal {other:?}"),
            })
            .collect();
        assert_eq!(
            codes_and_ids,
            vec![
                ("task-stale".to_owned(), "task.older".to_owned()),
                ("task-stale".to_owned(), "task.newer".to_owned()),
                ("question-age".to_owned(), "question.demo".to_owned()),
            ]
        );
    }
}

mod budget_ladder {
    use super::*;

    /// A notebook with every section populated: an in-flight Task with a
    /// log, review work, rules, seven ready rows, an epic, and aged debt.
    fn full_notebook() -> MemoryStorage {
        let mut files: Vec<(String, String)> = vec![
            (
                "tasks/task.epic.md".into(),
                record_file(
                    "task.epic",
                    "task",
                    "open",
                    &["blocked-by: task.flight"],
                    "",
                ),
            ),
            (
                "tasks/task.flight.md".into(),
                record_file(
                    "task.flight",
                    "task",
                    "active",
                    &["from: task.epic"],
                    "- 2026-08-25 claude: stopped at the ladder\n",
                ),
            ),
            (
                "tasks/task.waiting.md".into(),
                record_file("task.waiting", "task", "review", &[], ""),
            ),
            (
                "decisions/decision.rule.md".into(),
                record_file("decision.rule", "decision", "active", &["kind: rule"], ""),
            ),
            (
                "questions/question.aged.md".into(),
                "---\nid: question.aged\ntype: question\nstate: open\ntitle: A demo record\ncreated: 2026-08-01\nupdated: 2026-08-01\n---\n"
                    .into(),
            ),
        ];
        for index in 0..7 {
            files.push((
                format!("tasks/task.r{index}.md"),
                format!(
                    "---\nid: task.r{index}\ntype: task\nstate: open\ntitle: A demo record\ncreated: 2026-08-2{}\n---\n",
                    index % 8
                ),
            ));
        }
        MemoryStorage::from_files(files)
    }

    struct Rendered {
        text: String,
        spent: u32,
    }

    fn rendered(budget: Budget) -> Rendered {
        let mut storage = full_notebook();
        let status = Notebook::new(&mut storage)
            .status(TODAY, budget, no_lost_proofs)
            .unwrap();
        Rendered {
            text: status.text,
            spent: status.spent,
        }
    }

    /// A notebook whose every listing section overflows its bound: eight
    /// Tasks in flight, eight waiting on a human, eight standing rules, and
    /// eight epics.
    fn crowded_notebook() -> MemoryStorage {
        let mut files: Vec<(String, String)> = Vec::new();
        for index in 0..8 {
            files.push((
                format!("tasks/task.flight{index}.md"),
                record_file(&format!("task.flight{index}"), "task", "active", &[], ""),
            ));
            files.push((
                format!("tasks/task.waiting{index}.md"),
                record_file(&format!("task.waiting{index}"), "task", "review", &[], ""),
            ));
            files.push((
                format!("decisions/decision.rule{index}.md"),
                record_file(
                    &format!("decision.rule{index}"),
                    "decision",
                    "active",
                    &["kind: rule"],
                    "",
                ),
            ));
            files.push((
                format!("tasks/task.hub{index}.md"),
                record_file(
                    &format!("task.hub{index}"),
                    "task",
                    "open",
                    &[&format!("blocked-by: task.child{index}")],
                    "",
                ),
            ));
            files.push((
                format!("tasks/task.child{index}.md"),
                record_file(
                    &format!("task.child{index}"),
                    "task",
                    "open",
                    &[&format!("from: task.hub{index}")],
                    "",
                ),
            ));
        }
        MemoryStorage::from_files(files)
    }

    #[test]
    fn a_crowded_section_shows_five_rows_and_counts_the_rest() {
        let mut storage = crowded_notebook();
        let status = Notebook::new(&mut storage)
            .status(
                TODAY,
                Budget::Tokens(Budget::DEFAULT_TOKENS),
                no_lost_proofs,
            )
            .unwrap();
        assert_eq!(
            status.text,
            "ok: notebook — 32 tasks, 8 decisions, 0 notes, 0 questions\n\
             in-flight: task.flight0 \"A demo record\"\n\
             in-flight: task.flight1 \"A demo record\"\n\
             in-flight: task.flight2 \"A demo record\"\n\
             in-flight: task.flight3 \"A demo record\"\n\
             in-flight: task.flight4 \"A demo record\"\n  \u{2026} 3 more in flight\n\
             review[8]: task.waiting0, task.waiting1, task.waiting2, task.waiting3, \
             task.waiting4, \u{2026} 3 more — waiting on a human\n\
             rules[8]:\n\
             \x20 decision.rule0: A demo record\n\
             \x20 decision.rule1: A demo record\n\
             \x20 decision.rule2: A demo record\n\
             \x20 decision.rule3: A demo record\n\
             \x20 decision.rule4: A demo record\n  \u{2026} 3 more\n\
             ready[5]{id,priority,age,title}:\n\
             \x20 task.child0,-,3d,A demo record\n\
             \x20 task.child1,-,3d,A demo record\n\
             \x20 task.child2,-,3d,A demo record\n\
             \x20 task.child3,-,3d,A demo record\n\
             \x20 task.child4,-,3d,A demo record\n  \u{2026} 3 more: anb ready\n\
             epics[8]:\n\
             \x20 task.hub0: 0/1 closed, next: task.child0\n\
             \x20 task.hub1: 0/1 closed, next: task.child1\n\
             \x20 task.hub2: 0/1 closed, next: task.child2\n\
             \x20 task.hub3: 0/1 closed, next: task.child3\n\
             \x20 task.hub4: 0/1 closed, next: task.child4\n  \u{2026} 3 more\n\
             budget: ~309/1500 tokens\n",
            "every section stops at five rows and counts the rest, and a \
             notebook that could fill any of them still leaves the default \
             budget nothing to degrade"
        );
    }

    #[test]
    fn the_floor_keeps_one_in_flight_line_however_many_are_flying() {
        let mut storage = crowded_notebook();
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Tokens(1), no_lost_proofs)
            .unwrap();
        assert_eq!(
            status.text.lines().count(),
            4,
            "counts, where work stopped, what else is flying, and the budget line: {}",
            status.text
        );
        assert!(
            status.text.contains("  \u{2026} 7 more in flight\n"),
            "{}",
            status.text
        );
    }

    #[test]
    fn an_unbounded_budget_prints_every_section_and_the_no_ceiling_line() {
        let full = rendered(Budget::Unbounded);
        for section in [
            "in-flight: task.flight",
            "log: - 2026-08-25 claude: stopped at the ladder",
            "review[1]: task.waiting — waiting on a human",
            "rules[1]:",
            "ready[5]{id,priority,age,title}:",
            "  … 2 more: anb ready",
            "debt[",
        ] {
            assert!(
                full.text.contains(section),
                "missing `{section}` in: {}",
                full.text
            );
        }
        assert!(
            full.text
                .contains(&format!("budget: ~{} tokens (no ceiling)\n", full.spent))
        );
        assert!(!full.text.contains("cut:"), "{}", full.text);
    }

    #[test]
    fn a_fitting_budget_cuts_nothing_and_reports_spent_over_ceiling() {
        let comfortable = rendered(Budget::Tokens(1500));
        assert!(comfortable.spent <= 1500);
        assert!(
            comfortable
                .text
                .contains(&format!("budget: ~{}/1500 tokens\n", comfortable.spent)),
            "{}",
            comfortable.text
        );
        assert!(!comfortable.text.contains("cut:"), "{}", comfortable.text);
    }

    /// The ladder's fixed order, read off a descending budget sweep: ready
    /// rows go first, then debt collapses, then rules, then the log line —
    /// never the other way around — and the text fits every budget the
    /// floor has not been forced past.
    /// The indented rows under one section header, wherever the ladder has
    /// moved the sections around it.
    fn rows_under(text: &str, header: &str) -> usize {
        text.lines()
            .skip_while(|line| !line.starts_with(header))
            .skip(1)
            .take_while(|line| line.starts_with("  "))
            .count()
    }

    #[test]
    fn the_epic_block_states_progress_and_collapses_to_a_count() {
        let full = rendered(Budget::Unbounded);
        assert!(
            full.text.contains("epics[1]:\n  task.epic: 0/1 closed"),
            "an epic whose only child is in flight has nothing ready: {}",
            full.text
        );

        let tight = rendered(Budget::Tokens(110));
        assert!(
            tight.text.contains("epics: 1 — anb list --for <id>"),
            "collapsed, it keeps the count and names the way back: {}",
            tight.text
        );
        assert!(
            tight.text.contains("epics\u{2192}count"),
            "and the budget line says what it cut: {}",
            tight.text
        );
    }

    #[test]
    fn sections_degrade_in_the_fixed_order_as_the_budget_shrinks() {
        let mut stages = Vec::new();
        for ceiling in [
            400, 150, 130, 110, 100, 90, 80, 70, 60, 50, 40, 30, 20, 10, 1,
        ] {
            let step = rendered(Budget::Tokens(ceiling));
            let ready_rows = rows_under(&step.text, "ready[");
            let epics_itemized = step.text.contains("epics[");
            let debt_itemized = step.text.contains("debt[");
            let rules_itemized = step.text.contains("rules[");
            let has_log = step.text.contains("log: ");
            let floor = !step.text.contains("ready") && !step.text.contains("debt");

            assert!(
                epics_itemized || ready_rows == 0,
                "epics collapsed while ready rows remain at {ceiling}: {}",
                step.text
            );
            assert!(
                debt_itemized || !epics_itemized,
                "debt collapsed before epics at {ceiling}: {}",
                step.text
            );
            assert!(
                rules_itemized || !debt_itemized,
                "rules collapsed before debt at {ceiling}: {}",
                step.text
            );
            assert!(
                has_log || !rules_itemized,
                "the log dropped before rules collapsed at {ceiling}: {}",
                step.text
            );
            assert!(
                step.spent <= ceiling || floor,
                "over budget without reaching the floor at {ceiling}: ~{} tokens: {}",
                step.spent,
                step.text
            );
            if step.text.contains("cut:") {
                assert!(step.text.contains("anb status --budget 0"), "{}", step.text);
            }
            let review_collapsed = step.text.contains("review: 1");
            if review_collapsed && !floor {
                assert!(
                    step.text.contains("review\u{2192}count"),
                    "a collapsed review list must be named as cut at {ceiling}: {}",
                    step.text
                );
            }
            if !has_log && !floor {
                assert!(
                    step.text.contains("log"),
                    "a dropped log line must be named as cut at {ceiling}: {}",
                    step.text
                );
            }
            stages.push((ready_rows, debt_itemized, rules_itemized, has_log));
        }
        let mut previous = stages[0];
        for stage in stages {
            assert!(
                stage.0 <= previous.0
                    && (!stage.1 || previous.1)
                    && (!stage.2 || previous.2)
                    && (!stage.3 || previous.3),
                "a smaller budget restored a section: {stage:?} after {previous:?}"
            );
            previous = stage;
        }
    }

    /// The number on the budget line must never understate the text it sits
    /// in: it is the sum of per-part estimates, so it may exceed the whole
    /// text's estimate by the one rounding token, never fall under it.
    #[test]
    fn the_reported_spent_never_understates_the_text() {
        for ceiling in [
            Budget::Unbounded,
            Budget::Tokens(1500),
            Budget::Tokens(100),
            Budget::Tokens(1),
        ] {
            let step = {
                let mut storage = full_notebook();
                Notebook::new(&mut storage)
                    .status(TODAY, ceiling, no_lost_proofs)
                    .unwrap()
            };
            let whole = anb_core::estimate_tokens(&step.text);
            assert!(
                step.spent >= whole && step.spent <= whole + 1,
                "spent ~{} vs whole-text estimate {whole}:\n{}",
                step.spent,
                step.text
            );
        }
    }

    #[test]
    fn a_cut_note_never_names_a_log_line_that_did_not_exist() {
        // Ready work only: the gate opens with no active Task and no log.
        let files: Vec<(String, String)> = (0..6)
            .map(|index| {
                (
                    format!("tasks/task.r{index}.md"),
                    format!(
                        "---\nid: task.r{index}\ntype: task\nstate: open\ntitle: A demo record\ncreated: 2026-08-2{}\n---\n",
                        index % 8
                    ),
                )
            })
            .collect();
        for ceiling in [200, 100, 60, 40, 20, 10, 2] {
            let mut storage = MemoryStorage::from_files(files.clone());
            let status = Notebook::new(&mut storage)
                .status(TODAY, Budget::Tokens(ceiling), no_lost_proofs)
                .unwrap();
            if let Some(cut) = status.text.split("cut: ").nth(1) {
                assert!(
                    !cut.contains("log"),
                    "no log line existed to cut at {ceiling}: {}",
                    status.text
                );
            }
        }
    }

    #[test]
    fn the_floor_keeps_counts_in_flight_and_the_budget_line() {
        let floor = rendered(Budget::Tokens(1));
        let lines: Vec<&str> = floor.text.lines().collect();
        assert_eq!(lines.len(), 3, "{}", floor.text);
        assert!(lines[0].starts_with("ok: notebook — "), "{}", floor.text);
        assert!(
            lines[1].starts_with("in-flight: task.flight"),
            "{}",
            floor.text
        );
        assert!(
            lines[2].starts_with(&format!(
                "budget: ~{}/1 tokens; cut: all but the first in-flight",
                floor.spent
            )),
            "the floor ships over budget, reported honestly: {}",
            floor.text
        );
    }

    #[test]
    fn a_quiet_notebook_ignores_the_ladder() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &task_file("closed", &["closed: 2026-08-25"]),
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Tokens(1), no_lost_proofs)
            .unwrap();
        assert!(status.quiet);
        assert!(
            status.text.starts_with("ok: notebook quiet — "),
            "{}",
            status.text
        );
    }

    /// The Budget regression fixture. The 350 is a pin, not a derivation:
    /// this notebook's dashboard measured 212 true o200k tokens before the
    /// rules, review, and log sections existed (2026-08-25), and the pin
    /// grants those plus the estimator's overshoot their room. Growth past
    /// it is dashboard bloat, and a budget-line regression is a failure.
    #[test]
    fn a_sixty_task_notebook_fits_the_default_budget_with_headroom() {
        let mut files: Vec<(String, String)> = vec![(
            "tasks/task.flight.md".into(),
            record_file(
                "task.flight",
                "task",
                "active",
                &[],
                "- 2026-08-25 claude: stopped at the ladder\n",
            ),
        )];
        for index in 0..59 {
            let state = if index % 3 == 0 { "open" } else { "closed" };
            files.push((
                format!("tasks/task.t{index}.md"),
                format!(
                    "---\nid: task.t{index}\ntype: task\nstate: {state}\ntitle: Parser accepts budget handling in the fences path\npriority: {}\ncreated: 2026-08-1{}\n---\n",
                    index % 5,
                    index % 10,
                ),
            ));
        }
        for index in 0..20 {
            let kind = if index % 4 == 0 { "rule" } else { "shape" };
            files.push((
                format!("decisions/decision.d{index}.md"),
                format!(
                    "---\nid: decision.d{index}\ntype: decision\nstate: active\nkind: {kind}\ntitle: Never render the archive without a stated reason\ncreated: 2026-08-12\n---\n"
                ),
            ));
        }
        for index in 0..30 {
            files.push((
                format!("notes/note.n{index}.md"),
                format!(
                    "---\nid: note.n{index}\ntype: note\nstate: active\nkind: fact\ntitle: The renderer keeps every byte it did not touch\ncreated: 2026-08-12\n---\n"
                ),
            ));
        }
        for index in 0..15 {
            files.push((
                format!("questions/question.q{index}.md"),
                format!(
                    "---\nid: question.q{index}\ntype: question\nstate: open\ntitle: Does the ladder hold on compaction\ncreated: 2026-08-2{}\nupdated: 2026-08-2{}\n---\n",
                    index % 8,
                    index % 8,
                ),
            ));
        }
        let mut storage = MemoryStorage::from_files(files);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Tokens(1500), no_lost_proofs)
            .unwrap();
        assert!(!status.text.contains("cut:"), "{}", status.text);
        assert!(
            status.spent <= 350,
            "the dashboard grew past the pinned budget: ~{} tokens:\n{}",
            status.spent,
            status.text
        );
    }
}

mod notebook_config {
    use super::*;

    #[test]
    fn an_absent_config_is_the_default_budget() {
        let mut storage = MemoryStorage::new();
        let config = Notebook::new(&mut storage).config().unwrap();
        assert_eq!(config.budget(), Budget::Tokens(1500));
    }

    #[test]
    fn budget_zero_in_the_config_disables_the_ceiling() {
        let mut storage = storage_with(&[("config", "budget: 0\n")]);
        let config = Notebook::new(&mut storage).config().unwrap();
        assert_eq!(config.budget(), Budget::Unbounded);
    }

    #[test]
    fn a_config_budget_is_the_resolved_ceiling() {
        let mut storage = storage_with(&[("config", "budget: 400\n")]);
        let config = Notebook::new(&mut storage).config().unwrap();
        assert_eq!(config.budget(), Budget::Tokens(400));
    }

    #[test]
    fn an_unknown_config_key_is_a_warning_in_check() {
        let mut storage = storage_with(&[("config", "budgett: 400\n")]);
        let findings = Notebook::new(&mut storage).check().unwrap();
        assert!(
            findings.iter().any(|located| located.path == "config"
                && located.finding.code == FindingCode::UnknownField),
            "{findings:?}"
        );
    }

    #[test]
    fn a_malformed_value_keeps_the_default_and_check_names_it() {
        let mut storage = storage_with(&[("config", "budget: many\n")]);
        let notebook = Notebook::new(&mut storage);
        assert_eq!(notebook.config().unwrap().budget(), Budget::Tokens(1500));
        let findings = notebook.check().unwrap();
        assert!(
            findings
                .iter()
                .any(|located| located.path == "config"
                    && located.finding.code == FindingCode::BadValue),
            "{findings:?}"
        );
    }

    #[test]
    fn a_duplicate_config_key_is_named_and_the_first_value_wins() {
        let mut storage = storage_with(&[("config", "budget: 300\nbudget: 900\n")]);
        let notebook = Notebook::new(&mut storage);
        assert_eq!(notebook.config().unwrap().budget(), Budget::Tokens(300));
        let findings = notebook.check().unwrap();
        assert!(
            findings.iter().any(|located| located.path == "config"
                && located.finding.code == FindingCode::DuplicateField),
            "{findings:?}"
        );
    }

    #[test]
    fn a_line_that_is_not_a_field_is_a_bad_envelope_line() {
        let mut storage = storage_with(&[("config", "just prose\n")]);
        let findings = Notebook::new(&mut storage).check().unwrap();
        assert!(
            findings.iter().any(|located| located.path == "config"
                && located.finding.code == FindingCode::BadEnvelopeLine),
            "{findings:?}"
        );
    }

    #[test]
    fn a_config_threshold_moves_the_clock() {
        let mut storage = storage_with(&[
            ("config", "debt-question-age: 3\n"),
            (
                "questions/question.demo.md",
                "---\nid: question.demo\ntype: question\nstate: open\ntitle: A demo record\ncreated: 2026-08-24\nupdated: 2026-08-24\n---\n",
            ),
        ]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert_eq!(
            status.debt,
            vec![DebtSignal::QuestionAge {
                id: "question.demo".into(),
                days: 3
            }]
        );
    }
}

mod expunge_verb {
    use super::*;

    fn blockers_of(storage: &mut MemoryStorage, id: &str) -> Vec<Blocker> {
        match Notebook::new(storage).expunge(id).unwrap_err() {
            NotebookError::StillReferenced { blockers, .. } => blockers,
            other => panic!("expected a refusal naming the blockers, got {other:?}"),
        }
    }

    fn held_by(carrier: &str, through: &'static str) -> Blocker {
        Blocker {
            carrier: carrier.to_owned(),
            through,
        }
    }

    #[test]
    fn an_unreferenced_record_leaves_no_trace() {
        let mut storage = storage_with(&[
            (
                "notes/note.mistake.md",
                &record_file("note.mistake", "note", "active", &[], ""),
            ),
            ("tasks/task.demo.md", &task_file("open", &[])),
        ]);
        let gone = Notebook::new(&mut storage).expunge("note.mistake").unwrap();

        assert_eq!(gone.paths, vec!["notes/note.mistake.md"]);
        assert_eq!(storage.list("notes").unwrap(), Vec::<String>::new());
        assert!(
            storage.read("tasks/task.demo.md").is_ok(),
            "one record leaves; every other byte stays"
        );
    }

    #[test]
    fn every_kind_of_inbound_edge_holds_the_record_and_is_named() {
        let mut storage = storage_with(&[
            (
                "notes/note.mistake.md",
                &record_file("note.mistake", "note", "active", &[], ""),
            ),
            (
                "notes/note.heir.md",
                &record_file(
                    "note.heir",
                    "note",
                    "active",
                    &["supersedes: note.mistake"],
                    "",
                ),
            ),
            (
                "notes/note.replaced.md",
                &record_file(
                    "note.replaced",
                    "note",
                    "retired",
                    &["superseded-by: note.mistake"],
                    "",
                ),
            ),
            (
                "questions/question.routed.md",
                &record_file(
                    "question.routed",
                    "question",
                    "routed",
                    &["routed-to: note.mistake"],
                    "",
                ),
            ),
            (
                "tasks/task.born.md",
                &record_file("task.born", "task", "open", &["from: note.mistake"], ""),
            ),
            (
                "tasks/task.proved.md",
                &record_file(
                    "task.proved",
                    "task",
                    "closed",
                    &["link: note note.mistake"],
                    "",
                ),
            ),
            (
                "tasks/task.citing.md",
                &record_file(
                    "task.citing",
                    "task",
                    "open",
                    &[],
                    "grew out of note.mistake\n",
                ),
            ),
        ]);
        // Notebook order: type-major as every listing groups, then by path.
        assert_eq!(
            blockers_of(&mut storage, "note.mistake"),
            vec![
                held_by("task.born", "from"),
                held_by("task.citing", "body"),
                held_by("task.proved", "link"),
                held_by("note.heir", "supersedes"),
                held_by("note.replaced", "superseded-by"),
                held_by("question.routed", "routed-to"),
            ]
        );
    }

    #[test]
    fn a_link_that_names_no_record_holds_nothing() {
        let mut storage = storage_with(&[
            (
                "notes/note.mistake.md",
                &record_file("note.mistake", "note", "active", &[], ""),
            ),
            (
                "tasks/task.shipped.md",
                &record_file(
                    "task.shipped",
                    "task",
                    "closed",
                    &[
                        "link: report notes/note.mistake.md",
                        "link: pr note.mistake/7",
                    ],
                    "",
                ),
            ),
        ]);
        assert!(
            Notebook::new(&mut storage).expunge("note.mistake").is_ok(),
            "a path and a URL are not references, however they read"
        );
    }

    #[test]
    fn a_record_claiming_two_files_loses_both() {
        let text = record_file("note.mistake", "note", "retired", &[], "");
        let mut storage = storage_with(&[
            ("notes/note.mistake.md", &text),
            ("archive/notes/note.mistake.md", &text),
        ]);
        let gone = Notebook::new(&mut storage).expunge("note.mistake").unwrap();
        assert_eq!(
            gone.paths,
            vec!["notes/note.mistake.md", "archive/notes/note.mistake.md"],
            "an interrupted archive move leaves two files; no trace means neither"
        );
        assert!(
            Notebook::new(&mut storage).record("note.mistake").is_err(),
            "and the id resolves to nothing anywhere"
        );
    }

    #[test]
    fn a_refused_expunge_touches_nothing() {
        let files = [
            (
                "notes/note.mistake.md",
                record_file("note.mistake", "note", "active", &[], ""),
            ),
            (
                "tasks/task.citing.md",
                record_file("task.citing", "task", "open", &[], "see note.mistake\n"),
            ),
        ];
        let mut storage =
            storage_with(&files.each_ref().map(|(path, text)| (*path, text.as_str())));
        blockers_of(&mut storage, "note.mistake");
        for (path, text) in &files {
            assert_eq!(&storage.read(path).unwrap(), text);
        }
    }

    #[test]
    fn repairing_the_last_reference_opens_the_same_call() {
        let mut storage = storage_with(&[
            (
                "tasks/task.mistake.md",
                &record_file("task.mistake", "task", "open", &[], ""),
            ),
            (
                "tasks/task.waiting.md",
                &record_file(
                    "task.waiting",
                    "task",
                    "open",
                    &["blocked-by: task.mistake"],
                    "",
                ),
            ),
        ]);
        blockers_of(&mut storage, "task.mistake");
        Notebook::new(&mut storage)
            .unblock("task.waiting", "task.mistake", TODAY)
            .unwrap();
        assert!(Notebook::new(&mut storage).expunge("task.mistake").is_ok());
    }

    #[test]
    fn a_quoted_id_is_prose_about_a_record_and_holds_nothing() {
        let mut storage = storage_with(&[
            (
                "notes/note.mistake.md",
                &record_file("note.mistake", "note", "active", &[], ""),
            ),
            (
                "notes/note.talking.md",
                &record_file(
                    "note.talking",
                    "note",
                    "active",
                    &[],
                    "the id `note.mistake` is a bad name\n",
                ),
            ),
        ]);
        assert!(Notebook::new(&mut storage).expunge("note.mistake").is_ok());
    }

    #[test]
    fn a_record_citing_its_own_id_does_not_hold_itself() {
        let mut storage = storage_with(&[(
            "notes/note.mistake.md",
            &record_file(
                "note.mistake",
                "note",
                "active",
                &[],
                "note.mistake was a slip\n",
            ),
        )]);
        assert!(Notebook::new(&mut storage).expunge("note.mistake").is_ok());
    }

    #[test]
    fn an_archived_mistake_is_expunged_where_it_lies() {
        let mut storage = storage_with(&[(
            "archive/notes/note.mistake.md",
            &record_file("note.mistake", "note", "retired", &[], ""),
        )]);
        let gone = Notebook::new(&mut storage).expunge("note.mistake").unwrap();
        assert_eq!(gone.paths, vec!["archive/notes/note.mistake.md"]);
    }

    #[test]
    fn an_invalid_record_still_holds_the_id_it_names() {
        let mut storage = storage_with(&[
            (
                "notes/note.mistake.md",
                &record_file("note.mistake", "note", "active", &[], ""),
            ),
            (
                "tasks/task.broken.md",
                &record_file("task.broken", "task", "bogus", &["from: note.mistake"], ""),
            ),
        ]);
        assert_eq!(
            blockers_of(&mut storage, "note.mistake"),
            vec![held_by("task.broken", "from")],
            "a file the tool refuses to mutate is the worst place to leave a dangling reference"
        );
    }

    #[test]
    fn a_second_expunge_says_the_record_is_gone_rather_than_claiming_success() {
        let mut storage = storage_with(&[(
            "notes/note.mistake.md",
            &record_file("note.mistake", "note", "active", &[], ""),
        )]);
        Notebook::new(&mut storage).expunge("note.mistake").unwrap();
        assert_eq!(
            Notebook::new(&mut storage)
                .expunge("note.mistake")
                .unwrap_err(),
            NotebookError::UnknownId {
                id: "note.mistake".to_owned()
            }
        );
    }
}

mod archive_verb {
    use super::*;

    #[test]
    fn a_settled_task_moves_to_the_archive_byte_identical() {
        // Warnings only — BOM, a CRLF line, quirky spacing — so the move is
        // legal and byte-exactness is observable: a canonicalizing copy
        // would rewrite this file.
        let text = "\u{feff}---\nid: task.demo\ntype:  task\r\nstate: closed\ntitle: A demo record\ncreated: 2026-08-24\n---\nbody\n";
        let mut storage = storage_with(&[("tasks/task.demo.md", text)]);
        let moved = Notebook::new(&mut storage).archive("task.demo").unwrap();
        assert_eq!(moved.from, "tasks/task.demo.md");
        assert_eq!(moved.to, "archive/tasks/task.demo.md");
        assert!(!moved.already);
        assert_eq!(
            storage.read("archive/tasks/task.demo.md").unwrap(),
            text,
            "archive is a move: same filename, same bytes"
        );
        assert!(matches!(
            storage.read("tasks/task.demo.md"),
            Err(StorageError::NotFound { .. })
        ));
    }

    #[test]
    fn every_settled_type_archives() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.old.md",
                &record_file(
                    "decision.old",
                    "decision",
                    "superseded",
                    &["superseded-by: decision.new"],
                    "",
                ),
            ),
            (
                "decisions/decision.new.md",
                &record_file(
                    "decision.new",
                    "decision",
                    "active",
                    &["supersedes: decision.old"],
                    "",
                ),
            ),
            (
                "notes/note.done.md",
                &record_file("note.done", "note", "retired", &[], ""),
            ),
            (
                "questions/question.q.md",
                &record_file(
                    "question.q",
                    "question",
                    "routed",
                    &["routed-to: decision.new"],
                    "",
                ),
            ),
        ]);
        for (id, from, to) in [
            (
                "decision.old",
                "decisions/decision.old.md",
                "archive/decisions/decision.old.md",
            ),
            (
                "note.done",
                "notes/note.done.md",
                "archive/notes/note.done.md",
            ),
            (
                "question.q",
                "questions/question.q.md",
                "archive/questions/question.q.md",
            ),
        ] {
            let moved = Notebook::new(&mut storage).archive(id).unwrap();
            assert!(!moved.already, "{id} settles and must move");
            assert!(storage.read(to).is_ok(), "{id} must land in the archive");
            assert!(
                matches!(storage.read(from), Err(StorageError::NotFound { .. })),
                "{id} must leave the live directory"
            );
        }
    }

    #[test]
    fn a_replayed_archive_answers_already_and_changes_nothing() {
        let text = task_file("closed", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        Notebook::new(&mut storage).archive("task.demo").unwrap();
        let replay = Notebook::new(&mut storage).archive("task.demo").unwrap();
        assert!(replay.already);
        assert_eq!(storage.read("archive/tasks/task.demo.md").unwrap(), text);
    }

    #[test]
    fn a_corrupt_archived_copy_is_refused_not_called_already() {
        for (state, expected) in [
            ("open", FindingCode::ArchivedLiveRecord),
            ("bogus", FindingCode::BadValue),
        ] {
            let mut storage =
                storage_with(&[("archive/tasks/task.demo.md", &task_file(state, &[]))]);
            let refusal = Notebook::new(&mut storage)
                .archive("task.demo")
                .unwrap_err();
            match refusal {
                NotebookError::InvalidRecord { path, findings } => {
                    assert_eq!(path, "archive/tasks/task.demo.md");
                    assert_eq!(
                        findings.iter().map(|f| f.code).collect::<Vec<_>>(),
                        vec![expected]
                    );
                }
                other => panic!("`{state}` must not answer as a replayed move: {other:?}"),
            }
        }
    }

    #[test]
    fn a_live_record_is_refused_with_the_commands_that_settle_it() {
        let mut storage = storage_with(&[
            (
                "tasks/task.open.md",
                &record_file("task.open", "task", "open", &[], ""),
            ),
            (
                "tasks/task.working.md",
                &record_file("task.working", "task", "active", &[], ""),
            ),
            (
                "questions/question.q.md",
                &record_file("question.q", "question", "open", &[], ""),
            ),
            (
                "decisions/decision.d.md",
                &record_file("decision.d", "decision", "active", &[], ""),
            ),
        ]);
        let mut notebook = Notebook::new(&mut storage);
        for (id, state, valid) in [
            ("task.open", "open", vec!["start"]),
            ("task.working", "active", vec!["close"]),
            ("question.q", "open", vec!["answer"]),
            ("decision.d", "active", vec!["retire"]),
        ] {
            assert_eq!(
                notebook.archive(id).unwrap_err(),
                NotebookError::InvalidTransition {
                    id: id.to_owned(),
                    state: state.to_owned(),
                    valid,
                }
            );
        }
    }

    #[test]
    fn an_unknown_id_is_refused() {
        let mut storage = storage_with(&[]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .archive("task.ghost")
                .unwrap_err(),
            NotebookError::UnknownId { .. }
        ));
    }

    #[test]
    fn an_invalid_record_is_refused_before_any_byte_moves() {
        let text = task_file("cancelled", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .archive("task.demo")
                .unwrap_err(),
            NotebookError::InvalidRecord { .. }
        ));
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
    }

    #[test]
    fn a_divergent_archived_copy_is_never_overwritten() {
        let archived = task_file("closed", &["closed: 2026-08-20"]);
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("closed", &[])),
            ("archive/tasks/task.demo.md", &archived),
        ]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .archive("task.demo")
                .unwrap_err(),
            NotebookError::DuplicateId { .. }
        ));
        assert_eq!(
            storage.read("archive/tasks/task.demo.md").unwrap(),
            archived,
            "the archived history must survive the refused move"
        );
    }

    /// [`MemoryStorage`] whose `remove` always fails — the crash between an
    /// archive's write and its remove.
    struct RemoveFails(MemoryStorage);

    impl Storage for RemoveFails {
        fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
            self.0.list(dir)
        }
        fn read(&self, path: &str) -> Result<String, StorageError> {
            self.0.read(path)
        }
        fn write(&mut self, path: &str, content: &str) -> Result<(), StorageError> {
            self.0.write(path, content)
        }
        fn remove(&mut self, path: &str) -> Result<(), StorageError> {
            Err(StorageError::Io {
                path: path.to_owned(),
                detail: "refused".to_owned(),
            })
        }
    }

    #[test]
    fn a_move_interrupted_after_its_write_loses_nothing_and_replays_clean() {
        let text = task_file("closed", &[]);
        let mut storage = RemoveFails(storage_with(&[("tasks/task.demo.md", &text)]));
        assert!(Notebook::new(&mut storage).archive("task.demo").is_err());
        assert_eq!(
            storage.read("archive/tasks/task.demo.md").unwrap(),
            text,
            "the copy landed before the failure"
        );
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);

        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &text),
            ("archive/tasks/task.demo.md", &text),
        ]);
        let moved = Notebook::new(&mut storage).archive("task.demo").unwrap();
        assert!(!moved.already, "the identical copy is the crash replay");
        assert!(matches!(
            storage.read("tasks/task.demo.md"),
            Err(StorageError::NotFound { .. })
        ));
    }
}

mod edit_verb {
    use super::*;

    fn edit() -> Edit {
        Edit::default()
    }

    #[test]
    fn a_retitle_touches_only_the_title_and_updated_lines() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    title: Some("A sharper name".to_owned()),
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.changed, vec!["title"]);
        assert_eq!(
            storage.read("tasks/task.demo.md").unwrap(),
            "---\nid: task.demo\ntype: task\nstate: open\ntitle: A sharper name\ncreated: 2026-08-24\nupdated: 2026-08-27\n---\n"
        );
    }

    #[test]
    fn an_edit_matching_the_standing_values_changes_no_byte() {
        let text = task_file("open", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    title: Some("A demo record".to_owned()),
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.changed, Vec::<&str>::new());
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
    }

    #[test]
    fn a_new_body_replaces_the_old_one_whole() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("closed", &[]))]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    body: Some("The corrected story.".to_owned()),
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.changed, vec!["body"]);
        let text = storage.read("tasks/task.demo.md").unwrap();
        assert!(
            text.ends_with("---\n\nThe corrected story.\n"),
            "the edited body carries create's shape — blank line, content, final newline: {text}"
        );
    }

    #[test]
    fn an_empty_body_clears_it() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &record_file("task.demo", "task", "open", &[], "\nOld prose.\n"),
        )]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    body: Some(String::new()),
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.changed, vec!["body"]);
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .ends_with("---\n")
        );
    }

    #[test]
    fn tags_splice_into_the_standing_list() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &task_file("open", &["tags: cli, idea"]),
        )]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    add_tags: vec!["epic".to_owned()],
                    remove_tags: vec!["idea".to_owned()],
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.changed, vec!["tags"]);
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .contains("\ntags: cli, epic\n")
        );
    }

    #[test]
    fn removing_the_last_tag_drops_the_field() {
        let mut storage =
            storage_with(&[("tasks/task.demo.md", &task_file("open", &["tags: idea"]))]);
        Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    remove_tags: vec!["idea".to_owned()],
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert!(
            !storage
                .read("tasks/task.demo.md")
                .unwrap()
                .contains("tags:")
        );
    }

    #[test]
    fn the_origin_is_settable_retroactively() {
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("open", &[])),
            (
                "tasks/task.hub.md",
                &record_file("task.hub", "task", "open", &[], ""),
            ),
        ]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    from: Some("task.hub".to_owned()),
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.changed, vec!["from"]);
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .contains("\nfrom: task.hub\n")
        );
    }

    #[test]
    fn an_edit_requesting_nothing_is_refused() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .edit("task.demo", &edit(), TODAY)
                .unwrap_err(),
            NotebookError::InvalidArgument { .. }
        ));
    }

    #[test]
    fn a_whitespace_title_is_refused_before_any_byte_moves() {
        let text = task_file("open", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .edit(
                    "task.demo",
                    &Edit {
                        title: Some("   ".to_owned()),
                        ..edit()
                    },
                    TODAY,
                )
                .unwrap_err(),
            NotebookError::InvalidArgument { .. }
        ));
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
    }

    #[test]
    fn priority_on_a_non_task_is_refused() {
        let mut storage = storage_with(&[(
            "notes/note.n.md",
            &record_file("note.n", "note", "active", &[], ""),
        )]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .edit(
                    "note.n",
                    &Edit {
                        priority: Some(1),
                        ..edit()
                    },
                    TODAY,
                )
                .unwrap_err(),
            NotebookError::InvalidArgument { .. }
        ));
    }

    #[test]
    fn an_origin_naming_no_record_is_refused() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .edit(
                    "task.demo",
                    &Edit {
                        from: Some("task.ghost".to_owned()),
                        ..edit()
                    },
                    TODAY,
                )
                .unwrap_err(),
            NotebookError::DanglingRef { field: "from", .. }
        ));
    }

    #[test]
    fn a_record_cannot_become_its_own_origin() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .edit(
                    "task.demo",
                    &Edit {
                        from: Some("task.demo".to_owned()),
                        ..edit()
                    },
                    TODAY,
                )
                .unwrap_err(),
            NotebookError::InvalidArgument { .. }
        ));
    }

    #[test]
    fn a_review_by_that_is_not_a_date_is_refused() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .edit(
                    "task.demo",
                    &Edit {
                        review_by: Some("soon".to_owned()),
                        ..edit()
                    },
                    TODAY,
                )
                .unwrap_err(),
            NotebookError::InvalidArgument { .. }
        ));
    }

    #[test]
    fn an_archived_record_is_not_editable() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.done.md",
            &record_file("task.done", "task", "closed", &[], ""),
        )]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .edit(
                    "task.done",
                    &Edit {
                        title: Some("New name".to_owned()),
                        ..edit()
                    },
                    TODAY,
                )
                .unwrap_err(),
            NotebookError::Archived { .. }
        ));
    }

    #[test]
    fn priority_and_review_by_land_on_a_task() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    priority: Some(1),
                    review_by: Some("2026-09-15".to_owned()),
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.changed, vec!["priority", "review-by"]);
        let text = storage.read("tasks/task.demo.md").unwrap();
        assert!(text.contains("\npriority: 1\n"));
        assert!(text.contains("\nreview-by: 2026-09-15\n"));
        assert!(text.contains("\nupdated: 2026-08-27\n"));
    }

    #[test]
    fn a_new_body_citing_nothing_carries_the_dangling_mention_nudge() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    body: Some("Blocked by task.ghost.".to_owned()),
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.dangling_mentions, vec!["task.ghost"]);
    }
}

mod search_query {
    use super::*;

    fn demo_notebook() -> MemoryStorage {
        storage_with(&[
            (
                "tasks/task.parser.md",
                &record_file("task.parser", "task", "open", &[], "The grammar work.\n"),
            ),
            (
                "decisions/decision.rust.md",
                &record_file("decision.rust", "decision", "active", &["tags: stack"], ""),
            ),
            (
                "archive/tasks/task.spike.md",
                &record_file("task.spike", "task", "closed", &[], "Parser spike notes.\n"),
            ),
        ])
    }

    #[test]
    fn a_query_matches_titles_case_insensitively() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let rows = Notebook::new(&mut storage).search("DEMO RECORD").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "task.demo");
    }

    #[test]
    fn a_query_matches_bodies_ids_and_tags() {
        let mut storage = demo_notebook();
        let notebook = Notebook::new(&mut storage);
        let ids = |rows: Vec<anb_core::ListedRecord>| {
            rows.into_iter().map(|row| row.id).collect::<Vec<_>>()
        };
        assert_eq!(
            ids(notebook.search("grammar").unwrap()),
            vec!["task.parser"],
            "the body is a searched surface"
        );
        assert_eq!(
            ids(notebook.search("stack").unwrap()),
            vec!["decision.rust"],
            "tags are a searched surface"
        );
        assert_eq!(
            ids(notebook.search("task.spike").unwrap()),
            vec!["task.spike"],
            "the id is a searched surface"
        );
    }

    #[test]
    fn the_archive_is_searched_too() {
        let mut storage = demo_notebook();
        let rows = Notebook::new(&mut storage).search("parser").unwrap();
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["task.parser", "task.spike"],
            "history is findable; live rows lead within a type"
        );
    }

    #[test]
    fn an_empty_query_is_refused() {
        let mut storage = storage_with(&[]);
        assert!(matches!(
            Notebook::new(&mut storage).search("  ").unwrap_err(),
            NotebookError::InvalidArgument { .. }
        ));
    }

    #[test]
    fn a_query_matching_nothing_answers_no_rows() {
        let mut storage = demo_notebook();
        assert_eq!(
            Notebook::new(&mut storage).search("zeppelin").unwrap(),
            vec![]
        );
    }

    #[test]
    fn an_invalid_match_shows_as_state_invalid() {
        let mut storage = storage_with(&[(
            "tasks/task.broken.md",
            &record_file("task.broken", "task", "cancelled", &[], ""),
        )]);
        let rows = Notebook::new(&mut storage).search("broken").unwrap();
        assert_eq!(rows[0].state, "invalid");
    }
}

mod overview_query {
    use super::*;

    #[test]
    fn the_page_groups_live_records_by_type_and_counts_the_archive() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &record_file("task.a", "task", "open", &[], ""),
            ),
            (
                "decisions/decision.d.md",
                &record_file("decision.d", "decision", "active", &[], ""),
            ),
            (
                "archive/tasks/task.done.md",
                &record_file("task.done", "task", "closed", &[], ""),
            ),
        ]);
        let overview = Notebook::new(&mut storage).overview().unwrap();
        let shape: Vec<(RecordType, Vec<&str>)> = overview
            .sections
            .iter()
            .map(|section| {
                (
                    section.record_type,
                    section.rows.iter().map(|row| row.id.as_str()).collect(),
                )
            })
            .collect();
        assert_eq!(
            shape,
            vec![
                (RecordType::Task, vec!["task.a"]),
                (RecordType::Decision, vec!["decision.d"]),
                (RecordType::Note, vec![]),
                (RecordType::Question, vec![]),
            ],
            "every type has its section, empty ones included — the renderer decides what to show"
        );
        assert_eq!(
            (overview.archived.tasks, overview.archived.decisions),
            (1, 0)
        );
        assert_eq!(
            (overview.live.tasks, overview.live.decisions),
            (1, 1),
            "the page's own tally, computed once beside the rows"
        );
    }

    #[test]
    fn an_invalid_record_is_a_row_of_state_invalid() {
        let mut storage = storage_with(&[(
            "tasks/task.broken.md",
            &record_file("task.broken", "task", "cancelled", &[], ""),
        )]);
        let overview = Notebook::new(&mut storage).overview().unwrap();
        assert_eq!(overview.sections[0].rows[0].state, "invalid");
    }
}

mod unreadable_files {
    use super::*;

    /// [`MemoryStorage`] holds strings, so the adapter's duty is simulated:
    /// the marked paths answer reads with [`StorageError::NotUtf8`].
    struct BinaryHolding {
        inner: MemoryStorage,
        binary: Vec<String>,
    }

    impl BinaryHolding {
        fn with_binary_at(path: &str, files: &[(&str, &str)]) -> Self {
            let mut all: Vec<(&str, &str)> = files.to_vec();
            all.push((path, ""));
            BinaryHolding {
                inner: MemoryStorage::from_files(all),
                binary: vec![path.to_owned()],
            }
        }
    }

    impl Storage for BinaryHolding {
        fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
            self.inner.list(dir)
        }

        fn read(&self, path: &str) -> Result<String, StorageError> {
            if self.binary.iter().any(|held| held == path) {
                return Err(StorageError::NotUtf8 {
                    path: path.to_owned(),
                });
            }
            self.inner.read(path)
        }

        fn write(&mut self, path: &str, content: &str) -> Result<(), StorageError> {
            self.inner.write(path, content)
        }

        fn remove(&mut self, path: &str) -> Result<(), StorageError> {
            self.inner.remove(path)
        }
    }

    #[test]
    fn check_names_the_file_with_the_not_utf8_finding() {
        let storage = &mut BinaryHolding::with_binary_at("tasks/task.binary.md", &[]);
        let findings = Notebook::new(storage).check().unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].path, "tasks/task.binary.md");
        assert_eq!(findings[0].finding.code, FindingCode::NotUtf8);
    }

    #[test]
    fn an_unreadable_archived_copy_refuses_the_move_as_an_invalid_record() {
        let storage = &mut BinaryHolding::with_binary_at("archive/tasks/task.demo.md", &[]);
        let refusal = Notebook::new(storage).archive("task.demo").unwrap_err();
        match refusal {
            NotebookError::InvalidRecord { path, findings } => {
                assert_eq!(path, "archive/tasks/task.demo.md");
                assert_eq!(
                    findings.iter().map(|f| f.code).collect::<Vec<_>>(),
                    vec![FindingCode::NotUtf8]
                );
            }
            other => panic!("bytes that cannot be read name a record, not the medium: {other:?}"),
        }
    }

    #[test]
    fn the_listing_shows_the_file_as_invalid_instead_of_aborting() {
        let text = record_file("task.a", "task", "open", &[], "");
        let storage = &mut BinaryHolding::with_binary_at(
            "tasks/task.binary.md",
            &[("tasks/task.a.md", &text)],
        );
        let rows = Notebook::new(storage).list().unwrap();
        let states: Vec<(&str, &str)> = rows
            .iter()
            .map(|row| (row.id.as_str(), row.state.as_str()))
            .collect();
        assert_eq!(states, vec![("task.a", "open"), ("task.binary", "invalid")]);
    }

    #[test]
    fn a_mutation_is_refused_with_the_not_utf8_finding() {
        let storage = &mut BinaryHolding::with_binary_at("tasks/task.binary.md", &[]);
        let error = Notebook::new(storage)
            .start("task.binary", TODAY)
            .unwrap_err();
        let NotebookError::InvalidRecord { findings, .. } = error else {
            panic!("the refusal must carry the finding, got {error:?}");
        };
        assert_eq!(findings[0].code, FindingCode::NotUtf8);
    }

    #[test]
    fn a_binary_config_is_a_named_check_finding() {
        let storage = &mut BinaryHolding::with_binary_at("config", &[]);
        let findings = Notebook::new(storage).check().unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].path, "config");
        assert_eq!(findings[0].finding.code, FindingCode::NotUtf8);
    }

    #[test]
    fn an_id_held_by_an_unreadable_file_is_still_taken() {
        let storage = &mut BinaryHolding::with_binary_at("tasks/task.binary.md", &[]);
        let mut draft = Draft::new(RecordType::Task, "A demo record");
        draft.id = Some("task.binary".to_owned());
        assert!(matches!(
            Notebook::new(storage).create(&draft, TODAY).unwrap_err(),
            NotebookError::DuplicateId { .. }
        ));
    }
}

/// The epic pattern: a hub Task, its scope, and where it stands.
mod epics {
    use super::*;
    use anb_core::Epic;

    fn task(id: &str, state: &str, extra: &[&str]) -> (String, String) {
        (
            format!("tasks/{id}.md"),
            record_file(id, "task", state, extra, ""),
        )
    }

    /// A hub with three children, one of them closed, and a Question born
    /// inside it — the shape the pattern describes, written from both ends.
    fn an_epic() -> MemoryStorage {
        let files = [
            task(
                "task.epic-auth",
                "open",
                &[
                    "blocked-by: task.auth-login",
                    "blocked-by: task.auth-tokens",
                    "blocked-by: task.auth-audit",
                ],
            ),
            task("task.auth-login", "closed", &["from: task.epic-auth"]),
            task("task.auth-tokens", "open", &["from: task.epic-auth"]),
            task("task.auth-audit", "open", &["from: task.epic-auth"]),
            (
                "questions/question.auth-doubt.md".to_owned(),
                record_file(
                    "question.auth-doubt",
                    "question",
                    "open",
                    &["from: task.auth-tokens"],
                    "",
                ),
            ),
            task("task.unrelated", "open", &[]),
        ];
        storage_with(
            &files
                .iter()
                .map(|(path, text)| (path.as_str(), text.as_str()))
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn a_hub_reports_the_children_it_waits_on_and_what_to_pick_up_next() {
        assert_eq!(
            Notebook::new(&mut an_epic()).epics().unwrap(),
            vec![Epic {
                id: "task.epic-auth".to_owned(),
                closed: 1,
                total: 3,
                next: Some("task.auth-audit".to_owned()),
            }]
        );
    }

    #[test]
    fn a_task_that_merely_spawned_a_question_is_no_hub() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &record_file("task.demo", "task", "open", &[], ""),
            ),
            (
                "questions/question.doubt.md",
                &record_file(
                    "question.doubt",
                    "question",
                    "open",
                    &["from: task.demo"],
                    "",
                ),
            ),
        ]);
        assert_eq!(
            Notebook::new(&mut storage).epics().unwrap(),
            vec![],
            "origin alone is not decomposition — the task does not wait on the doubt"
        );
    }

    #[test]
    fn a_task_blocked_by_a_plain_dependency_is_no_hub() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &record_file("task.demo", "task", "open", &["blocked-by: task.other"], ""),
            ),
            (
                "tasks/task.other.md",
                &record_file("task.other", "task", "open", &[], ""),
            ),
        ]);
        assert_eq!(
            Notebook::new(&mut storage).epics().unwrap(),
            vec![],
            "waiting on something is not having given birth to it"
        );
    }

    #[test]
    fn scope_reaches_what_was_born_inside_it_however_deep() {
        assert_eq!(
            ids(Notebook::new(&mut an_epic())
                .list_for("task.epic-auth")
                .unwrap()),
            // Notebook order: type-major, then by path.
            vec![
                "task.auth-audit",
                "task.auth-login",
                "task.auth-tokens",
                "task.epic-auth",
                "question.auth-doubt",
            ],
            "the Question two Origins down belongs to the epic; the unrelated Task does not"
        );
    }

    #[test]
    fn the_scoped_queue_is_the_queue_narrowed_and_nothing_else() {
        let mut storage = an_epic();
        let notebook = Notebook::new(&mut storage);
        let scoped: Vec<String> = notebook
            .ready_for("task.epic-auth")
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(scoped, vec!["task.auth-audit", "task.auth-tokens"]);
        assert!(
            notebook
                .ready()
                .unwrap()
                .iter()
                .any(|row| row.id == "task.unrelated"),
            "the unscoped queue still carries what the scope left out"
        );
    }

    #[test]
    fn a_hub_whose_children_have_all_closed_is_awaiting_its_acceptance() {
        let mut storage = storage_with(&[
            (
                "tasks/task.epic-auth.md",
                &record_file(
                    "task.epic-auth",
                    "task",
                    "open",
                    &["blocked-by: task.auth-login"],
                    "",
                ),
            ),
            (
                "tasks/task.auth-login.md",
                &record_file(
                    "task.auth-login",
                    "task",
                    "closed",
                    &["from: task.epic-auth"],
                    "",
                ),
            ),
        ]);
        assert_eq!(
            Notebook::new(&mut storage).epics().unwrap(),
            vec![Epic {
                id: "task.epic-auth".to_owned(),
                closed: 1,
                total: 1,
                next: None,
            }],
            "a hub that reaches ready asks for its close, not for more work"
        );
    }

    #[test]
    fn a_scope_named_by_no_record_is_refused_rather_than_answered_empty() {
        let mut storage = an_epic();
        assert_eq!(
            Notebook::new(&mut storage)
                .ready_for("task.no-such-epic")
                .unwrap_err(),
            NotebookError::UnknownId {
                id: "task.no-such-epic".to_owned()
            }
        );
    }

    #[test]
    fn what_a_child_waits_on_is_work_the_epic_waits_on_too() {
        let mut storage = storage_with(&[
            (
                "tasks/task.epic.md",
                &record_file("task.epic", "task", "open", &["blocked-by: task.child"], ""),
            ),
            (
                "tasks/task.child.md",
                &record_file(
                    "task.child",
                    "task",
                    "open",
                    &["from: task.epic", "blocked-by: task.outside"],
                    "",
                ),
            ),
            (
                "tasks/task.outside.md",
                &record_file("task.outside", "task", "open", &[], ""),
            ),
        ]);
        let notebook = Notebook::new(&mut storage);
        assert_eq!(
            ids(notebook.list_for("task.epic").unwrap()),
            vec!["task.child", "task.epic", "task.outside"],
            "it must close before the child, which must close before the hub"
        );
        assert_eq!(
            notebook
                .ready_for("task.epic")
                .unwrap()
                .into_iter()
                .map(|row| row.id)
                .collect::<Vec<String>>(),
            vec!["task.outside"],
            "and it is the one thing the epic can actually be got on with"
        );
    }

    #[test]
    fn a_tier_assembled_from_the_hub_side_is_still_reached() {
        // The shape of an epic older than the edit surface: the middle tier
        // names its children, and they carry no Origin back.
        let mut storage = storage_with(&[
            (
                "tasks/task.outer.md",
                &record_file(
                    "task.outer",
                    "task",
                    "open",
                    &["blocked-by: task.born", "blocked-by: task.middle"],
                    "",
                ),
            ),
            (
                "tasks/task.born.md",
                &record_file("task.born", "task", "closed", &["from: task.outer"], ""),
            ),
            (
                "tasks/task.middle.md",
                &record_file(
                    "task.middle",
                    "task",
                    "open",
                    &["blocked-by: task.g1", "blocked-by: task.g2"],
                    "",
                ),
            ),
            (
                "tasks/task.g1.md",
                &record_file("task.g1", "task", "open", &[], ""),
            ),
            (
                "tasks/task.g2.md",
                &record_file("task.g2", "task", "open", &[], ""),
            ),
        ]);
        let notebook = Notebook::new(&mut storage);
        assert_eq!(
            notebook
                .ready_for("task.outer")
                .unwrap()
                .into_iter()
                .map(|row| row.id)
                .collect::<Vec<String>>(),
            vec!["task.g1", "task.g2"],
            "an epic with dispatchable work must never report an empty queue"
        );
        assert_eq!(
            notebook.epics().unwrap()[0].next.as_deref(),
            Some("task.g1")
        );
    }

    #[test]
    fn a_closed_hub_is_no_longer_an_epic_in_flight() {
        let mut storage = storage_with(&[
            (
                "tasks/task.epic.md",
                &record_file(
                    "task.epic",
                    "task",
                    "closed",
                    &["blocked-by: task.child"],
                    "",
                ),
            ),
            (
                "tasks/task.child.md",
                &record_file("task.child", "task", "closed", &["from: task.epic"], ""),
            ),
        ]);
        assert_eq!(
            Notebook::new(&mut storage).epics().unwrap(),
            vec![],
            "its acceptance close has happened; asking for it again asks for done work"
        );
    }

    #[test]
    fn an_invalid_hub_is_excluded_like_every_other_derived_query() {
        let mut storage = storage_with(&[
            (
                "tasks/task.epic.md",
                &record_file(
                    "task.epic",
                    "task",
                    "open",
                    &["blocked-by: question.doubt"],
                    "",
                ),
            ),
            (
                "questions/question.doubt.md",
                &record_file(
                    "question.doubt",
                    "question",
                    "open",
                    &["from: task.epic"],
                    "",
                ),
            ),
        ]);
        assert_eq!(
            Notebook::new(&mut storage).epics().unwrap(),
            vec![],
            "one command must not call a record invalid in one block and an epic in another"
        );
    }

    fn ids(rows: Vec<anb_core::ListedRecord>) -> Vec<String> {
        rows.into_iter().map(|row| row.id).collect()
    }
}
