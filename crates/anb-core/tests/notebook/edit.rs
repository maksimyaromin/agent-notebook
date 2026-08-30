mod edit_verb {
    use crate::*;

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
    fn an_origin_that_would_close_a_lineage_loop_is_refused_with_the_lineage_named() {
        // `task.child` was born inside `task.hub`, `task.grandchild` inside
        // it; making the grandchild the hub's origin would leave the hub
        // standing inside its own lineage.
        let mut storage = storage_with(&[
            (
                "tasks/task.hub.md",
                &record_file("task.hub", "task", "open", &[], ""),
            ),
            (
                "tasks/task.child.md",
                &record_file("task.child", "task", "open", &["from: task.hub"], ""),
            ),
            (
                "tasks/task.grandchild.md",
                &record_file("task.grandchild", "task", "open", &["from: task.child"], ""),
            ),
        ]);
        let before = storage.read("tasks/task.hub.md").unwrap();
        let error = Notebook::new(&mut storage)
            .edit(
                "task.hub",
                &Edit {
                    from: Some("task.grandchild".to_owned()),
                    ..edit()
                },
                TODAY,
            )
            .unwrap_err();
        let NotebookError::InvalidArgument { reason } = &error else {
            panic!("{error:?}");
        };
        assert!(
            reason.contains("task.grandchild \u{2192} task.child \u{2192} task.hub"),
            "the refusal walks the lineage it refuses: {reason}"
        );
        assert_eq!(
            storage.read("tasks/task.hub.md").unwrap(),
            before,
            "a refused edit moves no byte"
        );
    }

    #[test]
    fn a_lineage_that_does_not_loop_is_left_alone() {
        let mut storage = storage_with(&[
            (
                "tasks/task.hub.md",
                &record_file("task.hub", "task", "open", &[], ""),
            ),
            (
                "tasks/task.child.md",
                &record_file("task.child", "task", "open", &["from: task.hub"], ""),
            ),
            (
                "tasks/task.other.md",
                &record_file("task.other", "task", "open", &[], ""),
            ),
        ]);
        assert_eq!(
            Notebook::new(&mut storage)
                .edit(
                    "task.child",
                    &Edit {
                        from: Some("task.other".to_owned()),
                        ..edit()
                    },
                    TODAY,
                )
                .unwrap()
                .changed,
            vec!["from"]
        );
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
