mod dependency_graph {
    use crate::*;
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
    fn a_cycle_through_a_misnamed_task_file_is_refused_like_any_other() {
        // The Task directory is the graph: `tasks/note.x.md` says
        // `type: task` and carries a real edge, whatever its name claims.
        let mut storage = storage_with(&[
            ("tasks/task.a.md", &task("task.a", "open", &[])),
            (
                "tasks/task.b.md",
                &task("task.b", "open", &["blocked-by: note.x"]),
            ),
            (
                "tasks/note.x.md",
                &task("note.x", "open", &["blocked-by: task.a"]),
            ),
        ]);
        assert_eq!(
            Notebook::new(&mut storage)
                .block("task.a", "task.b", TODAY)
                .unwrap_err(),
            NotebookError::WouldCycle {
                chain: vec![
                    "task.a".to_owned(),
                    "task.b".to_owned(),
                    "note.x".to_owned(),
                    "task.a".to_owned(),
                ],
            },
            "the write guard and check must agree on what the graph holds"
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

    /// `unblock` is let past the findings to erase one edge; the edge it
    /// erases has to be the one they are about.
    #[test]
    fn unblocking_a_sound_edge_beside_a_corrupted_one_is_refused() {
        let text = task(
            "task.a",
            "open",
            &["blocked-by: task.ghost", "blocked-by: task.b"],
        );
        let mut storage = storage_with(&[
            ("tasks/task.a.md", &text),
            ("tasks/task.b.md", &task("task.b", "open", &[])),
        ]);
        let error = Notebook::new(&mut storage)
            .unblock("task.a", "task.b", TODAY)
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidRecord { .. }));
        assert_eq!(
            storage.read("tasks/task.a.md").unwrap(),
            text,
            "the corrupted edge would have outlived the write"
        );
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
    use crate::*;
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

/// The epic pattern: a hub Task, its scope, and where it stands.
mod epics {
    use crate::*;
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
