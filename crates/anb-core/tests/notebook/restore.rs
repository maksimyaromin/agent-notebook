mod restore_verb {
    use crate::*;

    #[test]
    fn an_archived_record_moves_back_byte_identical() {
        // Warnings only — BOM, a CRLF line, quirky spacing — so
        // byte-exactness is observable: a canonicalizing copy would
        // rewrite this file.
        let text = "\u{feff}---\nid: task.demo\ntype:  task\r\nstate: closed\ntitle: A demo record\ncreated: 2026-08-24\n---\nbody\n";
        let mut storage = storage_with(&[("archive/tasks/task.demo.md", text)]);
        let moved = Notebook::new(&mut storage).restore("task.demo").unwrap();
        assert_eq!(moved.from, "archive/tasks/task.demo.md");
        assert_eq!(moved.to, "tasks/task.demo.md");
        assert!(!moved.already);
        assert_eq!(
            storage.read("tasks/task.demo.md").unwrap(),
            text,
            "restore is a move: same filename, same bytes"
        );
        assert!(matches!(
            storage.read("archive/tasks/task.demo.md"),
            Err(StorageError::NotFound { .. })
        ));
    }

    #[test]
    fn the_other_record_types_restore_the_same_way() {
        let cases = [
            ("decision.old", "decision", "retired", "decisions"),
            ("note.done", "note", "retired", "notes"),
            ("question.q", "question", "closed", "questions"),
        ];
        for (id, type_word, state, home) in cases {
            let extra: &[&str] = if type_word == "question" {
                &["reason: moot"]
            } else {
                &[]
            };
            let from = format!("archive/{home}/{id}.md");
            let mut storage =
                storage_with(&[(from.as_str(), &record_file(id, type_word, state, extra, ""))]);
            let moved = Notebook::new(&mut storage).restore(id).unwrap();
            assert!(!moved.already, "{id} sits archived and must move");
            assert!(
                storage.read(&format!("{home}/{id}.md")).is_ok(),
                "{id} must land in the live home"
            );
            assert!(
                matches!(storage.read(&from), Err(StorageError::NotFound { .. })),
                "{id} must leave the archive"
            );
        }
    }

    /// The corruption `check` names as `archived-live-record`: a record
    /// whose state still binds while its file sits in the archive.
    /// Restore is its undo.
    #[test]
    fn restore_erases_the_archived_live_record_finding() {
        let mut storage = storage_with(&[("archive/tasks/task.demo.md", &task_file("open", &[]))]);
        Notebook::new(&mut storage).restore("task.demo").unwrap();
        let findings = Notebook::new(&mut storage).check().unwrap();
        assert!(
            findings.is_empty(),
            "the residence disagreement is undone: {findings:?}"
        );
    }

    #[test]
    fn a_restored_record_is_back_in_the_task_cycle() {
        let mut storage = storage_with(&[("archive/tasks/task.demo.md", &task_file("open", &[]))]);
        Notebook::new(&mut storage).restore("task.demo").unwrap();
        assert_eq!(
            Notebook::new(&mut storage)
                .start("task.demo", TODAY)
                .unwrap(),
            moved("task.demo", "open", "active")
        );
    }

    /// The bytes travel unjudged: a record carrying an error finding must
    /// be able to come back to where the repairing verbs are, or a broken
    /// archived record could never be repaired at all.
    #[test]
    fn an_invalid_archived_record_is_restored_verbatim_for_repair() {
        let text = task_file("open", &["priority: 9"]);
        let mut storage = storage_with(&[("archive/tasks/task.demo.md", &text)]);
        Notebook::new(&mut storage).restore("task.demo").unwrap();
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
    }

    /// The point of the verbatim move: once the record is back, `check`
    /// names the eraser for what is broken inside it, and the eraser runs.
    #[test]
    fn a_finding_that_came_back_from_the_archive_is_repairable() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.demo.md",
            &task_file("open", &["priority: 9"]),
        )]);
        Notebook::new(&mut storage).restore("task.demo").unwrap();
        let findings = Notebook::new(&mut storage).check().unwrap();
        let named = findings
            .iter()
            .find(|located| located.finding.code == FindingCode::BadValue)
            .expect("the stray priority is reported");
        assert_eq!(named.repair, Some(Repair::Clear("priority")));
        Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    clear: vec!["priority".to_owned()],
                    ..Edit::default()
                },
                TODAY,
            )
            .unwrap();
        assert!(Notebook::new(&mut storage).check().unwrap().is_empty());
    }

    /// The reports the archive move carried are retired history and stay
    /// history: restore brings back the record alone.
    #[test]
    fn a_carried_report_stays_archived_when_its_record_comes_back() {
        let mut storage = storage_with(&[
            (
                "archive/tasks/task.demo.md",
                &task_file("closed", &["link: note note.report"]),
            ),
            (
                "archive/notes/note.report.md",
                &record_file(
                    "note.report",
                    "note",
                    "retired",
                    &["from: task.demo"],
                    "What the work came to.\n",
                ),
            ),
        ]);
        Notebook::new(&mut storage).restore("task.demo").unwrap();
        assert!(
            storage.read("archive/notes/note.report.md").is_ok(),
            "the report is history and stays filed"
        );
        assert!(matches!(
            storage.read("notes/note.report.md"),
            Err(StorageError::NotFound { .. })
        ));
    }

    #[test]
    fn a_replayed_restore_answers_already_and_changes_nothing() {
        let text = task_file("closed", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        let replay = Notebook::new(&mut storage).restore("task.demo").unwrap();
        assert!(replay.already);
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
    }

    #[test]
    fn an_id_neither_home_holds_is_unknown() {
        let mut storage = storage_with(&[]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .restore("task.ghost")
                .unwrap_err(),
            NotebookError::UnknownId { .. }
        ));
    }

    #[test]
    fn a_malformed_id_is_refused_as_an_argument() {
        let mut storage = storage_with(&[]);
        assert!(matches!(
            Notebook::new(&mut storage).restore("no-dot").unwrap_err(),
            NotebookError::InvalidArgument { .. }
        ));
    }

    /// Identical interrupted copies can be reconciled without choosing
    /// between histories, including when the record needs later repair.
    #[test]
    fn an_interrupted_move_is_finished_by_the_next_call() {
        let text = task_file("open", &["priority: 9"]);
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &text),
            ("archive/tasks/task.demo.md", &text),
        ]);
        let moved = Notebook::new(&mut storage).restore("task.demo").unwrap();
        assert!(!moved.already);
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
        assert!(matches!(
            storage.read("archive/tasks/task.demo.md"),
            Err(StorageError::NotFound { .. })
        ));
    }

    #[test]
    fn a_live_correction_keeps_both_copies_until_they_are_reconciled() {
        let corrected = record_file("task.demo", "task", "closed", &["priority: 2"], "");
        let original = task_file("closed", &[]);
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &corrected),
            ("archive/tasks/task.demo.md", &original),
        ]);
        assert!(matches!(
            Notebook::new(&mut storage).restore("task.demo"),
            Err(NotebookError::DuplicateId { .. })
        ));
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), corrected);
        assert_eq!(
            storage.read("archive/tasks/task.demo.md").unwrap(),
            original
        );
    }

    /// The move lands its copy before it removes its source, so a failure
    /// between the two leaves a loud duplicate-id, never a lost record.
    #[test]
    fn a_move_interrupted_after_its_write_loses_nothing() {
        let text = task_file("closed", &[]);
        let mut storage = RemoveFails(storage_with(&[("archive/tasks/task.demo.md", &text)]));
        assert!(Notebook::new(&mut storage).restore("task.demo").is_err());
        assert_eq!(
            storage.read("tasks/task.demo.md").unwrap(),
            text,
            "the copy landed before the failure"
        );
        assert_eq!(storage.read("archive/tasks/task.demo.md").unwrap(), text);
    }

    /// A file declaring another id has different bytes and keeps its copy.
    #[test]
    fn a_leftover_declaring_another_record_is_not_removed() {
        let foreign = record_file("task.other", "task", "closed", &[], "");
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("closed", &[])),
            ("archive/tasks/task.demo.md", &foreign),
        ]);
        let refusal = Notebook::new(&mut storage)
            .restore("task.demo")
            .unwrap_err();
        assert!(
            matches!(refusal, NotebookError::DuplicateId { .. }),
            "{refusal:?}"
        );
        assert_eq!(storage.read("archive/tasks/task.demo.md").unwrap(), foreign);
        assert!(storage.read("tasks/task.demo.md").is_ok());
    }

    /// A foreign destination also blocks restore without losing either file.
    #[test]
    fn a_destination_held_by_bytes_the_move_cannot_call_its_own_is_refused() {
        let foreign = record_file("task.other", "task", "closed", &[], "");
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &foreign),
            ("archive/tasks/task.demo.md", &task_file("closed", &[])),
        ]);
        let refusal = Notebook::new(&mut storage)
            .restore("task.demo")
            .unwrap_err();
        assert!(
            matches!(
                refusal,
                NotebookError::DuplicateId { ref id, ref holder }
                    if id == "task.demo" && holder == "tasks/task.demo.md"
            ),
            "{refusal:?}"
        );
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), foreign);
        assert!(storage.read("archive/tasks/task.demo.md").is_ok());
    }

    #[test]
    fn an_unreadable_archived_source_refuses_the_move_as_an_invalid_record() {
        let storage = &mut BinaryHolding::with_binary_at("archive/tasks/task.demo.md", &[]);
        let refusal = Notebook::new(storage).restore("task.demo").unwrap_err();
        match refusal {
            NotebookError::InvalidRecord { path, findings } => {
                assert_eq!(path, "archive/tasks/task.demo.md");
                assert_eq!(
                    findings.iter().map(|f| f.code).collect::<Vec<_>>(),
                    vec![FindingCode::NotUtf8]
                );
            }
            other => panic!("bytes that cannot cross the seam cannot move: {other:?}"),
        }
    }

    #[test]
    fn an_unreadable_live_holder_refuses_the_move() {
        let text = task_file("closed", &[]);
        let storage = &mut BinaryHolding::with_binary_at(
            "tasks/task.demo.md",
            &[("archive/tasks/task.demo.md", &text)],
        );
        let refusal = Notebook::new(storage).restore("task.demo").unwrap_err();
        assert!(
            matches!(
                refusal,
                NotebookError::DuplicateId { ref holder, .. } if holder == "tasks/task.demo.md"
            ),
            "{refusal:?}"
        );
        assert_eq!(storage.read("archive/tasks/task.demo.md").unwrap(), text);
    }

    /// With nothing to move, the unreadable live file is the record
    /// itself: `already` over bytes no parse can read would report a
    /// corruption as a success.
    #[test]
    fn a_replay_over_an_unreadable_live_record_is_refused() {
        let storage = &mut BinaryHolding::with_binary_at("tasks/task.demo.md", &[]);
        let refusal = Notebook::new(storage).restore("task.demo").unwrap_err();
        match refusal {
            NotebookError::InvalidRecord { path, findings } => {
                assert_eq!(path, "tasks/task.demo.md");
                assert_eq!(
                    findings.iter().map(|f| f.code).collect::<Vec<_>>(),
                    vec![FindingCode::NotUtf8]
                );
            }
            other => panic!("the refusal must carry the finding, got {other:?}"),
        }
    }

    /// The residence finding is the one thing `restore` erases, so it is
    /// the one finding on an archived file that names it; what is broken
    /// deeper inside gets its repair once the record is back.
    #[test]
    fn check_names_restore_on_the_residence_finding_alone() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.demo.md",
            &task_file("open", &["priority: 9"]),
        )]);
        let findings = Notebook::new(&mut storage).check().unwrap();
        let repair_of = |code: FindingCode| {
            findings
                .iter()
                .find(|located| located.finding.code == code)
                .unwrap_or_else(|| panic!("{code} must be reported"))
                .repair
                .clone()
        };
        assert_eq!(
            repair_of(FindingCode::ArchivedLiveRecord),
            Some(Repair::Restore)
        );
        assert_eq!(repair_of(FindingCode::BadValue), None);
    }
}
