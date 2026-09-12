mod listing {
    use crate::*;

    #[test]
    fn listing_and_ready_preserve_full_titles_for_the_host_to_render() {
        let wall_of_words = "word ".repeat(400);
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &format!(
                "---\nid: task.demo\ntype: task\nstate: open\ntitle: {wall_of_words}\ncreated: 2026-08-24\nupdated: 2026-08-25\n---\n"
            ),
        )]);
        let notebook = Notebook::new(&mut storage);

        let listed = notebook
            .list(&Filter::default())
            .unwrap()
            .pop()
            .unwrap()
            .title
            .unwrap();
        let queued = notebook
            .ready(&Filter::default())
            .unwrap()
            .pop()
            .unwrap()
            .title;
        for title in [listed, queued] {
            assert_eq!(title, wall_of_words.trim_end());
        }
    }

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
        let rows = Notebook::new(&mut storage)
            .list(&Filter::default())
            .unwrap();
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
                attribution: anb_core::Attribution::default(),
                title: Some("A demo record".into()),
            }
        );
        assert_eq!(
            rows[2].priority, None,
            "a record without the field carries none"
        );
    }
}

mod narrowed_by_identity {
    use super::holding;
    use crate::*;

    /// Work belongs to who holds it and authorship is a different fact:
    /// one person's records are the Tasks they hold and the records of
    /// every other type they wrote. A Task they wrote and handed over is
    /// the other person's, a Task they hold from someone else's planning
    /// is theirs, and a Task nobody holds is nobody's, whoever wrote it.
    fn a_team_notebook() -> MemoryStorage {
        storage_with(&[
            (
                "tasks/task.mine.md",
                &record_file("task.mine", "task", "open", &["by: Ada"], ""),
            ),
            (
                "tasks/task.taken.md",
                &record_file(
                    "task.taken",
                    "task",
                    "open",
                    &["by: Grace", "taken-by: Ada"],
                    "",
                ),
            ),
            (
                "tasks/task.given-away.md",
                &record_file(
                    "task.given-away",
                    "task",
                    "open",
                    &["by: Ada", "taken-by: Grace"],
                    "",
                ),
            ),
            (
                "tasks/task.theirs.md",
                &record_file("task.theirs", "task", "open", &["by: Grace"], ""),
            ),
            (
                "decisions/decision.mine.md",
                &record_file("decision.mine", "decision", "active", &["by: Ada"], ""),
            ),
            (
                "decisions/decision.nobodys.md",
                &record_file("decision.nobodys", "decision", "active", &[], ""),
            ),
        ])
    }

    fn by(identity: &str) -> Filter {
        Filter {
            by: Some(identity.to_owned()),
            ..Filter::default()
        }
    }

    #[test]
    fn a_listing_narrowed_to_one_identity_keeps_the_tasks_it_holds_and_what_it_wrote() {
        let mut storage = a_team_notebook();
        let notebook = Notebook::new(&mut storage);
        let listed: Vec<String> = notebook
            .list(&by("Ada"))
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(listed, ["task.taken", "decision.mine"]);
        let queued: Vec<String> = notebook
            .ready(&by("Grace"))
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(queued, ["task.given-away"]);
    }

    /// What waits on a person is theirs too: the Question put to them and
    /// the review handed to them, whoever holds or wrote it. An open Task
    /// whose addressee is set ahead of its submit waits on nobody yet and
    /// is not theirs; `to` alone reaches every record addressed to them.
    #[test]
    fn a_listing_narrowed_to_one_identity_keeps_what_waits_on_them() {
        let mut storage = storage_with(&[
            (
                "tasks/task.her-review.md",
                &record_file(
                    "task.her-review",
                    "task",
                    "review",
                    &["taken-by: Grace", "to: Ada"],
                    "",
                ),
            ),
            (
                "tasks/task.her-plan.md",
                &record_file(
                    "task.her-plan",
                    "task",
                    "open",
                    &["taken-by: Grace", "to: Ada"],
                    "",
                ),
            ),
            (
                "questions/question.put-to-ada.md",
                &record_file(
                    "question.put-to-ada",
                    "question",
                    "open",
                    &["by: Grace", "to: Ada"],
                    "",
                ),
            ),
            (
                "questions/question.put-to-grace.md",
                &record_file(
                    "question.put-to-grace",
                    "question",
                    "open",
                    &["by: Ada", "to: Grace"],
                    "",
                ),
            ),
        ]);
        let notebook = Notebook::new(&mut storage);
        let ids = |filter: &Filter| -> Vec<String> {
            notebook
                .list(filter)
                .unwrap()
                .into_iter()
                .map(|row| row.id)
                .collect()
        };
        assert_eq!(
            ids(&by("Ada")),
            [
                "task.her-review",
                "question.put-to-ada",
                "question.put-to-grace"
            ]
        );
        assert_eq!(
            ids(&Filter {
                to: Some("Ada".to_owned()),
                ..Filter::default()
            }),
            ["task.her-plan", "task.her-review", "question.put-to-ada"]
        );
        let row = notebook
            .list(&by("Ada"))
            .unwrap()
            .into_iter()
            .find(|row| row.id == "task.her-review")
            .unwrap();
        assert_eq!(row.attribution.to.as_deref(), Some("Ada"));
    }

