mod listing {
    use crate::*;
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
    use crate::*;

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

mod search_query {
    use crate::*;

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
}

mod overview_query {
    use crate::*;

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
}
