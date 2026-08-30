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

    /// A record comes in sound, so the repair gate has nothing to hold
    /// against it: only the request's own guard stands between a caller and
    /// a value `check` would immediately condemn.
    #[test]
    fn a_priority_outside_the_scale_is_refused_on_a_sound_record() {
        let text = task_file("open", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        let error = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    priority: Some(9),
                    ..edit()
                },
                TODAY,
            )
            .unwrap_err();
        assert!(matches!(error, NotebookError::InvalidArgument { .. }));
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
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
    fn a_cleared_field_leaves_the_record() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &task_file("open", &["from: task.parent", "priority: 2"]),
        )]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    clear: vec!["from".to_owned(), "priority".to_owned()],
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.changed, vec!["from", "priority"]);
        assert_eq!(
            storage.read("tasks/task.demo.md").unwrap(),
            "---\nid: task.demo\ntype: task\nstate: open\ntitle: A demo record\ncreated: 2026-08-24\nupdated: 2026-08-27\n---\n",
            "a birth that never happened leaves no line behind"
        );
    }

    #[test]
    fn clearing_a_field_the_record_does_not_carry_changes_no_byte() {
        let text = task_file("open", &[]);
        let mut storage = storage_with(&[("tasks/task.demo.md", &text)]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "task.demo",
                &Edit {
                    clear: vec!["review-by".to_owned()],
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.changed, Vec::<&str>::new());
        assert_eq!(storage.read("tasks/task.demo.md").unwrap(), text);
    }

    /// A field the record's type does not allow is exactly the field a
    /// clear is for; only the write of one is refused.
    #[test]
    fn a_priority_a_decision_should_never_have_carried_is_cleared() {
        let mut storage = storage_with(&[(
            "decisions/decision.demo.md",
            &record_file("decision.demo", "decision", "active", &["priority: 2"], ""),
        )]);
        let edited = Notebook::new(&mut storage)
            .edit(
                "decision.demo",
                &Edit {
                    clear: vec!["priority".to_owned()],
                    ..edit()
                },
                TODAY,
            )
            .unwrap();
        assert_eq!(edited.changed, vec!["priority"]);
        assert!(
            !storage
                .read("decisions/decision.demo.md")
                .unwrap()
                .contains("priority")
        );
    }

    #[test]
    fn a_clear_is_refused_by_its_field() {
        let cases = [
            (
                Edit {
                    clear: vec!["state".to_owned()],
                    ..edit()
                },
                "state",
            ),
            (
                Edit {
                    from: Some("task.parent".to_owned()),
                    clear: vec!["from".to_owned()],
                    ..edit()
                },
                "from",
            ),
        ];
        for (edit, named) in cases {
            let mut storage = storage_with(&[
                ("tasks/task.demo.md", &task_file("open", &[])),
                (
                    "tasks/task.parent.md",
                    &record_file("task.parent", "task", "open", &[], ""),
                ),
            ]);
            let refused = Notebook::new(&mut storage)
                .edit("task.demo", &edit, TODAY)
                .unwrap_err();
            let NotebookError::InvalidArgument { reason } = refused else {
                panic!("a clear is judged before anything is read: {refused:?}");
            };
            assert!(
                reason.starts_with("clear:") && reason.contains(named),
                "the refusal names the field it is about: {reason}"
            );
        }
    }

    /// A record whose own bytes `check` condemns is frozen against every
    /// verb but the one that repairs it, and that verb is judged on what it
    /// leaves behind.
    mod a_record_with_an_error_finding {
        use super::*;

        /// The two repairs for a `from` naming nothing: erase the line, or
        /// point it somewhere real.
        #[test]
        fn is_written_by_an_edit_that_leaves_it_clean() {
            let repairs = [
                (
                    Edit {
                        clear: vec!["from".to_owned()],
                        ..edit()
                    },
                    "",
                ),
                (
                    Edit {
                        from: Some("task.parent".to_owned()),
                        ..edit()
                    },
                    "\nfrom: task.parent",
                ),
            ];
            for (repair, expected) in repairs {
                let mut storage = broken(&["from: task.ghost"]);
                let edited = Notebook::new(&mut storage)
                    .edit("task.demo", &repair, TODAY)
                    .expect("the edit leaves the record clean");
                assert_eq!(edited.changed, vec!["from"]);
                assert!(
                    storage
                        .read("tasks/task.demo.md")
                        .unwrap()
                        .contains(&format!("title: A demo record{expected}\n")),
                    "the repaired line reads as asked"
                );
                assert!(
                    Notebook::new(&mut storage).check().unwrap().is_empty(),
                    "and the notebook is clean after it"
                );
            }
        }

        /// The line an edit aims at is not always every line the findings
        /// are about, and the finding is what decides.
        #[test]
        fn is_refused_by_an_edit_that_would_leave_it_standing() {
            let half_repairs = [
                (
                    vec!["from: task.parent", "from: task.other"],
                    Edit {
                        from: Some("task.parent".to_owned()),
                        ..edit()
                    },
                ),
                (
                    vec!["tags: Not A Tag"],
                    Edit {
                        add_tags: vec!["ready".to_owned()],
                        ..edit()
                    },
                ),
                (
                    vec!["from: task.ghost"],
                    Edit {
                        title: Some("A sharper name".to_owned()),
                        ..edit()
                    },
                ),
            ];
            for (lines, half) in half_repairs {
                let mut storage = broken(&lines);
                let text = storage.read("tasks/task.demo.md").unwrap();
                assert!(
                    matches!(
                        Notebook::new(&mut storage).edit("task.demo", &half, TODAY),
                        Err(NotebookError::InvalidRecord { .. })
                    ),
                    "a half repair is refused like any other invalid write: {lines:?}"
                );
                assert_eq!(
                    storage.read("tasks/task.demo.md").unwrap(),
                    text,
                    "and writes nothing"
                );
            }
        }

        /// A required line missing is not on any line at all, and writing
        /// it is still the repair.
        #[test]
        fn is_written_by_the_edit_that_supplies_a_missing_line() {
            let mut storage = storage_with(&[(
                "tasks/task.demo.md",
                "---\nid: task.demo\ntype: task\nstate: open\ncreated: 2026-08-24\n---\n",
            )]);
            let edited = Notebook::new(&mut storage)
                .edit(
                    "task.demo",
                    &Edit {
                        title: Some("A recovered title".to_owned()),
                        ..edit()
                    },
                    TODAY,
                )
                .expect("the title the record lacks is the title the edit writes");
            assert_eq!(edited.changed, vec!["title"]);
            assert!(Notebook::new(&mut storage).check().unwrap().is_empty());
        }

        /// A file with no envelope is not a record to splice: there is no
        /// line to correct, only a file to write again.
        #[test]
        fn is_no_record_at_all_without_an_envelope() {
            let mut storage = storage_with(&[("tasks/task.demo.md", "just prose\n")]);
            assert!(matches!(
                Notebook::new(&mut storage)
                    .edit(
                        "task.demo",
                        &Edit {
                            title: Some("A sharper name".to_owned()),
                            ..edit()
                        },
                        TODAY,
                    )
                    .unwrap_err(),
                NotebookError::InvalidRecord { .. }
            ));
        }

        fn broken(lines: &[&str]) -> MemoryStorage {
            storage_with(&[
                ("tasks/task.demo.md", &task_file("open", lines)),
                (
                    "tasks/task.parent.md",
                    &record_file("task.parent", "task", "open", &[], ""),
                ),
            ])
        }
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