    /// The pool is the Tasks nobody holds, whoever wrote them: the work
    /// anyone may take. A Decision nobody signed is not in it, since only
    /// work is taken.
    #[test]
    fn the_pool_is_the_tasks_nobody_holds() {
        let mut storage = a_team_notebook();
        let notebook = Notebook::new(&mut storage);
        let pool = Filter {
            untaken: true,
            ..Filter::default()
        };
        let listed: Vec<String> = notebook
            .list(&pool)
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(listed, ["task.mine", "task.theirs"]);
        let queued: Vec<String> = notebook
            .ready(&pool)
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(queued, ["task.mine", "task.theirs"]);
    }

    /// A row carries who it names, so a reader of the listing sees who took
    /// a Task without opening the record.
    #[test]
    fn a_row_carries_its_creator_and_who_took_it() {
        let mut storage = a_team_notebook();
        let rows = Notebook::new(&mut storage)
            .ready(&Filter::default())
            .unwrap();
        let taken = rows.iter().find(|row| row.id == "task.taken").unwrap();
        assert_eq!(
            taken.attribution,
            anb_core::Attribution {
                by: Some("Grace".to_owned()),
                taken_by: Some("Ada".to_owned()),
                to: None,
            }
        );
    }

    #[test]
    fn the_scope_and_the_identity_narrow_together() {
        let mut storage = storage_with(&[
            (
                "tasks/task.hub.md",
                &record_file(
                    "task.hub",
                    "task",
                    "open",
                    &["blocked-by: task.child-a", "blocked-by: task.child-b"],
                    "",
                ),
            ),
            (
                "tasks/task.child-a.md",
                &record_file(
                    "task.child-a",
                    "task",
                    "open",
                    &["from: task.hub", "taken-by: Ada"],
                    "",
                ),
            ),
            (
                "tasks/task.child-b.md",
                &record_file(
                    "task.child-b",
                    "task",
                    "open",
                    &["from: task.hub", "taken-by: Grace"],
                    "",
                ),
            ),
            (
                "tasks/task.elsewhere.md",
                &record_file("task.elsewhere", "task", "open", &["taken-by: Ada"], ""),
            ),
        ]);
        let ids: Vec<String> = Notebook::new(&mut storage)
            .ready(&Filter {
                hub: Some("task.hub".to_owned()),
                by: Some("Ada".to_owned()),
                ..Filter::default()
            })
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(ids, ["task.child-a"]);
    }

    #[test]
    fn a_search_finds_a_record_by_the_people_it_names() {
        let mut storage = storage_with(&[
            (
                "tasks/task.taken.md",
                &record_file(
                    "task.taken",
                    "task",
                    "open",
                    &["by: Grace Hopper", "via: codex", "taken-by: Ada"],
                    "",
                ),
            ),
            (
                "tasks/task.other.md",
                &record_file("task.other", "task", "open", &[], ""),
            ),
        ]);
        let notebook = Notebook::new(&mut storage);
        for needle in ["hopper", "codex", "ada"] {
            let ids: Vec<String> = notebook
                .list(&holding(needle))
                .unwrap()
                .into_iter()
                .map(|row| row.id)
                .collect();
            assert_eq!(ids, ["task.taken"], "{needle}");
        }
    }
}

/// The listing narrowed to the records whose text holds `needle`.
/// A schema Note and the documents that declare it theirs through a
/// `link` line, beside one that only names it in prose, one that links a
/// path and one filed away: the smallest notebook that tells a link edge
/// from a mention.
fn a_schema_and_its_documents() -> crate::MemoryStorage {
    let note = |id: &str, extra: &[&str], body: &str| {
        crate::record_file(id, "note", "active", extra, body)
    };
    crate::storage_with(&[
        (
            "notes/note.schema.md",
            &note(
                "note.schema",
                &["kind: spec"],
                "Every primitive has an owner.\n",
            ),
        ),
        (
            "notes/note.credits.md",
            &note(
                "note.credits",
                &["kind: term", "link: schema note.schema"],
                "```yaml\nschema: note.schema\n```\n",
            ),
        ),
        (
            "notes/note.tenancy.md",
            &note(
                "note.tenancy",
                &["kind: term", "link: schema note.schema"],
                "",
            ),
        ),
        (
            "notes/note.sample.md",
            &note(
                "note.sample",
                &[
                    "kind: fact",
                    "link: example note.schema",
                    "link: doc docs/schema.md",
                ],
                "",
            ),
        ),
        (
            "notes/note.prose.md",
            &note(
                "note.prose",
                &["kind: fact"],
                "Reads note.schema for the shape.\n",
            ),
        ),
        (
            "notes/note.glossary.md",
            &note(
                "note.glossary",
                &["kind: term", "link: within note.credits"],
                "",
            ),
        ),
        (
            "archive/notes/note.old.md",
            &crate::record_file(
                "note.old",
                "note",
                "retired",
                &["kind: term", "link: schema note.schema"],
                "",
            ),
        ),
    ])
}

