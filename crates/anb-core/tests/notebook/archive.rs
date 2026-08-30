mod expunge_verb {
    use crate::*;

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

    /// History cites like anything else: a filed record still names what it
    /// grew from, and expunging out from under it would leave the archive
    /// pointing at a record that never existed.
    #[test]
    fn a_record_the_archive_still_names_cannot_be_expunged() {
        let mut storage = storage_with(&[
            (
                "notes/note.mistake.md",
                &record_file("note.mistake", "note", "active", &[], ""),
            ),
            (
                "archive/tasks/task.filed.md",
                &record_file("task.filed", "task", "closed", &["from: note.mistake"], ""),
            ),
        ]);
        assert_eq!(
            blockers_of(&mut storage, "note.mistake"),
            vec![held_by("task.filed", "from")]
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
    use crate::*;

    #[test]
    fn a_settled_task_moves_to_the_archive_byte_identical() {
        // Warnings only — BOM, a CRLF line, quirky spacing — so the move is
        // legal and byte-exactness is observable: a canonicalizing copy
        // would rewrite this file.
        let text = "\u{feff}---\nid: task.demo\ntype:  task\r\nstate: closed\ntitle: A demo record\ncreated: 2026-08-24\n---\nbody\n";
        let mut storage = storage_with(&[("tasks/task.demo.md", text)]);
        let moved = Notebook::new(&mut storage)
            .archive("task.demo", TODAY)
            .unwrap();
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
            let moved = Notebook::new(&mut storage).archive(id, TODAY).unwrap();
            assert!(!moved.already, "{id} settles and must move");
            assert!(storage.read(to).is_ok(), "{id} must land in the archive");
            assert!(
                matches!(storage.read(from), Err(StorageError::NotFound { .. })),
                "{id} must leave the live directory"
            );
        }
    }

    #[test]
    fn archiving_a_record_carries_the_report_it_closed_with() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &task_file("closed", &["link: note note.report"]),
            ),
            (
                "notes/note.report.md",
                &record_file(
                    "note.report",
                    "note",
                    "active",
                    &["from: task.demo"],
                    "What the work came to.\n",
                ),
            ),
        ]);
        let moved = Notebook::new(&mut storage)
            .archive("task.demo", TODAY)
            .unwrap();
        assert_eq!(moved.carried, ["note.report"]);
        assert!(matches!(
            storage.read("notes/note.report.md"),
            Err(StorageError::NotFound { .. })
        ));
        let filed = storage.read("archive/notes/note.report.md").unwrap();
        assert!(
            filed.contains("state: retired") && filed.contains("What the work came to."),
            "a carried report is settled where it lands: {filed}"
        );
    }

    /// A report is the record's own output: linked as evidence and born
    /// inside it. Either half alone leaves the Note where it is.
    #[test]
    fn only_a_report_born_inside_the_record_is_carried() {
        for (case, links, origin) in [
            ("a Note born from another Task", true, "task.other"),
            ("a Note this record never linked", false, "task.demo"),
        ] {
            let task_lines: &[&str] = if links {
                &["link: note note.report"]
            } else {
                &[]
            };
            let path = "notes/note.report.md";
            let mut storage = storage_with(&[
                ("tasks/task.demo.md", &task_file("closed", task_lines)),
                (
                    path,
                    &record_file(
                        "note.report",
                        "note",
                        "active",
                        &[&format!("from: {origin}")],
                        "",
                    ),
                ),
            ]);
            let moved = Notebook::new(&mut storage)
                .archive("task.demo", TODAY)
                .unwrap();
            assert!(
                moved.carried.is_empty(),
                "{case} is not this record\'s report"
            );
            assert!(storage.read(path).is_ok(), "{case} must stay live");
        }
    }

    /// The cascade is judged whole before it starts, so a report the
    /// notebook cannot move refuses the archive and leaves every record
    /// where it was — the repair is on the Note, and the move is one call
    /// away.
    #[test]
    fn a_report_that_cannot_be_judged_refuses_the_move_and_keeps_every_byte() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &task_file("closed", &["link: note note.report"]),
            ),
            (
                "notes/note.report.md",
                &record_file("note.report", "note", "wrong", &["from: task.demo"], ""),
            ),
        ]);

        let refusal = Notebook::new(&mut storage)
            .archive("task.demo", TODAY)
            .unwrap_err();
        assert!(
            matches!(&refusal, NotebookError::InvalidRecord { path, .. } if path == "notes/note.report.md"),
            "{refusal:?}"
        );
        assert!(
            storage.read("tasks/task.demo.md").is_ok()
                && matches!(
                    storage.read("archive/tasks/task.demo.md"),
                    Err(StorageError::NotFound { .. })
                ),
            "the record the caller named must not move on a refused cascade"
        );

        storage
            .write(
                "notes/note.report.md",
                &record_file("note.report", "note", "active", &["from: task.demo"], ""),
            )
            .unwrap();
        let moved = Notebook::new(&mut storage)
            .archive("task.demo", TODAY)
            .unwrap();
        assert_eq!(moved.carried, ["note.report"]);
    }

    /// The cascade decides before it writes, so a report whose archived
    /// home is taken by other bytes refuses the move with every record
    /// untouched — including the state of the report itself.
    #[test]
    fn a_report_whose_home_is_taken_refuses_before_anything_is_retired() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &task_file("closed", &["link: note note.report"]),
            ),
            (
                "notes/note.report.md",
                &record_file("note.report", "note", "active", &["from: task.demo"], ""),
            ),
            (
                "archive/notes/note.report.md",
                &record_file("note.report", "note", "retired", &[], "Older bytes.\n"),
            ),
        ]);

        let refusal = Notebook::new(&mut storage)
            .archive("task.demo", TODAY)
            .unwrap_err();
        assert!(
            matches!(&refusal, NotebookError::DuplicateId { id, .. } if id == "note.report"),
            "{refusal:?}"
        );
        assert!(
            storage
                .read("notes/note.report.md")
                .unwrap()
                .contains("state: active"),
            "a refused cascade retires nothing"
        );
        assert!(storage.read("tasks/task.demo.md").is_ok());
    }

    /// A link target is free text until the grammar says otherwise, and a
    /// target that is no Note id is never spelled into a path. The proof is
    /// a file waiting at exactly the path an unguarded cascade would build
    /// from each shape: one that climbs out of the notes directory, one
    /// whose case is wrong, and one that names another type.
    #[test]
    fn a_link_target_that_is_no_note_id_is_never_turned_into_a_path() {
        for (target, reachable_only_unguarded) in [
            ("../notes/note.report", "notes/../notes/note.report.md"),
            ("NOTE.REPORT", "notes/NOTE.REPORT.md"),
            ("task.report", "notes/task.report.md"),
        ] {
            let mut storage = storage_with(&[
                (
                    "tasks/task.demo.md",
                    &task_file("closed", &[&format!("link: note {target}")]),
                ),
                (
                    reachable_only_unguarded,
                    &record_file("note.report", "note", "active", &["from: task.demo"], ""),
                ),
            ]);

            let moved = Notebook::new(&mut storage)
                .archive("task.demo", TODAY)
                .unwrap();

            assert!(moved.carried.is_empty(), "`{target}` names no report");
            assert!(
                storage.read(reachable_only_unguarded).is_ok(),
                "`{target}` left the file it would have named where it lies"
            );
        }
    }

    /// The reports move first, so an interruption leaves the record live
    /// and the next call finishes the job: what is already filed is no
    /// longer carriable, and the record follows it.
    #[test]
    fn an_interrupted_cascade_is_finished_by_the_next_call() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &task_file("closed", &["link: note note.report"]),
            ),
            (
                "archive/notes/note.report.md",
                &record_file("note.report", "note", "retired", &["from: task.demo"], ""),
            ),
        ]);

        let moved = Notebook::new(&mut storage)
            .archive("task.demo", TODAY)
            .unwrap();
        assert!(!moved.already && moved.carried.is_empty());
        assert!(storage.read("archive/tasks/task.demo.md").is_ok());
    }

    #[test]
    fn a_replayed_archive_answers_already_and_changes_nothing() {
        let text = task_file("closed", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        Notebook::new(&mut storage)
            .archive("task.demo", TODAY)
            .unwrap();
        let replay = Notebook::new(&mut storage)
            .archive("task.demo", TODAY)
            .unwrap();
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
                .archive("task.demo", TODAY)
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
                notebook.archive(id, TODAY).unwrap_err(),
                NotebookError::InvalidTransition {
                    id: id.to_owned(),
                    state: state.to_owned(),
                    valid,
                }
            );
        }
    }

    #[test]
    fn an_invalid_record_is_refused_before_any_byte_moves() {
        let text = task_file("cancelled", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .archive("task.demo", TODAY)
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
                .archive("task.demo", TODAY)
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
        assert!(
            Notebook::new(&mut storage)
                .archive("task.demo", TODAY)
                .is_err()
        );
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
        let moved = Notebook::new(&mut storage)
            .archive("task.demo", TODAY)
            .unwrap();
        assert!(!moved.already, "the identical copy is the crash replay");
        assert!(matches!(
            storage.read("tasks/task.demo.md"),
            Err(StorageError::NotFound { .. })
        ));
    }
}
