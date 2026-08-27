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
