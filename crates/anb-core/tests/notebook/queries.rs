mod listing {
    use crate::*;

    /// A listing row is derived, and a title is a record's own text: the
    /// row carries as much of it as a reply affords, whatever a hand wrote.
    #[test]
    fn a_long_title_is_cut_in_a_listing_row_and_in_the_ready_queue() {
        let wall_of_words = "word ".repeat(400);
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &format!(
                "---\nid: task.demo\ntype: task\nstate: open\ntitle: {wall_of_words}\ncreated: 2026-08-24\nupdated: 2026-08-25\n---\n"
            ),
        )]);
        let notebook = Notebook::new(&mut storage);

        let listed = notebook.list().unwrap().pop().unwrap().title.unwrap();
        let queued = notebook.ready().unwrap().pop().unwrap().title;
        for title in [listed, queued] {
            assert!(
                title.chars().count() <= anb_core::encode::TEXT_BOUND,
                "unbounded title: {title}"
            );
            assert!(title.contains("more characters"), "no hint: {title}");
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
        let overview = Notebook::new(&mut storage).overview().unwrap();

        assert_eq!(overview.live.tasks, 2, "both files sit under tasks/");
        assert_eq!(overview.live.notes, 0);
        for section in &overview.sections {
            let held = match section.record_type {
                RecordType::Task => overview.live.tasks,
                RecordType::Decision => overview.live.decisions,
                RecordType::Note => overview.live.notes,
                RecordType::Question => overview.live.questions,
            };
            assert_eq!(
                section.rows.len(),
                held,
                "the {:?} section and the {:?} count disagree",
                section.record_type,
                section.record_type
            );
        }
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

/// The map: every Task a slice reaches, with the edges it draws.
mod task_map {
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
            archive: true,
            ..GraphSlice::default()
        }
    }

    fn inside(hub: &str) -> GraphSlice {
        GraphSlice {
            hub: Some(hub.to_owned()),
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
            types: vec![RecordType::Decision],
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
            hub: Some("task.hub".to_owned()),
            ..with_archive()
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

    #[test]
    fn the_ready_lens_holds_what_can_be_started_now() {
        let mut storage = an_epic();
        let ready = GraphSlice {
            ready_only: true,
            ..GraphSlice::default()
        };
        assert_eq!(
            map_of(&mut storage, &ready),
            vec!["task.open", "task.outside"],
            "a hub waiting on a live child is not dispatchable, and a closed Task is done"
        );
    }

    /// Every narrowing is a predicate over the same notebook, so asking for
    /// two asks for the intersection.
    #[test]
    fn the_ready_lens_narrows_inside_an_epic_branch() {
        let mut storage = an_epic();
        let ready_inside = GraphSlice {
            ready_only: true,
            ..inside("task.hub")
        };
        assert_eq!(map_of(&mut storage, &ready_inside), vec!["task.open"]);
    }

    /// A line has to land on a tile, so an edge whose far end the slice
    /// left off the map is not one.
    #[test]
    fn an_edge_off_the_map_is_not_drawn() {
        let mut storage = an_epic();
        let ready = GraphSlice {
            ready_only: true,
            ..GraphSlice::default()
        };
        let graph = Notebook::new(&mut storage).graph(&ready).unwrap();
        assert_eq!(
            graph.edges().len(),
            0,
            "the ready lens holds only what waits on nothing live"
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

    /// A focus asks what has to settle before a Task and what its settling
    /// releases. Another Task hanging off the same blocker answers neither.
    #[test]
    fn a_focus_walks_the_line_of_work_and_not_across_it() {
        let mut storage = a_chain();
        assert!(
            !map_of(&mut storage, &around("task.b", 2)).contains(&"task.sibling".to_owned()),
            "a Task waiting on the same blocker is beside this line of work, not on it"
        );
    }

    /// A depth of nothing is the record alone, which is the honest answer
    /// to asking for no hops.
    #[test]
    fn a_focus_with_no_depth_is_the_record_alone() {
        let mut storage = a_chain();
        assert_eq!(map_of(&mut storage, &around("task.c", 0)), vec!["task.c"]);
    }

    /// A focus and another narrowing are two predicates over one notebook.
    #[test]
    fn a_focus_narrows_with_the_lens_beside_it() {
        let mut storage = a_chain();
        let ready_around = GraphSlice {
            ready_only: true,
            ..around("task.c", 2)
        };
        assert_eq!(
            map_of(&mut storage, &ready_around),
            vec!["task.a"],
            "only the head of the chain waits on nothing"
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
    /// epic never reached.
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
                &task("task.open", "open", &["from: task.hub"], ""),
            ),
            (
                "archive/tasks/task.done.md",
                &task("task.done", "closed", &["from: task.hub"], ""),
            ),
            (
                "tasks/task.outside.md",
                &task("task.outside", "open", &[], ""),
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
