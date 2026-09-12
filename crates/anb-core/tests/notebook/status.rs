mod status_dashboard {
    use crate::*;
    use anb_core::Status;

    fn status(storage: &mut MemoryStorage, identity: Option<&str>, by: Option<&str>) -> Status {
        Notebook::new(storage)
            .with_identity(identity)
            .status(TODAY, Budget::Unbounded, by, no_lost_proofs)
            .unwrap()
    }

    fn ids<'a>(rows: impl Iterator<Item = &'a String>) -> Vec<&'a str> {
        rows.map(String::as_str).collect()
    }

    #[test]
    fn invalid_records_are_excluded_from_every_work_section() {
        for (kind, state, dir) in [
            ("task", "active", "tasks"),
            ("task", "review", "tasks"),
            ("question", "open", "questions"),
        ] {
            let id = format!("{kind}.demo");
            let path = format!("{dir}/{id}.md");
            let mut storage = storage_with(&[(
                &path,
                &record_file(&id, kind, state, &["from: task.ghost"], ""),
            )]);
            let seen = status(&mut storage, None, None);
            assert!(seen.active.is_empty());
            assert!(seen.review.is_empty());
            assert!(seen.questions.is_empty());
        }
    }

    #[test]
    fn each_active_task_keeps_its_latest_log_entry() {
        let mut storage = storage_with(&[
            (
                "tasks/task.a.md",
                &record_file(
                    "task.a",
                    "task",
                    "active",
                    &[],
                    "- 2026-08-24 Ada: first\n- 2026-08-25 Ada: continue A\n",
                ),
            ),
            (
                "tasks/task.b.md",
                &record_file(
                    "task.b",
                    "task",
                    "active",
                    &[],
                    "- 2026-08-25 Grace: continue B\n",
                ),
            ),
        ]);
        let seen = status(&mut storage, None, None);
        assert_eq!(seen.active.len(), 2);
        assert_eq!(
            seen.active[0].log.as_deref(),
            Some("- 2026-08-25 Ada: continue A")
        );
        assert_eq!(
            seen.active[1].log.as_deref(),
            Some("- 2026-08-25 Grace: continue B")
        );
    }

    #[test]
    fn held_tasks_are_separate_from_active_work_and_keep_their_reason() {
        let mut storage = storage_with(&[
            (
                "tasks/task.parked.md",
                &record_file(
                    "task.parked",
                    "task",
                    "active",
                    &[
                        "taken-by: Grace",
                        "hold: waits for the API key",
                        "hold-until: 2026-09-20",
                    ],
                    "",
                ),
            ),
            (
                "tasks/task.working.md",
                &record_file("task.working", "task", "active", &[], ""),
            ),
        ]);
        let seen = status(&mut storage, None, None);
        assert_eq!(
            ids(seen.active.iter().map(|task| &task.id)),
            ["task.working"]
        );
        assert_eq!(seen.held[0].id, "task.parked");
        assert_eq!(seen.held[0].reason, "waits for the API key");
        assert_eq!(seen.held[0].until.as_deref(), Some("2026-09-20"));
        assert_eq!(seen.held[0].attribution.taken_by.as_deref(), Some("Grace"));
    }

    #[test]
    fn a_closed_task_with_a_hold_field_is_not_waiting() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &record_file(
                "task.demo",
                "task",
                "closed",
                &["hold: waits for the API key", "closed: 2026-08-26"],
                "",
            ),
        )]);
        assert!(status(&mut storage, None, None).held.is_empty());
    }

    #[test]
    fn held_work_alone_is_quiet_until_it_becomes_debt() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &record_file(
                "task.demo",
                "task",
                "active",
                &["hold: waiting on legal, still pending"],
                "",
            ),
        )]);
        let seen = status(&mut storage, None, None);
        assert!(seen.quiet);
        assert_eq!(seen.held[0].reason, "waiting on legal, still pending");
    }

    #[test]
    fn active_tasks_are_ordered_by_latest_touch() {
        let mut storage = storage_with(&[
            (
                "tasks/task.old.md",
                &record_file("task.old", "task", "active", &[], "")
                    .replace("updated: 2026-08-25", "updated: 2026-08-21"),
            ),
            (
                "tasks/task.new.md",
                &record_file("task.new", "task", "active", &[], "")
                    .replace("updated: 2026-08-25", "updated: 2026-08-27"),
            ),
        ]);
        let seen = status(&mut storage, None, None);
        assert_eq!(
            ids(seen.active.iter().map(|task| &task.id)),
            ["task.new", "task.old"]
        );
    }

    #[test]
    fn the_readers_own_active_work_precedes_more_recent_team_work() {
        let mut storage = storage_with(&[
            (
                "tasks/task.hers.md",
                &record_file("task.hers", "task", "active", &["taken-by: Grace"], "")
                    .replace("updated: 2026-08-25", "updated: 2026-08-27"),
            ),
            (
                "tasks/task.mine.md",
                &record_file(
                    "task.mine",
                    "task",
                    "active",
                    &["by: Grace", "taken-by: Ada"],
                    "",
                )
                .replace("updated: 2026-08-25", "updated: 2026-08-21"),
            ),
        ]);
        let seen = status(&mut storage, Some("Ada"), None);
        assert_eq!(
            ids(seen.active.iter().map(|task| &task.id)),
            ["task.mine", "task.hers"]
        );
        assert_eq!(seen.active[0].attribution.by.as_deref(), Some("Grace"));
        assert_eq!(seen.active[0].attribution.taken_by.as_deref(), Some("Ada"));
        assert_eq!(
            seen.active[1].attribution.taken_by.as_deref(),
            Some("Grace")
        );
    }

    #[test]
    fn review_and_held_sections_prioritize_the_readers_own_work() {
        for state in ["review", "open"] {
            let own = if state == "open" {
                vec!["taken-by: Ada", "hold: waits on legal"]
            } else {
                vec!["taken-by: Ada"]
            };
            let other = if state == "open" {
                vec!["taken-by: Grace", "hold: waits on legal"]
            } else {
                vec!["taken-by: Grace"]
            };
            let mut storage = storage_with(&[
                (
                    "tasks/task.hers.md",
                    &record_file("task.hers", "task", state, &other, ""),
                ),
                (
                    "tasks/task.mine.md",
                    &record_file("task.mine", "task", state, &own, ""),
                ),
            ]);
            let seen = status(&mut storage, Some("Ada"), None);
            let ordered = if state == "review" {
                ids(seen.review.iter().map(|task| &task.id))
            } else {
                ids(seen.held.iter().map(|task| &task.id))
            };
            assert_eq!(ordered, ["task.mine", "task.hers"]);
        }
    }

    #[test]
    fn work_waiting_on_the_reader_is_in_their_scope() {
        let mut storage = storage_with(&[
            (
                "tasks/task.review.md",
                &record_file(
                    "task.review",
                    "task",
                    "review",
                    &["taken-by: Grace", "to: Ada"],
                    "",
                ),
            ),
            (
                "tasks/task.other.md",
                &record_file(
                    "task.other",
                    "task",
                    "review",
                    &["taken-by: Grace", "to: Grace"],
                    "",
                ),
            ),
            (
                "questions/question.review.md",
                &record_file(
                    "question.review",
                    "question",
                    "open",
                    &["by: Grace", "to: Ada"],
                    "",
                ),
            ),
        ]);
        let seen = status(&mut storage, Some("Ada"), Some("Ada"));
        assert_eq!(
            ids(seen.review.iter().map(|task| &task.id)),
            ["task.review"]
        );
        assert_eq!(
            ids(seen.questions.iter().map(|question| &question.id)),
            ["question.review"]
        );
        assert_eq!(seen.review[0].attribution.to.as_deref(), Some("Ada"));
    }

    #[test]
    fn unknown_reader_identity_keeps_each_records_attribution() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &record_file(
                "task.demo",
                "task",
                "active",
                &["by: Grace", "taken-by: Ada"],
                "",
            ),
        )]);
        let seen = status(&mut storage, None, None);
        assert_eq!(seen.active[0].attribution.by.as_deref(), Some("Grace"));
        assert_eq!(seen.active[0].attribution.taken_by.as_deref(), Some("Ada"));
    }

    #[test]
    fn questions_prioritize_the_reader_then_the_oldest_question() {
        let mut storage = storage_with(&[
            (
                "questions/question.old.md",
                &record_file("question.old", "question", "open", &[], "")
                    .replace("created: 2026-08-24", "created: 2026-08-10"),
            ),
            (
                "questions/question.mine.md",
                &record_file("question.mine", "question", "open", &["by: Ada"], ""),
            ),
            (
                "questions/question.new.md",
                &record_file("question.new", "question", "open", &[], ""),
            ),
        ]);
        let seen = status(&mut storage, Some("Ada"), None);
        assert_eq!(
            ids(seen.questions.iter().map(|question| &question.id)),
            ["question.mine", "question.old", "question.new"]
        );
    }

    #[test]
    fn review_ready_or_open_questions_make_the_notebook_nonquiet() {
        for (kind, state, dir) in [
            ("task", "review", "tasks"),
            ("task", "open", "tasks"),
            ("question", "open", "questions"),
        ] {
            let id = format!("{kind}.demo");
            let path = format!("{dir}/{id}.md");
            let mut storage = storage_with(&[(&path, &record_file(&id, kind, state, &[], ""))]);
            assert!(!status(&mut storage, None, None).quiet);
        }
    }

    #[test]
    fn debt_alone_makes_the_notebook_nonquiet() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &["review-by: 2026-08-26"],
                "",
            ),
        )]);
        let seen = status(&mut storage, None, None);
        assert!(!seen.quiet);
        assert_eq!(seen.debt, 1);
    }

    #[test]
    fn knowledge_alone_is_quiet() {
        let mut storage = storage_with(&[
            (
                "notes/note.demo.md",
                &record_file("note.demo", "note", "active", &[], ""),
            ),
            (
                "decisions/decision.rule.md",
                &record_file("decision.rule", "decision", "active", &["kind: rule"], ""),
            ),
        ]);
        assert!(status(&mut storage, None, None).quiet);
    }

    #[test]
    fn a_scoped_dashboard_contains_owned_work_and_keeps_project_counts() {
        let mut storage = storage_with(&[
            (
                "tasks/task.mine.md",
                &record_file("task.mine", "task", "active", &["taken-by: Ada"], ""),
            ),
            (
                "tasks/task.hers.md",
                &record_file("task.hers", "task", "active", &["taken-by: Grace"], ""),
            ),
            (
                "questions/question.mine.md",
                &record_file("question.mine", "question", "open", &["by: Ada"], ""),
            ),
            (
                "questions/question.hers.md",
                &record_file("question.hers", "question", "open", &["by: Grace"], ""),
            ),
        ]);
        let seen = status(&mut storage, Some("Ada"), Some("Ada"));
        assert_eq!(seen.by.as_deref(), Some("Ada"));
        assert_eq!(seen.counts.tasks, 2);
        assert_eq!(seen.counts.questions, 2);
        assert_eq!(ids(seen.active.iter().map(|task| &task.id)), ["task.mine"]);
        assert_eq!(
            ids(seen.questions.iter().map(|question| &question.id)),
            ["question.mine"]
        );
    }

    #[test]
    fn handing_work_over_removes_it_from_the_authors_scope() {
        let mut storage = storage_with(&[
            (
                "tasks/task.hers.md",
                &record_file(
                    "task.hers",
                    "task",
                    "active",
                    &["by: Ada", "taken-by: Grace"],
                    "",
                ),
            ),
            (
                "tasks/task.pool.md",
                &record_file("task.pool", "task", "open", &["by: Ada"], ""),
            ),
            (
                "tasks/task.mine.md",
                &record_file(
                    "task.mine",
                    "task",
                    "open",
                    &["by: Ada", "taken-by: Ada"],
                    "",
                ),
            ),
        ]);
        let seen = status(&mut storage, Some("Ada"), Some("Ada"));
        assert!(seen.active.is_empty());
        assert_eq!(ids(seen.ready.iter().map(|task| &task.id)), ["task.mine"]);
        assert_eq!(seen.untaken, 1);
    }

    #[test]
    fn a_pool_alone_keeps_an_empty_personal_scope_nonquiet() {
        let mut storage = storage_with(&[(
            "tasks/task.pool.md",
            &record_file("task.pool", "task", "open", &[], ""),
        )]);
        let seen = status(&mut storage, Some("Ada"), Some("Ada"));
        assert!(seen.ready.is_empty());
        assert_eq!(seen.untaken, 1);
        assert!(!seen.quiet);
    }

    #[test]
    fn another_persons_work_does_not_make_an_empty_scope_nonquiet() {
        let mut storage = storage_with(&[(
            "tasks/task.hers.md",
            &record_file("task.hers", "task", "active", &["taken-by: Grace"], ""),
        )]);
        let seen = status(&mut storage, Some("Ada"), Some("Ada"));
        assert_eq!(seen.by.as_deref(), Some("Ada"));
        assert!(seen.quiet);
    }

    #[test]
    fn identity_filtering_does_not_remove_dependency_blockers() {
        let mut storage = storage_with(&[
            (
                "tasks/task.mine.md",
                &record_file(
                    "task.mine",
                    "task",
                    "open",
                    &["taken-by: Ada", "blocked-by: task.hers"],
                    "",
                ),
            ),
            (
                "tasks/task.hers.md",
                &record_file("task.hers", "task", "active", &["taken-by: Grace"], ""),
            ),
        ]);
        assert!(
            status(&mut storage, Some("Ada"), Some("Ada"))
                .ready
                .is_empty()
        );
    }

    #[test]
    fn core_status_preserves_record_text_for_the_host_to_encode() {
        let title = r#"A \"quoted\" title"#;
        let body = "- 2026-08-26 Ada: continue\u{1b}[31m\n";
        let source = record_file("task.demo", "task", "active", &[], body)
            .replace("title: A demo record", &format!("title: \"{title}\""));
        let mut storage = storage_with(&[("tasks/task.demo.md", &source)]);
        let seen = status(&mut storage, None, None);
        assert_eq!(seen.active[0].title, "A \"quoted\" title");
        assert_eq!(seen.active[0].log.as_deref(), Some(body.trim_end()));
    }

    #[test]
    fn counts_include_every_live_type_and_exclude_the_archive() {
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &record_file("task.demo", "task", "open", &[], ""),
            ),
            (
                "notes/note.demo.md",
                &record_file("note.demo", "note", "active", &[], ""),
            ),
            (
                "decisions/decision.rule.md",
                &record_file("decision.rule", "decision", "active", &["kind: rule"], ""),
            ),
            (
                "questions/question.demo.md",
                &record_file("question.demo", "question", "open", &[], ""),
            ),
            (
                "archive/tasks/task.done.md",
                &record_file("task.done", "task", "closed", &[], ""),
            ),
        ]);
        let seen = status(&mut storage, None, None);
        assert_eq!(
            seen.counts,
            anb_core::Counts {
                tasks: 1,
                notes: 1,
                decisions: 1,
                questions: 1
            }
        );
    }
}

