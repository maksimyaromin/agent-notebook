mod budget_ladder {
    use crate::*;

    /// A notebook with every section populated: an active Task with a
    /// log, review work, seven ready rows, an open Question old enough to
    /// be Debt too, and a rule that reaches no section.
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
            .status(TODAY, budget, None, no_lost_proofs)
            .unwrap();
        Rendered {
            text: status.text,
            spent: status.spent,
        }
    }

    /// A notebook whose every listing section overflows its bound: eight
    /// active Tasks, eight waiting on a human, eight ready children of
    /// eight hubs, eight open Questions, and eight standing rules that
    /// reach no section.
    fn crowded_notebook() -> MemoryStorage {
        let mut files: Vec<(String, String)> = Vec::new();
        for index in 0..8 {
            files.push((
                format!("questions/question.q{index}.md"),
                record_file(&format!("question.q{index}"), "question", "open", &[], ""),
            ));
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
                None,
                no_lost_proofs,
            )
            .unwrap();
        let (sections, budget_line) = status.text.rsplit_once("budget: ").unwrap();
        assert_eq!(
            sections,
            "ok: notebook — 32 tasks, 8 decisions, 0 notes, 8 questions\n\
             active: task.flight0 \"A demo record\"\n\
             active: task.flight1 \"A demo record\"\n\
             active: task.flight2 \"A demo record\"\n\
             active: task.flight3 \"A demo record\"\n\
             active: task.flight4 \"A demo record\"\n  \u{2026} 3 more active\n\
             review[8]{id,taken-by,to}:\n\
             \x20 task.waiting0,-,-\n\
             \x20 task.waiting1,-,-\n\
             \x20 task.waiting2,-,-\n\
             \x20 task.waiting3,-,-\n\
             \x20 task.waiting4,-,-\n  \u{2026} 3 more\n\
             ready[8]{id,priority,age,taken-by,title}:\n\
             \x20 task.child0,-,3d,-,A demo record\n\
             \x20 task.child1,-,3d,-,A demo record\n\
             \x20 task.child2,-,3d,-,A demo record\n\
             \x20 task.child3,-,3d,-,A demo record\n\
             \x20 task.child4,-,3d,-,A demo record\n  \u{2026} 3 more: anb ready\n\
             questions[8]{id,age,by,to,title}:\n\
             \x20 question.q0,3d,-,-,A demo record\n\
             \x20 question.q1,3d,-,-,A demo record\n\
             \x20 question.q2,3d,-,-,A demo record\n\
             \x20 question.q3,3d,-,-,A demo record\n\
             \x20 question.q4,3d,-,-,A demo record\n  \u{2026} 3 more: anb list --type question\n",
            "every section stops at five rows and counts the rest; knowledge reaches none"
        );
        assert_eq!(
            budget_line,
            format!("~{}/{} tokens\n", status.spent, Budget::DEFAULT_TOKENS),
            "the budget line reports what the dashboard cost"
        );
        assert!(
            status.spent <= Budget::DEFAULT_TOKENS,
            "a notebook that could fill every section still leaves the \
             default budget nothing to degrade: {}",
            status.text
        );
    }

    #[test]
    fn the_floor_keeps_one_active_line_however_many_are_active() {
        let mut storage = crowded_notebook();
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Tokens(1), None, no_lost_proofs)
            .unwrap();
        assert_eq!(
            status.text.lines().count(),
            4,
            "counts, where work stopped, what else is flying, and the budget line: {}",
            status.text
        );
        assert!(
            status.text.contains("  \u{2026} 7 more active\n"),
            "{}",
            status.text
        );
    }

    #[test]
    fn an_unbounded_budget_prints_every_section_and_the_no_ceiling_line() {
        let full = rendered(Budget::Unbounded);
        for section in [
            "active: task.flight",
            "log: \"- 2026-08-25 claude: stopped at the ladder\"",
            "review[1]{id,taken-by,to}:\n  task.waiting,-,-\n",
            "ready[7]{id,priority,age,taken-by,title}:",
            "  … 2 more: anb ready",
            "questions[1]{id,age,by,to,title}:\n  question.aged,26d,-,-,A demo record\n",
            "debt: 1 — anb debt",
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
        assert!(
            !full.text.contains("decision.rule"),
            "a rule is read by a listing, never pushed into the opening: {}",
            full.text
        );
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

    /// The rows under one section header, wherever the ladder has moved
    /// the sections around it; the hint that counts the rest is no row.
    fn rows_under(text: &str, header: &str) -> usize {
        text.lines()
            .skip_while(|line| !line.starts_with(header))
            .skip(1)
            .take_while(|line| line.starts_with("  "))
            .filter(|line| !line.contains('\u{2026}'))
            .count()
    }

    #[test]
    fn the_questions_block_collapses_to_a_count_that_names_the_way_back() {
        let collapsed = (1..=400)
            .rev()
            .map(|ceiling| rendered(Budget::Tokens(ceiling)))
            .find(|step| {
                step.text
                    .contains("questions: 1 — anb list --type question")
            })
            .expect("some ceiling collapses the questions and keeps the rest");
        assert!(
            collapsed.text.contains("questions\u{2192}count"),
            "the budget line says what it cut: {}",
            collapsed.text
        );
        assert!(
            collapsed.text.contains("log: "),
            "the questions collapse before the log goes: {}",
            collapsed.text
        );
    }

    /// The ladder's fixed order, read off a descending budget sweep: ready
    /// rows go first, then the questions collapse, then the log line —
    /// never the other way around — and the text fits every budget the
    /// floor has not been forced past.
    #[test]
    fn sections_degrade_in_the_fixed_order_as_the_budget_shrinks() {
        let mut stages: Vec<(usize, bool, bool)> = Vec::new();
        for ceiling in (1..=400).rev() {
            let step = rendered(Budget::Tokens(ceiling));
            let ready_rows = rows_under(&step.text, "ready[");
            let questions_itemized = step.text.contains("questions[");
            let has_log = step.text.contains("log: ");
            let floor = !step.text.contains("ready") && !step.text.contains("debt");

            assert!(
                questions_itemized || ready_rows == 0,
                "questions collapsed while ready rows remain at {ceiling}: {}",
                step.text
            );
            assert!(
                has_log || !questions_itemized,
                "the log dropped before questions collapsed at {ceiling}: {}",
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
            let stage = (ready_rows, questions_itemized, has_log);
            if stages.last() != Some(&stage) {
                stages.push(stage);
            }
        }
        assert_eq!(stages.first(), Some(&(5, true, true)), "{stages:?}");
        assert_eq!(stages.last(), Some(&(0, false, false)), "{stages:?}");
        assert!(
            stages.len() >= 5,
            "the sweep must step down every rung the ladder has, not skip them: {stages:?}"
        );
        let mut previous = stages[0];
        for stage in &stages {
            let stage = *stage;
            assert!(
                stage.0 <= previous.0 && (!stage.1 || previous.1) && (!stage.2 || previous.2),
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
                    .status(TODAY, ceiling, None, no_lost_proofs)
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
    fn a_cut_note_names_only_what_the_dashboard_had() {
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
                .status(TODAY, Budget::Tokens(ceiling), None, no_lost_proofs)
                .unwrap();
            if let Some(cut) = status.text.split("cut: ").nth(1) {
                assert!(
                    !cut.contains("log"),
                    "no log line existed to cut at {ceiling}: {}",
                    status.text
                );
                assert!(
                    !cut.contains("active"),
                    "no Task was active to keep at {ceiling}: {}",
                    status.text
                );
            }
        }
    }

    #[test]
    fn the_floor_keeps_counts_the_active_line_and_the_budget_line() {
        let floor = rendered(Budget::Tokens(1));
        let lines: Vec<&str> = floor.text.lines().collect();
        assert_eq!(lines.len(), 3, "{}", floor.text);
        assert!(lines[0].starts_with("ok: notebook — "), "{}", floor.text);
        assert!(
            lines[1].starts_with("active: task.flight"),
            "{}",
            floor.text
        );
        assert!(
            lines[2].starts_with(&format!(
                "budget: ~{}/1 tokens; cut: all but the first active",
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
            .status(TODAY, Budget::Tokens(1), None, no_lost_proofs)
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
    /// review, log and questions sections existed, and the pin grants
    /// those plus the estimator's overshoot their room. Growth past it is
    /// dashboard bloat, and a budget-line regression is a failure.
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
            .status(TODAY, Budget::Tokens(1500), None, no_lost_proofs)
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