fn holding(needle: &str) -> anb_core::Filter {
    anb_core::Filter {
        text: Some(needle.to_owned()),
        ..anb_core::Filter::default()
    }
}

mod narrowed_by_kind_tag_and_type {
    use crate::*;

    fn ids(rows: Vec<anb_core::ListedRecord>) -> Vec<String> {
        rows.into_iter().map(|row| row.id).collect()
    }

    /// Two rules, a shape and a term, each tagged for the parser or the
    /// stack: the smallest notebook that tells the four narrowings apart.
    fn a_knowledge_notebook() -> MemoryStorage {
        storage_with(&[
            (
                "decisions/decision.nest.md",
                &record_file(
                    "decision.nest",
                    "decision",
                    "active",
                    &["kind: rule", "tags: parser, grammar"],
                    "",
                ),
            ),
            (
                "decisions/decision.rust.md",
                &record_file(
                    "decision.rust",
                    "decision",
                    "active",
                    &["kind: rule", "tags: stack"],
                    "",
                ),
            ),
            (
                "decisions/decision.shape.md",
                &record_file(
                    "decision.shape",
                    "decision",
                    "active",
                    &["kind: shape", "tags: parser"],
                    "",
                ),
            ),
            (
                "notes/note.fence.md",
                &record_file(
                    "note.fence",
                    "note",
                    "active",
                    &["kind: term", "tags: parser, grammar"],
                    "",
                ),
            ),
            ("tasks/task.demo.md", &task_file("open", &["tags: parser"])),
        ])
    }

    #[test]
    fn a_type_keeps_the_records_of_that_type_and_two_types_keep_both() {
        let mut storage = a_knowledge_notebook();
        let notebook = Notebook::new(&mut storage);
        let notes = Filter {
            types: vec![RecordType::Note],
            ..Filter::default()
        };
        assert_eq!(ids(notebook.list(&notes).unwrap()), ["note.fence"]);
        let work_and_notes = Filter {
            types: vec![RecordType::Task, RecordType::Note],
            ..Filter::default()
        };
        assert_eq!(
            ids(notebook.list(&work_and_notes).unwrap()),
            ["task.demo", "note.fence"]
        );
    }

    /// Every standing rule is one narrowing: the kind alone, since only a
    /// Decision carries it, and the type beside it when a reader spells the
    /// question out.
    #[test]
    fn a_kind_keeps_the_records_of_that_kind() {
        let mut storage = a_knowledge_notebook();
        let notebook = Notebook::new(&mut storage);
        let rules = Filter {
            kinds: vec!["rule".to_owned()],
            ..Filter::default()
        };
        assert_eq!(
            ids(notebook.list(&rules).unwrap()),
            ["decision.nest", "decision.rust"]
        );
        let rules_spelled_out = Filter {
            types: vec![RecordType::Decision],
            ..rules
        };
        assert_eq!(
            ids(notebook.list(&rules_spelled_out).unwrap()),
            ["decision.nest", "decision.rust"]
        );
    }

    /// One tag keeps every record carrying it; two tags keep the records
    /// carrying both, since every narrowing is an intersection.
    #[test]
    fn tags_narrow_to_the_records_carrying_every_one_of_them() {
        let mut storage = a_knowledge_notebook();
        let notebook = Notebook::new(&mut storage);
        let parser = Filter {
            tags: vec!["parser".to_owned()],
            ..Filter::default()
        };
        assert_eq!(
            ids(notebook.list(&parser).unwrap()),
            ["task.demo", "decision.nest", "decision.shape", "note.fence"]
        );
        let parser_grammar = Filter {
            tags: vec!["parser".to_owned(), "grammar".to_owned()],
            ..Filter::default()
        };
        assert_eq!(
            ids(notebook.list(&parser_grammar).unwrap()),
            ["decision.nest", "note.fence"]
        );
    }

    #[test]
    fn the_narrowings_compose_as_one_intersection() {
        let mut storage = a_knowledge_notebook();
        let parser_rules = Filter {
            types: vec![RecordType::Decision],
            kinds: vec!["rule".to_owned()],
            tags: vec!["parser".to_owned()],
            ..Filter::default()
        };
        assert_eq!(
            ids(Notebook::new(&mut storage).list(&parser_rules).unwrap()),
            ["decision.nest"]
        );
    }