mod debt_signals {
    use crate::*;

    /// A record whose clock is under the test's control: `updated` is the
    /// override this module is about.
    fn aged(id: &str, type_word: &str, state: &str, updated: &str, extra: &[&str]) -> String {
        let mut lines = extra.join("\n");
        if !lines.is_empty() {
            lines.push('\n');
        }
        format!(
            "---\nid: {id}\ntype: {type_word}\nstate: {state}\ntitle: A demo record\n{lines}created: 2026-08-01\nupdated: {updated}\n---\n"
        )
    }

    fn debt_of(storage: &mut MemoryStorage) -> Vec<DebtSignal> {
        Notebook::new(storage).debt(TODAY, no_lost_proofs).unwrap()
    }

    fn lines_of(debt: &[DebtSignal]) -> Vec<String> {
        debt.iter().map(DebtSignal::line).collect()
    }

    /// A clock counts days behind, and there are none: a date ahead of
    /// today reads as today rather than running the clock backwards.
    #[test]
    fn a_record_touched_in_the_future_is_not_stale() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "active", "2026-12-01", &[]),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    /// Each clock is configurable, and each key moves its own: a threshold
    /// wired to the wrong clock leaves the record it names silent, and one
    /// counting a day early speaks before it is owed anything.
    #[test]
    fn every_debt_key_moves_the_clock_it_names_on_the_day_it_names() {
        // Against a threshold of two days: due, and one day short of due.
        for (key, path, id, type_word, state, extra, expected) in [
            (
                "debt-task-stale",
                "tasks/task.demo.md",
                "task.demo",
                "task",
                "active",
                &[][..],
                DebtSignal::TaskStale {
                    id: "task.demo".into(),
                    days: 2,
                },
            ),
            (
                "debt-question-age",
                "questions/question.demo.md",
                "question.demo",
                "question",
                "open",
                &[][..],
                DebtSignal::QuestionAge {
                    id: "question.demo".into(),
                    days: 2,
                },
            ),
            (
                "debt-question-age-task-born",
                "questions/question.demo.md",
                "question.demo",
                "question",
                "open",
                &["from: task.origin"][..],
                DebtSignal::QuestionAge {
                    id: "question.demo".into(),
                    days: 2,
                },
            ),
            (
                "debt-hold-stale",
                "tasks/task.demo.md",
                "task.demo",
                "task",
                "open",
                &["hold: waiting on the owner"][..],
                DebtSignal::HoldStale {
                    id: "task.demo".into(),
                    days: 2,
                },
            ),
            (
                "debt-review-stale",
                "tasks/task.demo.md",
                "task.demo",
                "task",
                "review",
                &[][..],
                DebtSignal::ReviewStale {
                    id: "task.demo".into(),
                    days: 2,
                },
            ),
        ] {
            for (touched, owed) in [("2026-08-25", true), ("2026-08-26", false)] {
                let mut storage = storage_with(&[
                    ("config", &format!("{key}: 2\n")),
                    (
                        "tasks/task.origin.md",
                        &aged("task.origin", "task", "open", TODAY, &[]),
                    ),
                    (path, &aged(id, type_word, state, touched, extra)),
                ]);
                let signals = debt_of(&mut storage);
                if owed {
                    assert!(
                        signals.contains(&expected),
                        "`{key}` must move the clock it names: {signals:?}"
                    );
                } else {
                    assert!(
                        !signals
                            .iter()
                            .any(|signal| signal.code() == expected.code()),
                        "`{key}` speaks a day early: {signals:?}"
                    );
                }
            }
        }
    }

    /// Every threshold a notebook with no config file runs on, read from
    /// both sides: silent the day before it is owed, speaking on the day.
    /// A default quietly shortened passes every one-sided test there is.
    ///
    /// `days` is the threshold itself, and the two dates are derived from
    /// it against `TODAY`, so a row states its rule rather than hiding it
    /// in two hand-computed calendar dates. Every fixture also holds a
    /// fresh open `task.origin` — the record a task-born Question needs to
    /// point at, and silent on every clock itself.
    #[test]
    fn every_default_clock_speaks_on_the_day_it_is_owed_and_not_before() {
        for (case, path, id, type_word, state, extra, days, signal) in [
            (
                "task-stale",
                "tasks/task.demo.md",
                "task.demo",
                "task",
                "active",
                &[][..],
                7,
                DebtSignal::TaskStale {
                    id: "task.demo".into(),
                    days: 7,
                },
            ),
            (
                "question-age",
                "questions/question.demo.md",
                "question.demo",
                "question",
                "open",
                &[][..],
                14,
                DebtSignal::QuestionAge {
                    id: "question.demo".into(),
                    days: 14,
                },
            ),
            (
                "question-age-task-born",
                "questions/question.demo.md",
                "question.demo",
                "question",
                "open",
                &["from: task.origin"][..],
                7,
                DebtSignal::QuestionAge {
                    id: "question.demo".into(),
                    days: 7,
                },
            ),
            (
                "hold-stale",
                "tasks/task.demo.md",
                "task.demo",
                "task",
                "open",
                &["hold: waiting on the owner"][..],
                14,
                DebtSignal::HoldStale {
                    id: "task.demo".into(),
                    days: 14,
                },
            ),
            (
                "review-stale",
                "tasks/task.demo.md",
                "task.demo",
                "task",
                "review",
                &[][..],
                7,
                DebtSignal::ReviewStale {
                    id: "task.demo".into(),
                    days: 7,
                },
            ),
        ] {
            let origin = (
                "tasks/task.origin.md",
                aged("task.origin", "task", "open", TODAY, &[]),
            );
            let fixture = |touched: &str| {
                storage_with(&[
                    (origin.0, origin.1.as_str()),
                    (path, &aged(id, type_word, state, touched, extra)),
                ])
            };
            let mut before = fixture(&days_before(days - 1));
            assert_eq!(debt_of(&mut before), vec![], "{case}, the day before");
            let mut owed_day = fixture(&days_before(days));
            assert_eq!(debt_of(&mut owed_day), vec![signal], "{case}, on the day");
        }
    }

    /// The date `days` before `TODAY`, so a clock's row states its
    /// threshold instead of a calendar. Every default is under a fortnight,
    /// so the arithmetic never leaves the month.
    fn days_before(days: u32) -> String {
        format!("2026-08-{:02}", 27 - days)
    }

    #[test]
    fn a_held_task_gone_quiet_is_hold_stale_not_task_stale() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged(
                "task.demo",
                "task",
                "active",
                "2026-08-13",
                &["hold: waiting on the owner"],
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::HoldStale {
                id: "task.demo".into(),
                days: 14
            }]
        );
    }

    #[test]
    fn a_fresh_hold_is_not_debt_even_on_a_stale_clock_threshold() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged(
                "task.demo",
                "task",
                "active",
                "2026-08-20",
                &["hold: waiting on the owner"],
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_task_born_question_ages_faster_than_a_free_standing_one() {
        let mut storage = storage_with(&[
            (
                "questions/question.parked.md",
                &aged(
                    "question.parked",
                    "question",
                    "open",
                    "2026-08-20",
                    &["from: task.origin"],
                ),
            ),
            (
                "questions/question.free.md",
                &aged("question.free", "question", "open", "2026-08-20", &[]),
            ),
            (
                "tasks/task.origin.md",
                &record_file("task.origin", "task", "active", &[], ""),
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::QuestionAge {
                id: "question.parked".into(),
                days: 7
            }]
        );
    }

    #[test]
    fn a_question_surfaces_the_moment_its_origin_task_closes() {
        let mut storage = storage_with(&[
            (
                "questions/question.parked.md",
                &aged(
                    "question.parked",
                    "question",
                    "open",
                    TODAY,
                    &["from: task.origin"],
                ),
            ),
            (
                "tasks/task.origin.md",
                &record_file("task.origin", "task", "closed", &["closed: 2026-08-26"], ""),
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::OriginClosed {
                id: "question.parked".into(),
                origin: "task.origin".into()
            }]
        );
    }

    #[test]
    fn a_review_by_date_surfaces_on_its_own_day_on_any_record() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &aged(
                "note.demo",
                "note",
                "active",
                TODAY,
                &[&format!("review-by: {TODAY}")],
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::ReviewDue {
                id: "note.demo".into(),
                date: TODAY.into()
            }]
        );
    }

    #[test]
    fn a_future_review_by_stays_silent() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &aged(
                "note.demo",
                "note",
                "active",
                TODAY,
                &["review-by: 2026-09-15"],
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_dangling_mention_names_the_citer_and_the_target() {
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file(
                "note.demo",
                "note",
                "active",
                &[],
                "The fix waits on task.gone.\n",
            ),
        )]);
        let debt = debt_of(&mut storage);
        assert_eq!(
            debt,
            vec![DebtSignal::DanglingMention {
                id: "note.demo".into(),
                target: "task.gone".into()
            }]
        );
        assert_eq!(
            lines_of(&debt),
            ["dangling-mention: note.demo -> task.gone"]
        );
    }

    #[test]
    fn a_mention_of_an_existing_record_is_not_debt() {
        let mut storage = storage_with(&[
            (
                "notes/note.demo.md",
                &record_file(
                    "note.demo",
                    "note",
                    "active",
                    &[],
                    "See task.demo for the plan.\n",
                ),
            ),
            (
                "tasks/task.demo.md",
                &task_file("closed", &["closed: 2026-08-26"]),
            ),
        ]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn an_invalid_record_is_a_debt_line_with_its_error_count_and_no_clock() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "wandering", "2026-08-01", &[]),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::Invalid {
                path: "tasks/task.demo.md".into(),
                errors: 1
            }]
        );
    }

    #[test]
    fn origin_closed_outranks_the_age_clock_when_both_would_fire() {
        let mut storage = storage_with(&[
            (
                "questions/question.parked.md",
                &aged(
                    "question.parked",
                    "question",
                    "open",
                    "2026-08-10",
                    &["from: task.origin"],
                ),
            ),
            (
                "tasks/task.origin.md",
                &record_file("task.origin", "task", "closed", &["closed: 2026-08-26"], ""),
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::OriginClosed {
                id: "question.parked".into(),
                origin: "task.origin".into()
            }]
        );
    }

    #[test]
    fn a_closed_task_still_carrying_a_hold_line_does_not_tick() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged(
                "task.demo",
                "task",
                "closed",
                "2026-08-01",
                &["hold: parked before the close", "closed: 2026-08-02"],
            ),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_dead_lifecycle_state_silences_review_by() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.demo.md",
                &aged(
                    "decision.demo",
                    "decision",
                    "superseded",
                    "2026-08-01",
                    &["superseded-by: decision.next", "review-by: 2026-08-10"],
                ),
            ),
            (
                "decisions/decision.next.md",
                &record_file(
                    "decision.next",
                    "decision",
                    "active",
                    &["supersedes: decision.demo"],
                    "",
                ),
            ),
        ]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_record_in_the_wrong_directory_resolves_nothing() {
        // The referenced file exists only under `tasks/`; as a `note.*` id it
        // resolves nowhere, so the referrer is excluded from the clocks and
        // both files are listed invalid.
        let mut storage = storage_with(&[
            (
                "tasks/note.helper.md",
                &record_file("note.helper", "note", "active", &[], ""),
            ),
            (
                "tasks/task.demo.md",
                &aged(
                    "task.demo",
                    "task",
                    "active",
                    "2026-08-10",
                    &["from: note.helper"],
                ),
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![
                DebtSignal::Invalid {
                    path: "tasks/note.helper.md".into(),
                    errors: 1
                },
                DebtSignal::Invalid {
                    path: "tasks/task.demo.md".into(),
                    errors: 1
                },
            ]
        );
    }

    #[test]
    fn the_proofs_offered_for_the_world_to_settle_are_the_live_records_claims() {
        let mut storage = storage_with(&[
            (
                "tasks/task.shipped.md",
                &record_file(
                    "task.shipped",
                    "task",
                    "closed",
                    &[
                        "link: sha f00dfeed",
                        "link: report notes/report.md",
                        "link: pr https://example.com/pull/7",
                        "link: note note.elsewhere",
                    ],
                    "",
                ),
            ),
            (
                "notes/note.elsewhere.md",
                &record_file("note.elsewhere", "note", "active", &[], ""),
            ),
            (
                "archive/tasks/task.filed.md",
                &record_file("task.filed", "task", "closed", &["link: sha deadbeef"], ""),
            ),
        ]);
        let mut offered = Vec::new();
        Notebook::new(&mut storage)
            .debt(TODAY, |cited| {
                offered = cited.to_vec();
                Vec::new()
            })
            .unwrap();
        assert_eq!(
            offered,
            vec![
                CitedProof {
                    record: "task.shipped".to_owned(),
                    kind: "sha".to_owned(),
                    target: "f00dfeed".to_owned(),
                },
                CitedProof {
                    record: "task.shipped".to_owned(),
                    kind: "report".to_owned(),
                    target: "notes/report.md".to_owned(),
                },
            ],
            "a commit and a file are claims about the world; a pull request \
             is not ours to reach, a note resolves inside the notebook, and \
             an archived record's claim is history no verb can settle"
        );
    }

    #[test]
    fn a_proof_the_world_no_longer_holds_is_named_on_the_dashboard() {
        let mut storage = storage_with(&[(
            "tasks/task.shipped.md",
            &record_file(
                "task.shipped",
                "task",
                "closed",
                &["link: sha f00dfeed"],
                "",
            ),
        )]);
        let debt = Notebook::new(&mut storage)
            .debt(TODAY, <[CitedProof]>::to_vec)
            .unwrap();
        assert_eq!(
            debt,
            vec![DebtSignal::LostProof {
                id: "task.shipped".to_owned(),
                proof: "sha f00dfeed".to_owned(),
            }],
            "a claim the world cannot answer for is the record's while it is live"
        );
        assert_eq!(
            lines_of(&debt),
            ["lost-proof: task.shipped -> sha f00dfeed"]
        );
    }

    #[test]
    fn a_record_its_own_errors_already_condemn_is_left_to_check() {
        let mut storage = storage_with(&[(
            "tasks/task.broken.md",
            &record_file(
                "task.broken",
                "task",
                "closed",
                &["from: task.never-written", "link: sha f00dfeed"],
                "",
            ),
        )]);
        let lost = vec![CitedProof {
            record: "task.broken".to_owned(),
            kind: "sha".to_owned(),
            target: "f00dfeed".to_owned(),
        }];
        assert!(
            Notebook::new(&mut storage)
                .debt(TODAY, |_| lost.clone())
                .unwrap()
                .iter()
                .all(|signal| !matches!(signal, DebtSignal::LostProof { .. })),
            "one broken file, one signal — the invalid line already names it"
        );
    }

    #[test]
    fn the_archive_raises_no_debt_of_its_own() {
        // A settled record, valid where it sits, carrying the one signal
        // that outlives a settled state: a body citing an id nobody holds.
        // Only residence can silence it.
        let mut storage = storage_with(&[(
            "archive/tasks/task.demo.md",
            &record_file(
                "task.demo",
                "task",
                "closed",
                &[],
                "the work task.ghost asked for\n",
            ),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![],
            "history is over: it neither ages nor asks anything of the reader"
        );
    }

    #[test]
    fn a_corrupt_file_in_the_archive_is_checks_to_name_and_not_the_dashboards() {
        let mut storage = storage_with(&[(
            "archive/questions/question.demo.md",
            &aged("question.demo", "question", "open", "2026-06-01", &[]),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![],
            "a session start reads the live notebook, not every file history holds"
        );
        assert_eq!(
            Notebook::new(&mut storage)
                .check()
                .unwrap()
                .iter()
                .map(|located| located.path.clone())
                .collect::<Vec<String>>(),
            vec!["archive/questions/question.demo.md"],
            "nothing is lost: the surface that reads every file still names it"
        );
    }

    #[test]
    fn debt_orders_classes_by_the_clock_table_and_oldest_first_within() {
        let mut storage = storage_with(&[
            (
                "tasks/task.older.md",
                &aged("task.older", "task", "active", "2026-08-10", &[]),
            ),
            (
                "tasks/task.newer.md",
                &aged("task.newer", "task", "active", "2026-08-19", &[]),
            ),
            (
                "questions/question.demo.md",
                &aged("question.demo", "question", "open", "2026-08-01", &[]),
            ),
        ]);
        let codes_and_ids: Vec<(String, String)> = debt_of(&mut storage)
            .iter()
            .map(|signal| match signal {
                DebtSignal::TaskStale { id, .. } => ("task-stale".into(), id.clone()),
                DebtSignal::QuestionAge { id, .. } => ("question-age".into(), id.clone()),
                other => panic!("unexpected signal {other:?}"),
            })
            .collect();
        assert_eq!(
            codes_and_ids,
            vec![
                ("task-stale".to_owned(), "task.older".to_owned()),
                ("task-stale".to_owned(), "task.newer".to_owned()),
                ("question-age".to_owned(), "question.demo".to_owned()),
            ]
        );
    }
}

mod notebook_config {
    use crate::*;

    #[test]
    fn an_absent_config_is_the_default_budget() {
        let mut storage = MemoryStorage::new();
        let config = Notebook::new(&mut storage).config().unwrap();
        assert_eq!(config.budget(), Budget::Tokens(1500));
    }

    #[test]
    fn budget_zero_in_the_config_disables_the_ceiling() {
        let mut storage = storage_with(&[("config", "budget: 0\n")]);
        let config = Notebook::new(&mut storage).config().unwrap();
        assert_eq!(config.budget(), Budget::Unbounded);
    }

    #[test]
    fn a_config_budget_is_the_resolved_ceiling() {
        let mut storage = storage_with(&[("config", "budget: 400\n")]);
        let config = Notebook::new(&mut storage).config().unwrap();
        assert_eq!(config.budget(), Budget::Tokens(400));
    }

    /// The `scope` key is the one word-valued key: `mine` makes every read
    /// answer with the caller's own by default, and a word the key has no
    /// meaning for keeps the team's, named by check.
    #[test]
    fn the_scope_key_reads_mine_or_team_and_a_bad_word_keeps_the_team() {
        let mut storage = storage_with(&[("config", "scope: mine\n")]);
        assert_eq!(
            Notebook::new(&mut storage).config().unwrap().scope(),
            anb_core::Scope::Mine
        );
        let mut storage = storage_with(&[("config", "scope: ours\n")]);
        let notebook = Notebook::new(&mut storage);
        assert_eq!(notebook.config().unwrap().scope(), anb_core::Scope::Team);
        let findings = notebook.check().unwrap();
        assert!(
            findings.iter().any(|located| located.path == "config"
                && located.finding.code == FindingCode::BadValue
                && located.finding.message.contains("team, mine")),
            "{findings:?}"
        );
    }

    #[test]
    fn an_absent_config_is_the_teams_scope() {
        let mut storage = MemoryStorage::new();
        let config = Notebook::new(&mut storage).config().unwrap();
        assert_eq!(config.scope(), anb_core::Scope::Team);
    }

    #[test]
    fn an_unknown_config_key_is_a_warning_in_check() {
        let mut storage = storage_with(&[("config", "budgett: 400\n")]);
        let findings = Notebook::new(&mut storage).check().unwrap();
        assert!(
            findings.iter().any(|located| located.path == "config"
                && located.finding.code == FindingCode::UnknownField),
            "{findings:?}"
        );
    }

    #[test]
    fn a_malformed_value_keeps_the_default_and_check_names_it() {
        let mut storage = storage_with(&[("config", "budget: many\n")]);
        let notebook = Notebook::new(&mut storage);
        assert_eq!(notebook.config().unwrap().budget(), Budget::Tokens(1500));
        let findings = notebook.check().unwrap();
        assert!(
            findings
                .iter()
                .any(|located| located.path == "config"
                    && located.finding.code == FindingCode::BadValue),
            "{findings:?}"
        );
    }

    #[test]
    fn a_duplicate_config_key_is_named_and_the_first_value_wins() {
        let mut storage = storage_with(&[("config", "budget: 300\nbudget: 900\n")]);
        let notebook = Notebook::new(&mut storage);
        assert_eq!(notebook.config().unwrap().budget(), Budget::Tokens(300));
        let findings = notebook.check().unwrap();
        assert!(
            findings.iter().any(|located| located.path == "config"
                && located.finding.code == FindingCode::DuplicateField),
            "{findings:?}"
        );
    }

    #[test]
    fn a_line_that_is_not_a_field_is_a_bad_envelope_line() {
        let mut storage = storage_with(&[("config", "just prose\n")]);
        let findings = Notebook::new(&mut storage).check().unwrap();
        assert!(
            findings.iter().any(|located| located.path == "config"
                && located.finding.code == FindingCode::BadEnvelopeLine),
            "{findings:?}"
        );
    }
}
