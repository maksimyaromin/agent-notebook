//! The record model's write-time invariants, specified at the Storage seam:
//! strings in, exact strings and returned models out. Expected file bytes
//! derive from the format spec's canonical form, never from running the
//! code.

use anb_core::{
    Draft, FindingCode, Link, MemoryStorage, Notebook, NotebookError, Proof, RecordType, Storage,
    Transitioned,
};

const TODAY: &str = "2026-08-27";

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
            .drop_question("question.demo", "overtaken by the S2 decision", TODAY)
            .unwrap();
        assert_eq!(reply, moved("question.demo", "open", "dropped"));
        let text = storage.read("questions/question.demo.md").unwrap();
        assert!(text.contains("\nstate: dropped\n"));
        assert!(text.ends_with("---\nDropped 2026-08-27: overtaken by the S2 decision\n"));
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
        assert!(replay.already);
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
            target: ".tmp/docs/spec-anb-format.md".to_owned(),
        }];
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert!(
            storage
                .read(&created.path)
                .unwrap()
                .contains("link: doc .tmp/docs/spec-anb-format.md\n")
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
    fn two_files_claiming_one_id_are_both_named() {
        let text = task_file("open", &[]);
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
                "decisions/decision.old.md",
                &record_file("decision.old", "decision", "retired", &[], ""),
            ),
        ]);
        assert_eq!(
            findings_for(&mut storage),
            vec![
                (
                    "decisions/decision.new.md".to_owned(),
                    FindingCode::BrokenSupersession
                ),
                (
                    "decisions/decision.old.md".to_owned(),
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
    fn a_routed_question_pointing_at_nothing_has_lost_its_thread() {
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
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
                "questions/question.demo.md".to_owned(),
                FindingCode::BrokenRouting
            )]
        );
    }
}