    /// The queue takes the same narrowing: a tag reaches the Tasks
    /// carrying it, and a kind reaches nothing, since no Task carries one.
    #[test]
    fn the_queue_narrows_by_the_same_filter() {
        let mut storage = a_knowledge_notebook();
        let notebook = Notebook::new(&mut storage);
        let parser = Filter {
            tags: vec!["parser".to_owned()],
            ..Filter::default()
        };
        let queued: Vec<String> = notebook
            .ready(&parser)
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(queued, ["task.demo"]);
        let rules = Filter {
            kinds: vec!["rule".to_owned()],
            ..Filter::default()
        };
        assert_eq!(notebook.ready(&rules).unwrap(), vec![]);
    }

    /// A word no type allows as a kind, and a tag the grammar rejects, can
    /// match no record: an empty listing would read as a notebook holding
    /// none, so the filter is refused with the vocabulary named.
    #[test]
    fn a_kind_no_type_allows_and_a_malformed_tag_are_refused() {
        let mut storage = a_knowledge_notebook();
        let notebook = Notebook::new(&mut storage);
        let law = Filter {
            kinds: vec!["law".to_owned()],
            ..Filter::default()
        };
        let Err(NotebookError::InvalidArgument { reason }) = notebook.list(&law) else {
            panic!("a kind no type allows must be refused");
        };
        assert!(reason.contains("rule, shape, drift, fact"), "{reason}");
        let shouted = Filter {
            tags: vec!["Parser".to_owned()],
            ..Filter::default()
        };
        assert!(matches!(
            notebook.ready(&shouted),
            Err(NotebookError::InvalidArgument { .. })
        ));
    }
}

mod record_view {
    use super::a_schema_and_its_documents;
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

    /// A link whose target is a record id relates the two records, and
    /// the target can read the relation back: each carrier under the kind
    /// its line gives, kind by kind so one relation's members stand
    /// together. A mention in prose stays a mention, a link to a path
    /// relates no record, and a carrier filed away is history.
    #[test]
    fn linked_by_lists_the_live_records_whose_link_lines_name_it_by_kind() {
        let mut storage = a_schema_and_its_documents();
        let view = Notebook::new(&mut storage).view("note.schema").unwrap();
        assert_eq!(
            view.linked_by,
            vec![
                ("example".to_owned(), "note.sample".to_owned()),
                ("schema".to_owned(), "note.credits".to_owned()),
                ("schema".to_owned(), "note.tenancy".to_owned()),
            ]
        );
        assert_eq!(view.mentioned_by, vec!["note.prose"]);
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

    /// One reply must not group a record two ways: the counts line and the
    /// section that shows it are read together, and a record in the wrong
    /// directory is a state `check` names and a reader can see.
    #[test]
    fn the_counts_group_a_record_where_the_sections_show_it() {
        let mut storage = storage_with(&[
            (
                "tasks/note.misplaced.md",
                "---\nid: note.misplaced\ntype: note\nstate: active\ncreated: 2026-08-01\nupdated: 2026-08-01\ntitle: A note living in tasks\n---\n\nB.\n",
            ),
            (
                "tasks/task.real.md",
                "---\nid: task.real\ntype: task\nstate: open\ncreated: 2026-08-01\nupdated: 2026-08-01\ntitle: Real\n---\n\nB.\n",
            ),
        ]);
        let notebook = Notebook::new(&mut storage);
        let counts = notebook
            .status(TODAY, Budget::Unbounded, None, no_lost_proofs)
            .unwrap()
            .counts;
        assert_eq!(counts.tasks, 2, "both files sit under tasks/");
        assert_eq!(counts.notes, 0);
        let listed = notebook.list(&Filter::default()).unwrap();
        assert_eq!(
            listed.len(),
            2,
            "the listing shows every file the counts count: {listed:?}"
        );
    }
}

mod narrowed_by_text {
    use super::holding;
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
    fn a_text_matches_titles_case_insensitively() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let rows = Notebook::new(&mut storage)
            .list(&holding("DEMO RECORD"))
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "task.demo");
    }

    #[test]
    fn a_text_matches_bodies_ids_and_tags() {
        let mut storage = demo_notebook();
        let notebook = Notebook::new(&mut storage);
        let ids = |rows: Vec<anb_core::ListedRecord>| {
            rows.into_iter().map(|row| row.id).collect::<Vec<_>>()
        };
        assert_eq!(
            ids(notebook.list(&holding("grammar")).unwrap()),
            vec!["task.parser"],
            "the body is a matched surface"
        );
        assert_eq!(
            ids(notebook.list(&holding("stack")).unwrap()),
            vec!["decision.rust"],
            "tags are a matched surface"
        );
        let history = Filter {
            archive: true,
            ..holding("task.spike")
        };
        assert_eq!(
            ids(notebook.list(&history).unwrap()),
            vec!["task.spike"],
            "the id is a matched surface"
        );
    }

    /// History is findable when asked for: the archive joins the listing
    /// with `archive`, and live rows lead within a type.
    #[test]
    fn the_archive_is_matched_when_asked_for() {
        let mut storage = demo_notebook();
        let notebook = Notebook::new(&mut storage);
        let live: Vec<String> = notebook
            .list(&holding("parser"))
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(live, ["task.parser"]);
        let history = Filter {
            archive: true,
            ..holding("parser")
        };
        let ids: Vec<String> = notebook
            .list(&history)
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(ids, ["task.parser", "task.spike"]);
    }

    #[test]
    fn an_empty_text_is_refused() {
        let mut storage = storage_with(&[]);
        assert!(matches!(
            Notebook::new(&mut storage)
                .list(&holding("  "))
                .unwrap_err(),
            NotebookError::InvalidArgument { .. }
        ));
    }
}

