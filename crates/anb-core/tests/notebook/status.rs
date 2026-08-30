mod status_dashboard {
    use crate::*;

    fn status_text(storage: &mut MemoryStorage) -> String {
        Notebook::new(storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap()
            .text
    }

    #[test]
    fn an_active_task_is_the_in_flight_line_with_its_last_log_line() {
        let body = "Acceptance: the ladder holds.\n\n- 2026-08-25 claude: stopped at the ladder\n";
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &record_file("task.demo", "task", "active", &[], body),
        )]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("in-flight: task.demo \"A demo record\"\n"),
            "{text}"
        );
        assert!(
            text.contains("log: - 2026-08-25 claude: stopped at the ladder\n"),
            "{text}"
        );
    }

    #[test]
    fn the_dashboard_opens_on_ready_work_alone() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert!(!status.quiet);
        assert!(
            status.text.contains("ready[1]{id,priority,age,title}:\n"),
            "{}",
            status.text
        );
    }

    #[test]
    fn the_dashboard_opens_on_debt_alone() {
        // A routed question is settled; the open one aged past fourteen days.
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
            "---\nid: question.demo\ntype: question\nstate: open\ntitle: A demo record\ncreated: 2026-08-01\nupdated: 2026-08-01\n---\n",
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert!(!status.quiet);
        assert!(
            status.text.contains("question-age: question.demo (26d)"),
            "{}",
            status.text
        );
    }

    #[test]
    fn rules_alone_do_not_open_the_gate() {
        let mut storage = storage_with(&[(
            "decisions/decision.demo.md",
            &record_file("decision.demo", "decision", "active", &["kind: rule"], ""),
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert!(
            status.quiet,
            "a standing rule is not work in motion: {}",
            status.text
        );
    }

    #[test]
    fn more_ready_than_five_rows_shows_five_and_the_shorter_hint() {
        let files: Vec<(String, String)> = (0..7)
            .map(|index| {
                (
                    format!("tasks/task.t{index}.md"),
                    format!(
                        "---\nid: task.t{index}\ntype: task\nstate: open\ntitle: A demo record\ncreated: 2026-08-2{}\n---\n",
                        index % 8
                    ),
                )
            })
            .collect();
        let mut storage = MemoryStorage::from_files(files);
        let text = status_text(&mut storage);
        assert!(
            text.contains("ready[7]{id,priority,age,title}:\n"),
            "{text}"
        );
        assert_eq!(text.matches("\n  task.").count(), 5, "{text}");
        assert!(text.contains("  … 2 more: anb ready\n"), "{text}");
    }

    #[test]
    fn standing_rules_list_live_rule_decisions_only() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.rule.md",
                &record_file("decision.rule", "decision", "active", &["kind: rule"], ""),
            ),
            (
                "decisions/decision.shape.md",
                &record_file("decision.shape", "decision", "active", &["kind: shape"], ""),
            ),
            (
                "decisions/decision.dead.md",
                &record_file("decision.dead", "decision", "retired", &["kind: rule"], ""),
            ),
            ("tasks/task.demo.md", &task_file("open", &[])),
        ]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("rules[1]:\n  decision.rule: A demo record\n"),
            "{text}"
        );
        assert!(!text.contains("decision.shape"), "{text}");
        assert!(!text.contains("decision.dead"), "{text}");
    }

    #[test]
    fn an_in_flight_title_with_a_quote_is_escaped() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            "---\nid: task.demo\ntype: task\nstate: active\ntitle: Fix the \"quiet\" line\ncreated: 2026-08-24\n---\n",
        )]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("in-flight: task.demo \"Fix the \\\"quiet\\\" line\"\n"),
            "{text}"
        );
    }

    #[test]
    fn counts_span_live_records_of_every_type_and_skip_the_archive() {
        let mut storage = storage_with(&[
            ("tasks/task.demo.md", &task_file("open", &[])),
            (
                "decisions/decision.demo.md",
                &record_file("decision.demo", "decision", "active", &[], ""),
            ),
            (
                "notes/note.demo.md",
                &record_file("note.demo", "note", "active", &[], ""),
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
        let text = status_text(&mut storage);
        assert!(
            text.starts_with("ok: notebook — 1 tasks, 1 decisions, 1 notes, 1 questions\n"),
            "{text}"
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
        Notebook::new(storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap()
            .debt
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
    /// wired to the wrong clock would leave the record it names silent.
    #[test]
    fn every_debt_key_moves_the_clock_it_names() {
        let quiet_origin = (
            "tasks/task.origin.md",
            aged("task.origin", "task", "open", TODAY, &[]),
        );
        for (key, files, expected) in [
            (
                "debt-task-stale",
                vec![(
                    "tasks/task.demo.md",
                    aged("task.demo", "task", "active", "2026-08-25", &[]),
                )],
                DebtSignal::TaskStale {
                    id: "task.demo".into(),
                    days: 2,
                },
            ),
            (
                "debt-question-age",
                vec![(
                    "questions/question.demo.md",
                    aged("question.demo", "question", "open", "2026-08-25", &[]),
                )],
                DebtSignal::QuestionAge {
                    id: "question.demo".into(),
                    days: 2,
                },
            ),
            (
                "debt-question-age-task-born",
                vec![
                    quiet_origin.clone(),
                    (
                        "questions/question.demo.md",
                        aged(
                            "question.demo",
                            "question",
                            "open",
                            "2026-08-25",
                            &["from: task.origin"],
                        ),
                    ),
                ],
                DebtSignal::QuestionAge {
                    id: "question.demo".into(),
                    days: 2,
                },
            ),
            (
                "debt-hold-quiet",
                vec![(
                    "tasks/task.demo.md",
                    aged(
                        "task.demo",
                        "task",
                        "open",
                        "2026-08-25",
                        &["hold: waiting on the owner"],
                    ),
                )],
                DebtSignal::HoldQuiet {
                    id: "task.demo".into(),
                    days: 2,
                },
            ),
            (
                "debt-review-wait",
                vec![(
                    "tasks/task.demo.md",
                    aged("task.demo", "task", "review", "2026-08-25", &[]),
                )],
                DebtSignal::ReviewWait {
                    id: "task.demo".into(),
                    days: 2,
                },
            ),
        ] {
            let mut named: Vec<(&str, String)> = vec![("config", format!("{key}: 2\n"))];
            named.extend(files);
            let mut storage = storage_with(
                &named
                    .iter()
                    .map(|(path, text)| (*path, text.as_str()))
                    .collect::<Vec<_>>(),
            );
            assert!(
                debt_of(&mut storage).contains(&expected),
                "`{key}` must move the clock it names"
            );
        }
    }

    #[test]
    fn an_active_task_untouched_for_seven_days_is_stale() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "active", "2026-08-20", &[]),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::TaskStale {
                id: "task.demo".into(),
                days: 7
            }]
        );
    }

    #[test]
    fn six_quiet_days_are_not_yet_stale() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "active", "2026-08-21", &[]),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
    }

    #[test]
    fn a_held_task_gone_quiet_is_hold_quiet_not_task_stale() {
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
            vec![DebtSignal::HoldQuiet {
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
    fn a_free_standing_question_ages_at_fourteen_days() {
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
            &aged("question.demo", "question", "open", "2026-08-13", &[]),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::QuestionAge {
                id: "question.demo".into(),
                days: 14
            }]
        );
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
    fn a_review_task_waiting_seven_days_surfaces() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "review", "2026-08-20", &[]),
        )]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::ReviewWait {
                id: "task.demo".into(),
                days: 7
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
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert_eq!(
            status.debt,
            vec![DebtSignal::DanglingMention {
                id: "note.demo".into(),
                target: "task.gone".into()
            }]
        );
        assert!(
            status
                .text
                .contains("  dangling-mention: note.demo -> task.gone\n"),
            "{}",
            status.text
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
    fn two_live_decisions_citing_without_an_edge_are_one_pair_with_both_authors() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.a.md",
                &record_file(
                    "decision.a",
                    "decision",
                    "active",
                    &["by: supolka"],
                    "This tightens decision.b without replacing it.\n",
                ),
            ),
            (
                "decisions/decision.b.md",
                &record_file(
                    "decision.b",
                    "decision",
                    "active",
                    &["by: supolka", "via: claude-code"],
                    "And decision.a is the counterpart.\n",
                ),
            ),
        ]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        let pairs: Vec<&DebtSignal> = status
            .debt
            .iter()
            .filter(|signal| matches!(signal, DebtSignal::UndeclaredPair { .. }))
            .collect();
        assert_eq!(pairs.len(), 1, "both directions of citation are one pair");
        assert!(
            status.text.contains(
                "  undeclared-pair: decision.a (supolka) <-> decision.b (supolka/claude-code)\n"
            ),
            "{}",
            status.text
        );
    }

    #[test]
    fn a_declared_link_edge_silences_the_pair() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.a.md",
                &record_file(
                    "decision.a",
                    "decision",
                    "active",
                    &["link: see decision.b"],
                    "This tightens decision.b without replacing it.\n",
                ),
            ),
            (
                "decisions/decision.b.md",
                &record_file("decision.b", "decision", "active", &[], ""),
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
    fn a_review_task_six_days_in_is_not_yet_debt() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &aged("task.demo", "task", "review", "2026-08-21", &[]),
        )]);
        assert_eq!(debt_of(&mut storage), vec![]);
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
    fn an_invalid_counterpart_cannot_enter_an_undeclared_pair() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.a.md",
                &record_file(
                    "decision.a",
                    "decision",
                    "active",
                    &[],
                    "This tightens decision.b without replacing it.\n",
                ),
            ),
            (
                "decisions/decision.b.md",
                "---\nid: decision.b\ntype: decision\nstate: active\ntitle: A demo record\ncreated: 2026-8-4\n---\n",
            ),
        ]);
        assert_eq!(
            debt_of(&mut storage),
            vec![DebtSignal::Invalid {
                path: "decisions/decision.b.md".into(),
                errors: 1
            }]
        );
    }

    #[test]
    fn pairs_rank_by_the_older_members_created_date() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.young.md",
                "---\nid: decision.young\ntype: decision\nstate: active\ntitle: A demo record\ncreated: 2026-08-20\n---\n\nSee decision.newish.\n",
            ),
            (
                "decisions/decision.newish.md",
                "---\nid: decision.newish\ntype: decision\nstate: active\ntitle: A demo record\ncreated: 2026-08-18\n---\n",
            ),
            (
                "decisions/decision.old.md",
                "---\nid: decision.old\ntype: decision\nstate: active\ntitle: A demo record\ncreated: 2026-08-02\n---\n\nSee decision.older.\n",
            ),
            (
                "decisions/decision.older.md",
                "---\nid: decision.older\ntype: decision\nstate: active\ntitle: A demo record\ncreated: 2026-08-01\n---\n",
            ),
        ]);
        let pair_firsts: Vec<String> = debt_of(&mut storage)
            .iter()
            .filter_map(|signal| match signal {
                DebtSignal::UndeclaredPair { first, .. } => Some(first.id.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            pair_firsts,
            vec!["decision.old".to_owned(), "decision.newish".to_owned()]
        );
    }

    #[test]
    fn six_dangling_mentions_render_five_lines_and_a_hint() {
        let body = "task.gone0 task.gone1 task.gone2 task.gone3 task.gone4 task.gone5";
        let mut storage = storage_with(&[(
            "notes/note.demo.md",
            &record_file("note.demo", "note", "active", &[], &format!("{body}\n")),
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, no_lost_proofs)
            .unwrap();
        assert_eq!(status.debt.len(), 6, "the model keeps every signal");
        assert_eq!(
            status.text.matches("dangling-mention:").count(),
            5,
            "{}",
            status.text
        );
        assert!(status.text.contains("  … 1 more\n"), "{}", status.text);
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
            .status(TODAY, Budget::Unbounded, |cited| {
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
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, <[CitedProof]>::to_vec)
            .unwrap();
        assert_eq!(
            status.debt,
            vec![DebtSignal::LostProof {
                id: "task.shipped".to_owned(),
                proof: "sha f00dfeed".to_owned(),
            }],
            "a claim the world cannot answer for is the record's while it is live"
        );
        assert!(
            status
                .text
                .contains("  lost-proof: task.shipped -> sha f00dfeed\n"),
            "{}",
            status.text
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
                .status(TODAY, Budget::Unbounded, |_| lost.clone())
                .unwrap()
                .debt
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
