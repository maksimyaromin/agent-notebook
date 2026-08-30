/// What a caller learns from a refusal beyond its own text.
mod refusals {
    use crate::*;
    use std::error::Error as _;

    #[test]
    fn a_storage_failure_stays_reachable_under_the_refusal_that_carries_it() {
        let mut storage = MemoryStorage::new();
        let refusal = Notebook::new(&mut storage)
            .start("task.ghost", TODAY)
            .unwrap_err();
        assert_eq!(refusal.code(), "unknown-id");
        assert!(
            refusal.source().is_none(),
            "a refusal of the notebook's own is nobody else's failure"
        );

        let carried = NotebookError::from(StorageError::NotUtf8 {
            path: "tasks/task.demo.md".to_owned(),
        });
        assert_eq!(carried.code(), "not-utf8");
        assert_eq!(
            carried.source().map(ToString::to_string),
            Some("not UTF-8: tasks/task.demo.md".to_owned()),
            "the adapter's failure is the source of the refusal above it"
        );
    }
}

mod task_cycle {
    use crate::*;

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
        notebook.archive(&filed, TODAY).unwrap();
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
}

mod hold {
    use crate::*;

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

    /// A hand-broken hold freezes a Task while naming the verb that would
    /// free it, so `unhold` reads over its own findings: erasing the pair
    /// is the repair, and the record it leaves carries neither.
    #[test]
    fn unhold_erases_a_hold_whose_date_is_broken() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &task_file("active", &["hold: a reason", "hold-until: someday"]),
        )]);

        let reply = Notebook::new(&mut storage)
            .unhold("task.demo", TODAY)
            .unwrap();

        assert!(!reply.already);
        let text = storage.read("tasks/task.demo.md").unwrap();
        assert!(!text.contains("\nhold:"), "{text}");
        assert!(!text.contains("\nhold-until:"), "{text}");
        assert!(
            Notebook::new(&mut storage).check().unwrap().is_empty(),
            "the repaired record reads clean"
        );
    }

    /// The admission is for a repair, not a pass: a verb that would erase
    /// nothing leaves every finding standing, so it is refused rather than
    /// answered `already`.
    #[test]
    fn unhold_is_refused_on_a_broken_task_that_carries_no_hold() {
        let mut storage =
            storage_with(&[("tasks/task.demo.md", &task_file("active", &["priority: 9"]))]);

        let refusal = Notebook::new(&mut storage)
            .unhold("task.demo", TODAY)
            .unwrap_err();

        assert!(
            matches!(refusal, NotebookError::InvalidRecord { .. }),
            "{refusal:?}"
        );
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
    use crate::*;

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
            storage.read("tasks/task.demo.md").unwrap(),
            "---\nid: task.demo\ntype: task\nstate: active\ntitle: A demo record\ncreated: 2026-08-24\nupdated: 2026-08-27\n---\n- 2026-08-27 claude-code: parser done, tests next\n"
        );
    }

    #[test]
    fn an_entry_without_an_acting_hand_shows_the_dash() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        Notebook::new(&mut storage)
            .comment("task.demo", None, "stopped at the render step", TODAY)
            .unwrap();
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .ends_with("- 2026-08-27 -: stopped at the render step\n")
        );
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

mod routing {
    use crate::*;

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
    use crate::*;

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