/// The map: every Task a slice reaches, with the edges it draws.
mod task_map {
    use super::a_schema_and_its_documents;
    use crate::*;
    use anb_core::{EdgeKind, Focus, GraphSlice};

    fn task(id: &str, state: &str, extra: &[&str], body: &str) -> String {
        record_file(id, "task", state, extra, body)
    }

    fn whole() -> GraphSlice {
        GraphSlice::default()
    }

    fn with_archive() -> GraphSlice {
        GraphSlice {
            filter: Filter {
                archive: true,
                ..Filter::default()
            },
            ..GraphSlice::default()
        }
    }

    fn inside(hub: &str) -> GraphSlice {
        GraphSlice {
            filter: within(hub),
            ..GraphSlice::default()
        }
    }

    fn around(id: &str, depth: usize) -> GraphSlice {
        GraphSlice {
            focus: Some(Focus {
                id: id.to_owned(),
                depth,
            }),
            ..GraphSlice::default()
        }
    }

    fn map_of(storage: &mut MemoryStorage, slice: &GraphSlice) -> Vec<String> {
        Notebook::new(storage)
            .graph(slice)
            .unwrap()
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect()
    }

    /// The graph is of the notebook, not of its queue: a Decision is as
    /// much a record as the Task that cites it, and a reader drawing the
    /// notebook needs both.
    #[test]
    fn every_type_of_record_is_a_node() {
        assert_eq!(
            map_of(&mut a_task_and_a_decision(), &whole()),
            vec!["task.open", "decision.rule"]
        );
    }

    /// The types are a slice a caller asks for, so one type can be had
    /// without losing the others from the vocabulary.
    #[test]
    fn a_type_asked_for_is_the_only_type_drawn() {
        let only_decisions = GraphSlice {
            filter: Filter {
                types: vec![RecordType::Decision],
                ..Filter::default()
            },
            ..whole()
        };

        assert_eq!(
            map_of(&mut a_task_and_a_decision(), &only_decisions),
            vec!["decision.rule"]
        );
    }

    /// One Task and one Decision: the smallest notebook that can tell a
    /// graph of the work from a graph of the notebook.
    fn a_task_and_a_decision() -> MemoryStorage {
        storage_with(&[
            ("tasks/task.open.md", &task("task.open", "open", &[], "")),
            (
                "decisions/decision.rule.md",
                &record_file("decision.rule", "decision", "active", &["kind: rule"], ""),
            ),
        ])
    }

    /// The reader asks for history when they want it.
    #[test]
    fn finished_work_stays_off_the_map_until_it_is_asked_for() {
        let mut storage = storage_with(&[
            ("tasks/task.open.md", &task("task.open", "open", &[], "")),
            (
                "archive/tasks/task.done.md",
                &task("task.done", "closed", &[], ""),
            ),
        ]);

        assert_eq!(map_of(&mut storage, &whole()), vec!["task.open"]);
        assert_eq!(
            map_of(&mut storage, &with_archive()),
            vec!["task.open", "task.done"]
        );
    }

