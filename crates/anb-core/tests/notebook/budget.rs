mod budget_ladder {
    use crate::*;

    /// A notebook with every section populated: an in-flight Task with a
    /// log, review work, rules, seven ready rows, an epic, and aged debt.
    fn full_notebook() -> MemoryStorage {
        let mut files: Vec<(String, String)> = vec![
            (
                "tasks/task.epic.md".into(),
                record_file(
                    "task.epic",
                    "task",
                    "open",
                    &["blocked-by: task.flight"],
                    "",
                ),
            ),
            (
                "tasks/task.flight.md".into(),
                record_file(
                    "task.flight",
                    "task",
                    "active",
                    &["from: task.epic"],
                    "- 2026-08-25 claude: stopped at the ladder\n",
                ),
            ),
            (
                "tasks/task.waiting.md".into(),
                record_file("task.waiting", "task", "review", &[], ""),
            ),
            (
                "decisions/decision.rule.md".into(),
                record_file("decision.rule", "decision", "active", &["kind: rule"], ""),
            ),
            (
                "questions/question.aged.md".into(),
                "---\nid: question.aged\ntype: question\nstate: open\ntitle: A demo record\ncreated: 2026-08-01\nupdated: 2026-08-01\n---\n"
                    .into(),
            ),
        ];
        for index in 0..7 {
            files.push((
                format!("tasks/task.r{index}.md"),
                format!(
                    "---\nid: task.r{index}\ntype: task\nstate: open\ntitle: A demo record\ncreated: 2026-08-2{}\n---\n",
                    index % 8
                ),
            ));
        }
        MemoryStorage::from_files(files)
    }

    struct Rendered {
        text: String,
        spent: u32,
    }

    fn rendered(budget: Budget) -> Rendered {
        let mut storage = full_notebook();
        let status = Notebook::new(&mut storage)
            .status(TODAY, budget, no_lost_proofs)
            .unwrap();
        Rendered {
            text: status.text,
            spent: status.spent,
        }
    }

    /// A notebook whose every listing section overflows its bound: eight
    /// Tasks in flight, eight waiting on a human, eight standing rules, and
    /// eight epics.
    fn crowded_notebook() -> MemoryStorage {
        let mut files: Vec<(String, String)> = Vec::new();
        for index in 0..8 {
            files.push((
                format!("tasks/task.flight{index}.md"),
                record_file(&format!("task.flight{index}"), "task", "active", &[], ""),
            ));
            files.push((
                format!("tasks/task.waiting{index}.md"),
                record_file(&format!("task.waiting{index}"), "task", "review", &[], ""),
            ));
            files.push((
                format!("decisions/decision.rule{index}.md"),
                record_file(
                    &format!("decision.rule{index}"),
                    "decision",
                    "active",
                    &["kind: rule"],
                    "",
                ),
            ));
            files.push((
                format!("tasks/task.hub{index}.md"),
                record_file(
                    &format!("task.hub{index}"),
                    "task",
                    "open",
                    &[&format!("blocked-by: task.child{index}")],
                    "",
                ),
            ));
            files.push((
                format!("tasks/task.child{index}.md"),
                record_file(
                    &format!("task.child{index}"),
                    "task",
                    "open",
                    &[&format!("from: task.hub{index}")],
                    "",
                ),
            ));
        }
        MemoryStorage::from_files(files)
    }

    #[test]
    fn a_crowded_section_shows_five_rows_and_counts_the_rest() {
        let mut storage = crowded_notebook();
        let status = Notebook::new(&mut storage)
            .status(
                TODAY,
                Budget::Tokens(Budget::DEFAULT_TOKENS),
                no_lost_proofs,
            )
            .unwrap();
        assert_eq!(
            status.text,
            "ok: notebook — 32 tasks, 8 decisions, 0 notes, 0 questions\n\
             in-flight: task.flight0 \"A demo record\"\n\
             in-flight: task.flight1 \"A demo record\"\n\
             in-flight: task.flight2 \"A demo record\"\n\
             in-flight: task.flight3 \"A demo record\"\n\
             in-flight: task.flight4 \"A demo record\"\n  \u{2026} 3 more in flight\n\
             review[8]: task.waiting0, task.waiting1, task.waiting2, task.waiting3, \
             task.waiting4, \u{2026} 3 more — waiting on a human\n\
             rules[8]:\n\
             \x20 decision.rule0: A demo record\n\
             \x20 decision.rule1: A demo record\n\
             \x20 decision.rule2: A demo record\n\
             \x20 decision.rule3: A demo record\n\
             \x20 decision.rule4: A demo record\n  \u{2026} 3 more\n\
             ready[8]{id,priority,age,title}:\n\
             \x20 task.child0,-,3d,A demo record\n\
             \x20 task.child1,-,3d,A demo record\n\
             \x20 task.child2,-,3d,A demo record\n\
             \x20 task.child3,-,3d,A demo record\n\
             \x20 task.child4,-,3d,A demo record\n  \u{2026} 3 more: anb ready\n\
             epics[8]:\n\
             \x20 task.hub0: 0/1 closed, next: task.child0\n\
             \x20 task.hub1: 0/1 closed, next: task.child1\n\
             \x20 task.hub2: 0/1 closed, next: task.child2\n\
             \x20 task.hub3: 0/1 closed, next: task.child3\n\
             \x20 task.hub4: 0/1 closed, next: task.child4\n  \u{2026} 3 more\n\
             budget: ~309/1500 tokens\n",
            "every section stops at five rows and counts the rest, and a \
             notebook that could fill any of them still leaves the default \
             budget nothing to degrade"
        );
    }

    #[test]
    fn the_floor_keeps_one_in_flight_line_however_many_are_flying() {
        let mut storage = crowded_notebook();
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Tokens(1), no_lost_proofs)
            .unwrap();
        assert_eq!(
            status.text.lines().count(),
            4,
            "counts, where work stopped, what else is flying, and the budget line: {}",
            status.text
        );
        assert!(
            status.text.contains("  \u{2026} 7 more in flight\n"),
            "{}",
            status.text
        );
    }

    #[test]
    fn an_unbounded_budget_prints_every_section_and_the_no_ceiling_line() {
        let full = rendered(Budget::Unbounded);
        for section in [
            "in-flight: task.flight",
            "log: - 2026-08-25 claude: stopped at the ladder",
            "review[1]: task.waiting — waiting on a human",
            "rules[1]:",
            "ready[7]{id,priority,age,title}:",
            "  … 2 more: anb ready",
            "debt[",
        ] {
            assert!(
                full.text.contains(section),
                "missing `{section}` in: {}",
                full.text
            );
        }
        assert!(
            full.text
                .contains(&format!("budget: ~{} tokens (no ceiling)\n", full.spent))
        );
        assert!(!full.text.contains("cut:"), "{}", full.text);
    }

    #[test]
    fn a_fitting_budget_cuts_nothing_and_reports_spent_over_ceiling() {
        let comfortable = rendered(Budget::Tokens(1500));
        assert!(comfortable.spent <= 1500);
        assert!(
            comfortable
                .text
                .contains(&format!("budget: ~{}/1500 tokens\n", comfortable.spent)),
            "{}",
            comfortable.text
        );
        assert!(!comfortable.text.contains("cut:"), "{}", comfortable.text);
    }

    /// The ladder's fixed order, read off a descending budget sweep: ready
    /// rows go first, then debt collapses, then rules, then the log line —
    /// never the other way around — and the text fits every budget the
    /// floor has not been forced past.
    /// The indented rows under one section header, wherever the ladder has
    /// moved the sections around it.
    fn rows_under(text: &str, header: &str) -> usize {
        text.lines()
            .skip_while(|line| !line.starts_with(header))
            .skip(1)
            .take_while(|line| line.starts_with("  "))
            .count()
    }

    #[test]
    fn the_epic_block_states_progress_and_collapses_to_a_count() {
        let full = rendered(Budget::Unbounded);
        assert!(
            full.text.contains("epics[1]:\n  task.epic: 0/1 closed"),
            "an epic whose only child is in flight has nothing ready: {}",
            full.text
        );

        let tight = rendered(Budget::Tokens(110));
        assert!(
            tight.text.contains("epics: 1 — anb list --for <id>"),
            "collapsed, it keeps the count and names the way back: {}",
            tight.text
        );
        assert!(
            tight.text.contains("epics\u{2192}count"),
            "and the budget line says what it cut: {}",
            tight.text
        );
    }

    #[test]
    fn sections_degrade_in_the_fixed_order_as_the_budget_shrinks() {
        let mut stages = Vec::new();
        for ceiling in [
            400, 150, 130, 110, 100, 90, 80, 70, 60, 50, 40, 30, 20, 10, 1,
        ] {
            let step = rendered(Budget::Tokens(ceiling));
            let ready_rows = rows_under(&step.text, "ready[");
            let epics_itemized = step.text.contains("epics[");
            let debt_itemized = step.text.contains("debt[");
            let rules_itemized = step.text.contains("rules[");
            let has_log = step.text.contains("log: ");
            let floor = !step.text.contains("ready") && !step.text.contains("debt");

            assert!(
                epics_itemized || ready_rows == 0,
                "epics collapsed while ready rows remain at {ceiling}: {}",
                step.text
            );
            assert!(
                debt_itemized || !epics_itemized,
                "debt collapsed before epics at {ceiling}: {}",
                step.text
            );
            assert!(
                rules_itemized || !debt_itemized,
                "rules collapsed before debt at {ceiling}: {}",
                step.text
            );
            assert!(
                has_log || !rules_itemized,
                "the log dropped before rules collapsed at {ceiling}: {}",
                step.text
            );
            assert!(
                step.spent <= ceiling || floor,
                "over budget without reaching the floor at {ceiling}: ~{} tokens: {}",
                step.spent,
                step.text
            );
            if step.text.contains("cut:") {
                assert!(step.text.contains("anb status --budget 0"), "{}", step.text);
            }
            let review_collapsed = step.text.contains("review: 1");
            if review_collapsed && !floor {
                assert!(
                    step.text.contains("review\u{2192}count"),
                    "a collapsed review list must be named as cut at {ceiling}: {}",
                    step.text
                );
            }
            if !has_log && !floor {
                assert!(
                    step.text.contains("log"),
                    "a dropped log line must be named as cut at {ceiling}: {}",
                    step.text
                );
            }
            stages.push((ready_rows, debt_itemized, rules_itemized, has_log));
        }
        let mut previous = stages[0];
        for stage in stages {
            assert!(
                stage.0 <= previous.0
                    && (!stage.1 || previous.1)
                    && (!stage.2 || previous.2)
                    && (!stage.3 || previous.3),
                "a smaller budget restored a section: {stage:?} after {previous:?}"
            );
            previous = stage;
        }
    }

    /// The number on the budget line must never understate the text it sits
    /// in: it is the sum of per-part estimates, so it may exceed the whole
    /// text's estimate by the one rounding token, never fall under it.
    #[test]
    fn the_reported_spent_never_understates_the_text() {
        for ceiling in [
            Budget::Unbounded,
            Budget::Tokens(1500),
            Budget::Tokens(100),
            Budget::Tokens(1),
        ] {
            let step = {
                let mut storage = full_notebook();
                Notebook::new(&mut storage)
                    .status(TODAY, ceiling, no_lost_proofs)
                    .unwrap()
            };
            let whole = anb_core::estimate_tokens(&step.text);
            assert!(
                step.spent >= whole && step.spent <= whole + 1,
                "spent ~{} vs whole-text estimate {whole}:\n{}",
                step.spent,
                step.text
            );
        }
    }

    #[test]
    fn a_cut_note_never_names_a_log_line_that_did_not_exist() {
        // Ready work only: the gate opens with no active Task and no log.
        let files: Vec<(String, String)> = (0..6)
            .map(|index| {
                (
                    format!("tasks/task.r{index}.md"),
                    format!(
                        "---\nid: task.r{index}\ntype: task\nstate: open\ntitle: A demo record\ncreated: 2026-08-2{}\n---\n",
                        index % 8
                    ),
                )
            })
            .collect();
        for ceiling in [200, 100, 60, 40, 20, 10, 2] {
            let mut storage = MemoryStorage::from_files(files.clone());
            let status = Notebook::new(&mut storage)
                .status(TODAY, Budget::Tokens(ceiling), no_lost_proofs)
                .unwrap();
            if let Some(cut) = status.text.split("cut: ").nth(1) {
                assert!(
                    !cut.contains("log"),
                    "no log line existed to cut at {ceiling}: {}",
                    status.text
                );
            }
        }
    }

    #[test]
    fn the_floor_keeps_counts_in_flight_and_the_budget_line() {
        let floor = rendered(Budget::Tokens(1));
        let lines: Vec<&str> = floor.text.lines().collect();
        assert_eq!(lines.len(), 3, "{}", floor.text);
        assert!(lines[0].starts_with("ok: notebook — "), "{}", floor.text);
        assert!(
            lines[1].starts_with("in-flight: task.flight"),
            "{}",
            floor.text
        );
        assert!(
            lines[2].starts_with(&format!(
                "budget: ~{}/1 tokens; cut: all but the first in-flight",
                floor.spent
            )),
            "the floor ships over budget, reported honestly: {}",
            floor.text
        );
    }

    #[test]
    fn a_quiet_notebook_ignores_the_ladder() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &task_file("closed", &["closed: 2026-08-25"]),
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Tokens(1), no_lost_proofs)
            .unwrap();
        assert!(status.quiet);
        assert!(
            status.text.starts_with("ok: notebook quiet — "),
            "{}",
            status.text
        );
    }

    /// The Budget regression fixture. The 350 is a pin, not a derivation:
    /// this notebook's dashboard measured 212 true o200k tokens before the
    /// rules, review, and log sections existed (2026-08-25), and the pin
    /// grants those plus the estimator's overshoot their room. Growth past
    /// it is dashboard bloat, and a budget-line regression is a failure.
    #[test]
    fn a_sixty_task_notebook_fits_the_default_budget_with_headroom() {
        let mut files: Vec<(String, String)> = vec![(
            "tasks/task.flight.md".into(),
            record_file(
                "task.flight",
                "task",
                "active",
                &[],
                "- 2026-08-25 claude: stopped at the ladder\n",
            ),
        )];
        for index in 0..59 {
            let state = if index % 3 == 0 { "open" } else { "closed" };
            files.push((
                format!("tasks/task.t{index}.md"),
                format!(
                    "---\nid: task.t{index}\ntype: task\nstate: {state}\ntitle: Parser accepts budget handling in the fences path\npriority: {}\ncreated: 2026-08-1{}\n---\n",
                    index % 5,
                    index % 10,
                ),
            ));
        }
        for index in 0..20 {
            let kind = if index % 4 == 0 { "rule" } else { "shape" };
            files.push((
                format!("decisions/decision.d{index}.md"),
                format!(
                    "---\nid: decision.d{index}\ntype: decision\nstate: active\nkind: {kind}\ntitle: Never render the archive without a stated reason\ncreated: 2026-08-12\n---\n"
                ),
            ));
        }
        for index in 0..30 {
            files.push((
                format!("notes/note.n{index}.md"),
                format!(
                    "---\nid: note.n{index}\ntype: note\nstate: active\nkind: fact\ntitle: The renderer keeps every byte it did not touch\ncreated: 2026-08-12\n---\n"
                ),
            ));
        }
        for index in 0..15 {
            files.push((
                format!("questions/question.q{index}.md"),
                format!(
                    "---\nid: question.q{index}\ntype: question\nstate: open\ntitle: Does the ladder hold on compaction\ncreated: 2026-08-2{}\nupdated: 2026-08-2{}\n---\n",
                    index % 8,
                    index % 8,
                ),
            ));
        }
        let mut storage = MemoryStorage::from_files(files);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Tokens(1500), no_lost_proofs)
            .unwrap();
        assert!(!status.text.contains("cut:"), "{}", status.text);
        assert!(
            status.spent <= 350,
            "the dashboard grew past the pinned budget: ~{} tokens:\n{}",
            status.spent,
            status.text
        );
    }
}