    /// A filed tile is history, and a reader has to be able to see which
    /// tiles are.
    #[test]
    fn a_filed_task_says_so_and_keeps_the_state_it_settled_in() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.done.md",
            &task("task.done", "closed", &[], ""),
        )]);

        let node = Notebook::new(&mut storage)
            .graph(&with_archive())
            .unwrap()
            .nodes
            .pop()
            .unwrap();
        assert!(node.archived);
        assert_eq!(node.state, "closed");
    }

    /// An interrupted archive move leaves one id claiming two files. The
    /// map draws one tile for it, the live file's, as every other query
    /// reads it.
    #[test]
    fn an_id_claiming_two_files_is_one_tile_and_the_live_one_wins() {
        let mut storage = storage_with(&[
            ("tasks/task.torn.md", &task("task.torn", "active", &[], "")),
            (
                "archive/tasks/task.torn.md",
                &task("task.torn", "closed", &[], ""),
            ),
        ]);

        let nodes = Notebook::new(&mut storage)
            .graph(&with_archive())
            .unwrap()
            .nodes;
        assert_eq!(nodes.len(), 1, "one id is one tile: {nodes:?}");
        assert_eq!(nodes[0].state, "active");
        assert!(!nodes[0].archived);
    }

    /// A record its own findings exclude is out of every derived query, and
    /// a map that dropped it would be the one surface where a reader could
    /// not see that something is wrong.
    #[test]
    fn an_invalid_task_keeps_its_tile_and_says_it_is_invalid() {
        let mut storage =
            storage_with(&[("tasks/task.demo.md", &task("task.demo", "bogus", &[], ""))]);

        let node = Notebook::new(&mut storage)
            .graph(&whole())
            .unwrap()
            .nodes
            .pop()
            .unwrap();
        assert_eq!(node.state, "invalid");
    }

    /// The epic branch: the hub, what it waits on however far down that
    /// goes, and what was born inside it — the closed children included,
    /// since the hub's counter is over all of them.
    #[test]
    fn the_epic_branch_holds_the_hub_its_children_and_the_closed_ones() {
        let mut storage = an_epic();
        let branch = GraphSlice {
            filter: Filter {
                archive: true,
                ..within("task.hub")
            },
            ..GraphSlice::default()
        };

        let mut drawn = map_of(&mut storage, &branch);
        drawn.sort();
        assert_eq!(drawn, vec!["task.done", "task.hub", "task.open"]);
        assert!(
            !drawn.contains(&"task.outside".to_owned()),
            "a Task the epic never reached is another epic's"
        );
    }

    /// Narrowing changes what a map shows, never what it counts: a hub's
    /// progress is settled against the whole notebook, so leaving the
    /// closed children off the picture does not undo them.
    #[test]
    fn a_hub_counts_children_the_slice_left_off_the_map() {
        let mut storage = an_epic();
        let nodes = Notebook::new(&mut storage)
            .graph(&inside("task.hub"))
            .unwrap()
            .nodes;

        assert_eq!(
            nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["task.hub", "task.open"],
            "the closed child is filed, and the archive was not asked for"
        );
        let hub = nodes.iter().find(|node| node.id == "task.hub").unwrap();
        let epic = hub.epic.as_ref().expect("the hub is an epic");
        assert_eq!((epic.closed, epic.total), (1, 2));
        assert!(
            nodes
                .iter()
                .filter(|node| node.id != "task.hub")
                .all(|node| node.epic.is_none()),
            "a plain Task carries no counter"
        );
    }

    /// Every narrowing is a predicate over the same notebook, so asking for
    /// two asks for the intersection: one person's work inside one epic.
    #[test]
    fn an_identity_narrows_inside_an_epic_branch() {
        let mut storage = an_epic();
        let hers_inside = GraphSlice {
            filter: Filter {
                by: Some("Grace".to_owned()),
                ..within("task.hub")
            },
            ..GraphSlice::default()
        };
        assert_eq!(map_of(&mut storage, &hers_inside), vec!["task.open"]);
    }

    /// A line has to land on a tile, so an edge whose far end the slice
    /// left off the map is not one.
    #[test]
    fn an_edge_off_the_map_is_not_drawn() {
        let mut storage = an_epic();
        let children_only = GraphSlice {
            filter: Filter {
                by: Some("Grace".to_owned()),
                ..Filter::default()
            },
            ..GraphSlice::default()
        };
        let graph = Notebook::new(&mut storage).graph(&children_only).unwrap();
        assert_eq!(
            map_of(&mut storage, &children_only),
            vec!["task.open", "task.outside"],
            "the hub is nobody's"
        );
        assert_eq!(
            graph.edges().len(),
            0,
            "the child's edges land on the hub, which is off the map"
        );
    }

    /// A reader learns two strokes once: what a Task waits on, and what it
    /// was born from. Both run the way work becomes possible — out of what
    /// settles first, into what the settling releases.
    #[test]
    fn the_two_edges_a_record_declares_are_told_apart_and_run_one_way() {
        let mut storage = an_epic();
        let graph = Notebook::new(&mut storage)
            .graph(&inside("task.hub"))
            .unwrap();

        let drawn: Vec<(&str, &str, EdgeKind)> = graph
            .edges()
            .iter()
            .map(|edge| (edge.from, edge.to, edge.kind))
            .collect();
        assert!(drawn.contains(&("task.open", "task.hub", EdgeKind::BlockedBy)));
        assert!(drawn.contains(&("task.hub", "task.open", EdgeKind::Origin)));
    }

    /// A link to a record is an edge in the notebook's own vocabulary: it
    /// carries the link's kind and runs the way it was written, out of the
    /// record that declares it. A body naming the same record is the same
    /// relation stated twice, and the declared word survives.
    #[test]
    fn a_link_to_a_record_is_an_edge_carrying_the_links_own_kind() {
        let mut storage = a_schema_and_its_documents();
        let graph = Notebook::new(&mut storage).graph(&whole()).unwrap();
        let drawn: Vec<(&str, &str, EdgeKind)> = graph
            .edges()
            .iter()
            .map(|edge| (edge.from, edge.to, edge.kind))
            .collect();
        assert!(drawn.contains(&("note.credits", "note.schema", EdgeKind::Link("schema"))));
        assert!(drawn.contains(&("note.sample", "note.schema", EdgeKind::Link("example"))));
        assert!(drawn.contains(&("note.prose", "note.schema", EdgeKind::Mentions)));
        assert!(
            !drawn.contains(&("note.credits", "note.schema", EdgeKind::Mentions)),
            "a quoted id in the body is no mention, and a bare one would be the same pair"
        );
        assert_eq!(EdgeKind::Link("schema").word(), "schema");
    }

    /// A link's kind is the notebook's own word, never one of the three the
    /// graph draws itself, since the edge would then read as that relation;
    /// and a relation has two ends, so a record cannot link itself, and a
    /// hand-written self-link draws no edge.
    #[test]
    fn a_link_spelled_like_a_drawn_relation_or_pointing_at_itself_is_no_edge() {
        let mut storage = storage_with(&[
            ("tasks/task.first.md", &task("task.first", "open", &[], "")),
            (
                "notes/note.self.md",
                &record_file(
                    "note.self",
                    "note",
                    "active",
                    &["link: schema note.self"],
                    "",
                ),
            ),
        ]);
        for word in ["waits", "born", "mentions"] {
            let refused = Notebook::new(&mut storage).edit(
                "task.first",
                &Edit {
                    add_links: vec![Link {
                        kind: word.to_owned(),
                        target: "note.self".to_owned(),
                    }],
                    ..Edit::default()
                },
                TODAY,
            );
            assert!(
                matches!(refused, Err(NotebookError::InvalidArgument { .. })),
                "{word}: {refused:?}"
            );
        }
        let refused = Notebook::new(&mut storage).edit(
            "task.first",
            &Edit {
                add_links: vec![Link {
                    kind: "schema".to_owned(),
                    target: "task.first".to_owned(),
                }],
                ..Edit::default()
            },
            TODAY,
        );
        assert!(matches!(
            refused,
            Err(NotebookError::InvalidArgument { .. })
        ));
        let graph = Notebook::new(&mut storage).graph(&whole()).unwrap();
        assert!(graph.edges().is_empty(), "{:?}", graph.edges());
        assert_eq!(graph.degrees().get("note.self"), Some(&0));
    }

    #[test]
    fn membership_follows_origins_while_focus_also_follows_contextual_links() {
        let mut storage = a_schema_and_its_documents();
        assert_eq!(
            map_of(&mut storage, &inside("note.schema")),
            vec!["note.schema"]
        );
        assert_eq!(
            map_of(&mut storage, &around("note.schema", 1)),
            vec![
                "note.credits",
                "note.prose",
                "note.sample",
                "note.schema",
                "note.tenancy"
            ]
        );
        let listed: Vec<String> = Notebook::new(&mut storage)
            .list(&within("note.credits"))
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(listed, vec!["note.credits"]);
    }

    /// A web reads by weight, and weight is how much of the slice meets at
    /// a tile.
    #[test]
    fn a_tile_carries_how_many_lines_meet_at_it() {
        let mut storage = an_epic();
        let graph = Notebook::new(&mut storage)
            .graph(&inside("task.hub"))
            .unwrap();

        let degrees = graph.degrees();
        assert_eq!(degrees.get("task.hub"), Some(&2));
        assert_eq!(degrees.get("task.open"), Some(&2));
    }

    #[test]
    fn a_hub_the_notebook_does_not_hold_is_refused() {
        let mut storage = an_epic();
        assert!(matches!(
            Notebook::new(&mut storage).graph(&inside("task.absent")),
            Err(NotebookError::UnknownId { .. })
        ));
    }

    /// The focus is what lets a map answer for a notebook of thousands:
    /// one record, and the graph reaching out from it as far as asked.
    #[test]
    fn a_focus_holds_what_reaches_the_record_within_the_depth_asked_for() {
        let mut storage = a_chain();

        assert_eq!(
            map_of(&mut storage, &around("task.c", 1)),
            vec!["task.b", "task.c", "task.d"]
        );
        assert_eq!(
            map_of(&mut storage, &around("task.c", 2)),
            vec!["task.a", "task.b", "task.c", "task.d", "task.e"]
        );
    }

    #[test]
    fn a_focus_reaches_a_shared_blockers_other_task_at_two_edges() {
        let mut storage = a_chain();
        assert_eq!(
            map_of(&mut storage, &around("task.b", 1)),
            ["task.a", "task.b", "task.c"]
        );
        assert_eq!(
            map_of(&mut storage, &around("task.b", 2)),
            ["task.a", "task.b", "task.c", "task.d", "task.sibling"]
        );
    }

    /// A depth of nothing is the record alone, which is the honest answer
    /// to asking for no hops.
    #[test]
    fn a_focus_with_no_depth_is_the_record_alone() {
        let mut storage = a_chain();
        assert_eq!(map_of(&mut storage, &around("task.c", 0)), vec!["task.c"]);
    }

    /// A focus and another narrowing are two predicates over one notebook:
    /// the neighbourhood is walked whole, and the text keeps one of it.
    #[test]
    fn a_focus_narrows_with_the_filter_beside_it() {
        let mut storage = a_chain();
        let head_around = GraphSlice {
            filter: Filter {
                text: Some("task.a".to_owned()),
                ..Filter::default()
            },
            ..around("task.c", 2)
        };
        assert_eq!(
            map_of(&mut storage, &head_around),
            vec!["task.a"],
            "only the head of the chain holds the text"
        );
    }

    #[test]
    fn a_focus_the_notebook_does_not_hold_is_refused() {
        let mut storage = a_chain();
        assert!(matches!(
            Notebook::new(&mut storage).graph(&around("task.absent", 1)),
            Err(NotebookError::UnknownId { .. })
        ));
    }

    /// A focus the slice itself leaves off the map would answer an empty
    /// picture, which reads exactly like a notebook with nothing in it. The
    /// refusal names the flag that puts the record back.
    #[test]
    fn a_focus_on_filed_work_names_the_flag_that_draws_it() {
        let mut storage = storage_with(&[(
            "archive/tasks/task.done.md",
            &task("task.done", "closed", &[], ""),
        )]);

        let refused = Notebook::new(&mut storage)
            .graph(&around("task.done", 1))
            .expect_err("a focus off its own map draws nothing");

        let NotebookError::InvalidArgument { reason } = &refused else {
            panic!("a focus off the map is a refused argument: {refused:?}")
        };
        assert!(reason.contains("--archive"), "{reason}");
    }

    /// Every record is a node, so every record can be the centre of one.
    /// A reader asking what a Decision touches is asking the same question
    /// as a reader asking it of a Task.
    #[test]
    fn any_record_can_be_the_centre_of_a_neighbourhood() {
        let mut storage = storage_with(&[(
            "decisions/decision.rule.md",
            &record_file("decision.rule", "decision", "active", &["kind: rule"], ""),
        )]);

        assert_eq!(
            map_of(&mut storage, &around("decision.rule", 1)),
            vec!["decision.rule"]
        );
    }

    /// A hub with one child open and one closed and filed, plus a Task the
    /// epic never reached; the children and the stray are Grace's, the hub
    /// nobody's.
    fn an_epic() -> MemoryStorage {
        storage_with(&[
            (
                "tasks/task.hub.md",
                &task(
                    "task.hub",
                    "open",
                    &["blocked-by: task.open", "blocked-by: task.done"],
                    "",
                ),
            ),
            (
                "tasks/task.open.md",
                &task(
                    "task.open",
                    "open",
                    &["from: task.hub", "taken-by: Grace"],
                    "",
                ),
            ),
            (
                "archive/tasks/task.done.md",
                &task(
                    "task.done",
                    "closed",
                    &["from: task.hub", "taken-by: Grace"],
                    "",
                ),
            ),
            (
                "tasks/task.outside.md",
                &task("task.outside", "open", &["taken-by: Grace"], ""),
            ),
        ])
    }

    /// A line of work four deep, a Task hanging off the same blocker as the
    /// second link, and one the chain never touches.
    fn a_chain() -> MemoryStorage {
        storage_with(&[
            ("tasks/task.a.md", &task("task.a", "open", &[], "")),
            (
                "tasks/task.b.md",
                &task("task.b", "open", &["blocked-by: task.a"], ""),
            ),
            (
                "tasks/task.c.md",
                &task("task.c", "open", &["blocked-by: task.b"], ""),
            ),
            (
                "tasks/task.d.md",
                &task("task.d", "open", &["blocked-by: task.c"], ""),
            ),
            (
                "tasks/task.e.md",
                &task("task.e", "open", &["blocked-by: task.d"], ""),
            ),
            (
                "tasks/task.sibling.md",
                &task("task.sibling", "open", &["blocked-by: task.a"], ""),
            ),
        ])
    }
}
