mod status_dashboard {
    use crate::*;

    fn status_text(storage: &mut MemoryStorage) -> String {
        Notebook::new(storage)
            .status(TODAY, Budget::Unbounded, None, no_lost_proofs)
            .unwrap()
            .text
    }

    fn status_by(storage: &mut MemoryStorage, identity: &str, by: Option<&str>) -> String {
        Notebook::new(storage)
            .with_identity(Some(identity))
            .status(TODAY, Budget::Unbounded, by, no_lost_proofs)
            .unwrap()
            .text
    }

    fn question_file(id: &str, extra: &[&str]) -> String {
        record_file(id, "question", "open", extra, "")
    }

    /// A dashboard is derived, so its size is the dashboard's shape and
    /// not the notebook's: a title and a log line are a record's own text,
    /// and a hand writes them as long as it likes. Every line that carries
    /// one is cut; the whole line is one `show` away.
    #[test]
    fn no_dashboard_line_carries_a_record_s_text_whole() {
        let wall_of_words = "word ".repeat(400);
        let mut storage = storage_with(&[
            (
                "tasks/task.demo.md",
                &format!(
                    "---\nid: task.demo\ntype: task\nstate: active\ntitle: {wall_of_words}\ncreated: 2026-08-24\nupdated: 2026-08-25\n---\n- 2026-08-26 -: {wall_of_words}\n"
                ),
            ),
            (
                "questions/question.demo.md",
                &question_file("question.demo", &[])
                    .replace("title: A demo record", &format!("title: {wall_of_words}")),
            ),
        ]);
        let text = status_text(&mut storage);

        for prefix in ["active:", "log:", "  question.demo,"] {
            let line = text
                .lines()
                .find(|line| line.starts_with(prefix))
                .unwrap_or_else(|| panic!("no `{prefix}` line in: {text}"));
            assert!(
                line.chars().count() < wall_of_words.chars().count(),
                "the wall reached the dashboard: {line}"
            );
            assert!(line.contains("more characters"), "no elision: {line}");
        }
    }

    /// The dashboard is a derived query like every other: a record whose
    /// findings put it outside `ready` and `list` cannot lead a session
    /// from the active, review or questions line either. Each of the three
    /// reads the notebook through its own fold, so each can lose the gate
    /// on its own.
    #[test]
    fn an_invalid_record_reaches_no_line_of_the_dashboard() {
        for (case, path, id, type_word, state) in [
            (
                "active",
                "tasks/task.demo.md",
                "task.demo",
                "task",
                "active",
            ),
            (
                "review",
                "tasks/task.demo.md",
                "task.demo",
                "task",
                "review",
            ),
            (
                "questions",
                "questions/question.demo.md",
                "question.demo",
                "question",
                "open",
            ),
        ] {
            let sound = record_file(id, type_word, state, &[], "");
            let mut storage = storage_with(&[(path, &sound)]);
            let seen = Notebook::new(&mut storage)
                .status(TODAY, Budget::Unbounded, None, no_lost_proofs)
                .unwrap();
            let named: Vec<&str> = seen
                .active
                .iter()
                .map(|task| task.id.as_str())
                .chain(seen.review.iter().map(|task| task.id.as_str()))
                .chain(seen.questions.iter().map(|question| question.id.as_str()))
                .collect();
            assert_eq!(named, vec![id], "{case}: a sound record is on its line");

            // A reference no record answers excludes the record, and its
            // state is untouched, so only the gate can drop the line.
            let broken = record_file(id, type_word, state, &["from: task.ghost"], "");
            let mut storage = storage_with(&[(path, &broken)]);
            let seen = Notebook::new(&mut storage)
                .status(TODAY, Budget::Unbounded, None, no_lost_proofs)
                .unwrap();
            assert_eq!(seen.active, vec![], "{case}");
            assert_eq!(seen.review, vec![], "{case}");
            assert_eq!(seen.questions, vec![], "{case}");
        }
    }

    #[test]
    fn an_active_task_is_the_active_line_with_its_last_log_line() {
        let body = "Acceptance: the ladder holds.\n\n- 2026-08-25 claude: stopped at the ladder\n";
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            &record_file("task.demo", "task", "active", &[], body),
        )]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("active: task.demo \"A demo record\"\n"),
            "{text}"
        );
        assert!(
            text.contains("log: \"- 2026-08-25 claude: stopped at the ladder\"\n"),
            "{text}"
        );
    }

    /// A hold is a pause somebody chose, so the paused Task is not the one
    /// a session resumes from: it waits in its own section, with the
    /// reason it waits for and who took it, whatever state the hold froze
    /// it in.
    #[test]
    fn a_held_task_is_not_active_but_waits_with_its_reason() {
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
            (
                "tasks/task.shelved.md",
                &record_file(
                    "task.shelved",
                    "task",
                    "open",
                    &["hold: after the release"],
                    "",
                ),
            ),
        ]);
        let text = status_text(&mut storage);
        let active: Vec<&str> = text
            .lines()
            .filter(|line| line.starts_with("active: "))
            .collect();
        assert_eq!(active, ["active: task.working \"A demo record\""], "{text}");
        assert!(
            text.contains(
                "held[2]{id,reason,until,taken-by}:\n  task.parked,waits for the API key,2026-09-20,Grace\n  task.shelved,after the release,-,-\n"
            ),
            "{text}"
        );
    }

    /// A hold line left on settled work is not a wait: closing a held Task
    /// ends it, and the dashboard lists what still waits.
    #[test]
    fn a_closed_task_still_carrying_its_hold_line_is_not_held() {
        let mut storage = storage_with(&[
            (
                "tasks/task.parked.md",
                &record_file(
                    "task.parked",
                    "task",
                    "closed",
                    &["hold: waits for the API key", "closed: 2026-08-26"],
                    "",
                ),
            ),
            (
                "tasks/task.working.md",
                &record_file("task.working", "task", "active", &[], ""),
            ),
        ]);
        let text = status_text(&mut storage);
        assert!(!text.contains("held"), "{text}");
    }

    /// A reason is a value in a comma table, so one holding the delimiter
    /// is quoted the way a ready row quotes its title.
    #[test]
    fn a_hold_reason_holding_a_comma_is_quoted_in_its_row() {
        let mut storage = storage_with(&[
            (
                "tasks/task.parked.md",
                &record_file(
                    "task.parked",
                    "task",
                    "open",
                    &["hold: waiting on legal, still pending"],
                    "",
                ),
            ),
            (
                "tasks/task.working.md",
                &record_file("task.working", "task", "active", &[], ""),
            ),
        ]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("  task.parked,\"waiting on legal, still pending\",-,-\n"),
            "{text}"
        );
    }

    /// A notebook whose only work is on hold has nothing for a session to
    /// pick up, so the gate stays closed; a hold gone stale is Debt's to
    /// raise.
    #[test]
    fn a_notebook_whose_only_work_is_held_is_quiet() {
        let mut storage = storage_with(&[(
            "tasks/task.parked.md",
            &record_file(
                "task.parked",
                "task",
                "active",
                &["hold: waits for the API key"],
                "",
            ),
        )]);
        let text = status_text(&mut storage);
        assert!(text.starts_with("ok: notebook quiet"), "{text}");
    }

    /// The held section stands still with review: over budget it collapses
    /// to a count at the same rung, and the floor drops it altogether.
    #[test]
    fn the_held_section_collapses_with_review_and_leaves_the_floor() {
        let mut storage = storage_with(&[
            (
                "tasks/task.parked.md",
                &record_file(
                    "task.parked",
                    "task",
                    "active",
                    &["hold: waits for the API key"],
                    "",
                ),
            ),
            (
                "tasks/task.working.md",
                &record_file("task.working", "task", "active", &[], ""),
            ),
        ]);
        let floor = Notebook::new(&mut storage)
            .status(TODAY, Budget::Tokens(1), None, no_lost_proofs)
            .unwrap()
            .text;
        assert!(!floor.contains("held"), "{floor}");
        assert!(floor.contains("cut: all but the first active"), "{floor}");
    }

    /// Two active Tasks is a session that lost track of one of them, so
    /// the dashboard leads with the one last touched and spends its log
    /// line there.
    #[test]
    fn the_active_lines_lead_with_the_task_last_touched() {
        let mut storage = storage_with(&[
            (
                "tasks/task.stale.md",
                "---\nid: task.stale\ntype: task\nstate: active\ntitle: A demo record\ncreated: 2026-08-20\nupdated: 2026-08-21\n---\n- 2026-08-21 claude: parked here\n",
            ),
            (
                "tasks/task.fresh.md",
                "---\nid: task.fresh\ntype: task\nstate: active\ntitle: A demo record\ncreated: 2026-08-20\nupdated: 2026-08-27\n---\n- 2026-08-27 claude: resumed here\n",
            ),
        ]);
        let text = status_text(&mut storage);
        let active: Vec<&str> = text
            .lines()
            .filter(|line| line.starts_with("active: "))
            .collect();
        assert_eq!(
            active,
            [
                "active: task.fresh \"A demo record\"",
                "active: task.stale \"A demo record\""
            ],
            "{text}"
        );
        let logs: Vec<&str> = text
            .lines()
            .filter(|line| line.starts_with("log: "))
            .collect();
        assert_eq!(
            logs,
            ["log: \"- 2026-08-27 claude: resumed here\""],
            "one log line, and it belongs to the Task the session is on: {text}"
        );
    }

    /// Several people work one notebook, and each session opens on its own
    /// person's work: the reader's Tasks lead, whatever was touched last,
    /// and every other line says who holds it, so nobody resumes another
    /// person's Task by mistake. A Task nobody holds is nobody's to mark,
    /// whoever wrote it.
    #[test]
    fn the_active_lines_lead_with_the_readers_own_and_mark_the_others() {
        fn active(id: &str, updated: &str, extra: &[&str]) -> String {
            record_file(id, "task", "active", extra, "").replace("updated: 2026-08-25", updated)
        }
        let mut storage = storage_with(&[
            (
                "tasks/task.hers.md",
                &active("task.hers", "updated: 2026-08-27", &["taken-by: Grace"]),
            ),
            (
                "tasks/task.mine.md",
                &active(
                    "task.mine",
                    "updated: 2026-08-21",
                    &["by: Grace", "taken-by: Ada"],
                ),
            ),
            (
                "tasks/task.legacy.md",
                &active("task.legacy", "updated: 2026-08-26", &["by: Maks"]),
            ),
            (
                "tasks/task.nobodys.md",
                &active("task.nobodys", "updated: 2026-08-25", &[]),
            ),
        ]);
        let status = Notebook::new(&mut storage)
            .with_identity(Some("Ada"))
            .status(TODAY, Budget::Unbounded, None, no_lost_proofs)
            .unwrap();
        let active: Vec<&str> = status
            .text
            .lines()
            .filter(|line| line.starts_with("active: "))
            .collect();
        assert_eq!(
            active,
            [
                "active: task.mine \"A demo record\"",
                "active: task.hers \"A demo record\" (Grace)",
                "active: task.legacy \"A demo record\"",
                "active: task.nobodys \"A demo record\"",
            ],
            "{}",
            status.text
        );
        assert_eq!(
            status.active[0].attribution,
            anb_core::Attribution {
                by: Some("Grace".to_owned()),
                taken_by: Some("Ada".to_owned()),
            }
        );
    }

    /// Work standing still is somebody's too: the review list and the held
    /// table lead with the reader's own, and a line another person holds
    /// says whose, the way the active lines do.
    #[test]
    fn review_and_held_lead_with_the_readers_own_and_name_the_others() {
        let mut storage = storage_with(&[
            (
                "tasks/task.hers.md",
                &record_file("task.hers", "task", "review", &["taken-by: Grace"], ""),
            ),
            (
                "tasks/task.mine.md",
                &record_file("task.mine", "task", "review", &["taken-by: Ada"], ""),
            ),
            (
                "tasks/task.her-hold.md",
                &record_file(
                    "task.her-hold",
                    "task",
                    "open",
                    &["taken-by: Grace", "hold: waits on legal"],
                    "",
                ),
            ),
            (
                "tasks/task.my-hold.md",
                &record_file(
                    "task.my-hold",
                    "task",
                    "open",
                    &["taken-by: Ada", "hold: waits on the vendor"],
                    "",
                ),
            ),
        ]);
        let text = status_by(&mut storage, "Ada", None);
        assert!(
            text.contains("review[2]: task.mine, task.hers (Grace) — waiting on a human\n"),
            "{text}"
        );
        assert!(
            text.contains(
                "held[2]{id,reason,until,taken-by}:\n  task.my-hold,waits on the vendor,-,Ada\n  task.her-hold,waits on legal,-,Grace\n"
            ),
            "{text}"
        );
    }

    /// A host that knows nobody cannot call any line its own, so every
    /// line that names a person carries the name, and the order is the
    /// order of touch alone.
    #[test]
    fn without_an_identity_every_named_line_is_marked() {
        let mut storage = storage_with(&[
            (
                "tasks/task.hers.md",
                &record_file("task.hers", "task", "active", &["taken-by: Grace"], ""),
            ),
            (
                "tasks/task.nobodys.md",
                &record_file("task.nobodys", "task", "active", &[], ""),
            ),
        ]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("active: task.hers \"A demo record\" (Grace)\n"),
            "{text}"
        );
        assert!(
            text.contains("active: task.nobodys \"A demo record\"\n"),
            "{text}"
        );
    }

    /// An open doubt is what a session is about to work past, so the
    /// dashboard names it with who asked it: the reader's own first, then
    /// oldest first, since the oldest doubt has waited longest.
    #[test]
    fn open_questions_lead_with_the_readers_own_and_then_the_oldest() {
        let mut storage = storage_with(&[
            (
                "questions/question.hers.md",
                &question_file("question.hers", &["by: Grace"])
                    .replace("created: 2026-08-24", "created: 2026-08-20"),
            ),
            (
                "questions/question.mine.md",
                &question_file("question.mine", &["by: Ada"]),
            ),
            (
                "questions/question.old.md",
                &question_file("question.old", &[])
                    .replace("created: 2026-08-24", "created: 2026-08-10"),
            ),
            (
                "questions/question.settled.md",
                &record_file(
                    "question.settled",
                    "question",
                    "closed",
                    &["reason: moot", "closed: 2026-08-25"],
                    "",
                ),
            ),
        ]);
        let text = status_by(&mut storage, "Ada", None);
        assert!(
            text.contains(
                "questions[3]{id,age,by,title}:\n  question.mine,3d,Ada,A demo record\n  question.old,17d,-,A demo record\n  question.hers,7d,Grace,A demo record\n"
            ),
            "{text}"
        );
    }

    /// A Task parked at acceptance is a human's turn, and the session must
    /// open on it even when nothing else in the notebook stirs.
    #[test]
    fn the_dashboard_opens_on_a_task_waiting_for_a_human_alone() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("review", &[]))]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, None, no_lost_proofs)
            .unwrap();
        assert!(!status.quiet, "{}", status.text);
        assert!(
            status.text.contains("review[1]: task.demo"),
            "{}",
            status.text
        );
    }

    #[test]
    fn the_dashboard_opens_on_ready_work_alone() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, None, no_lost_proofs)
            .unwrap();
        assert!(!status.quiet);
        assert!(
            status
                .text
                .contains("ready[1]{id,priority,age,taken-by,title}:\n"),
            "{}",
            status.text
        );
    }

    /// A doubt nobody has answered is a signal on its own: the session
    /// opens on it before it works past it.
    #[test]
    fn the_dashboard_opens_on_an_open_question_alone() {
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
            &question_file("question.demo", &[]),
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, None, no_lost_proofs)
            .unwrap();
        assert!(!status.quiet, "{}", status.text);
        assert!(
            status
                .text
                .contains("questions[1]{id,age,by,title}:\n  question.demo,3d,-,A demo record\n"),
            "{}",
            status.text
        );
    }

    /// Decay opens the dashboard, and is answered with a count and the
    /// read that itemises it: the signals are the notebook's worst day, and
    /// a session's opening is not where to spend it.
    #[test]
    fn the_dashboard_opens_on_debt_alone_and_counts_it() {
        // A stale hold: work standing still is not a signal, its decay is.
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            "---\nid: task.demo\ntype: task\nstate: open\ntitle: A demo record\nhold: waits\ncreated: 2026-08-01\nupdated: 2026-08-01\n---\n",
        )]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, None, no_lost_proofs)
            .unwrap();
        assert!(!status.quiet);
        assert_eq!(status.debt, 1);
        assert!(
            status.text.contains("debt: 1 — anb debt\n"),
            "{}",
            status.text
        );
        assert!(
            !status.text.contains("hold-stale"),
            "the signal itself is the listing's: {}",
            status.text
        );
    }

    /// Knowledge binds nothing at session start: a rule is read before the
    /// work it governs, by a listing, and a notebook of rules, shapes and
    /// facts with no work in it stays quiet.
    #[test]
    fn knowledge_alone_leaves_the_notebook_quiet() {
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
                "notes/note.demo.md",
                &record_file("note.demo", "note", "active", &["kind: fact"], ""),
            ),
        ]);
        let status = Notebook::new(&mut storage)
            .status(TODAY, Budget::Unbounded, None, no_lost_proofs)
            .unwrap();
        assert!(status.quiet, "{}", status.text);
        assert!(!status.text.contains("decision.rule"), "{}", status.text);
    }

    /// A dashboard narrowed to one identity holds the Tasks that person
    /// holds and the Questions they asked, nothing else, says whose it is,
    /// and points every hint at the same narrowing, so a reader who follows
    /// one lands on the same rows.
    #[test]
    fn a_dashboard_narrowed_to_one_identity_holds_their_work_and_says_so() {
        let mut files = vec![
            (
                "tasks/task.mine.md".to_owned(),
                record_file("task.mine", "task", "active", &["taken-by: Ada"], ""),
            ),
            (
                "tasks/task.hers.md".to_owned(),
                record_file("task.hers", "task", "active", &["taken-by: Grace"], ""),
            ),
            (
                "tasks/task.her-review.md".to_owned(),
                record_file(
                    "task.her-review",
                    "task",
                    "review",
                    &["taken-by: Grace"],
                    "",
                ),
            ),
            (
                "questions/question.hers.md".to_owned(),
                question_file("question.hers", &["by: Grace"]),
            ),
            (
                "questions/question.mine.md".to_owned(),
                question_file("question.mine", &["by: Ada"]),
            ),
        ];
        for index in 0..7 {
            files.push((
                format!("tasks/task.r{index}.md"),
                record_file(
                    &format!("task.r{index}"),
                    "task",
                    "open",
                    &["by: Grace", "taken-by: Ada"],
                    "",
                ),
            ));
        }
        let mut storage = MemoryStorage::from_files(files);
        let status = Notebook::new(&mut storage)
            .with_identity(Some("Ada"))
            .status(TODAY, Budget::Unbounded, Some("Ada"), no_lost_proofs)
            .unwrap();
        let text = &status.text;
        assert!(
            text.starts_with(
                "ok: notebook — 10 tasks, 0 decisions, 0 notes, 2 questions\nby: Ada — anb status --team\nactive: task.mine \"A demo record\"\n"
            ),
            "{text}"
        );
        assert!(!text.contains("task.hers"), "{text}");
        assert!(!text.contains("review"), "{text}");
        assert!(!text.contains("question.hers"), "{text}");
        assert!(
            text.contains("  \u{2026} 2 more: anb ready --by Ada\n"),
            "{text}"
        );
        assert!(
            text.contains("questions[1]{id,age,by,title}:\n  question.mine,3d,Ada,A demo record\n"),
            "{text}"
        );
        assert_eq!(status.by.as_deref(), Some("Ada"));
    }

    /// A Task written by the reader and handed to a colleague is the
    /// colleague's in every section: it never opens the reader's session,
    /// and it is not in the reader's queue.
    #[test]
    fn a_task_the_reader_wrote_and_handed_over_is_not_in_their_dashboard() {
        let mut storage = storage_with(&[
            (
                "tasks/task.planned-for-grace.md",
                &record_file(
                    "task.planned-for-grace",
                    "task",
                    "active",
                    &["by: Ada", "taken-by: Grace"],
                    "",
                ),
            ),
            (
                "tasks/task.planned-for-nobody.md",
                &record_file("task.planned-for-nobody", "task", "open", &["by: Ada"], ""),
            ),
            (
                "tasks/task.planned-for-ada.md",
                &record_file(
                    "task.planned-for-ada",
                    "task",
                    "open",
                    &["by: Ada", "taken-by: Ada"],
                    "",
                ),
            ),
        ]);
        let text = status_by(&mut storage, "Ada", Some("Ada"));
        assert!(!text.contains("active:"), "{text}");
        assert!(
            text.contains("ready[1]{id,priority,age,taken-by,title}:\n  task.planned-for-ada,"),
            "{text}"
        );
        assert!(
            text.contains("untaken: 1 — anb ready --untaken\n"),
            "{text}"
        );
    }

    /// A dashboard narrowed to one identity hides the pool with the rest
    /// of the team's work, so it counts the pool where a session with an
    /// empty queue looks for its next work, and a pool alone keeps the
    /// notebook from reading as quiet. The whole team's dashboard lists the
    /// pool in its queue and needs no count.
    #[test]
    fn a_narrowed_dashboard_counts_the_pool_and_a_pool_alone_is_not_quiet() {
        let mut storage = storage_with(&[
            (
                "tasks/task.free.md",
                &record_file("task.free", "task", "open", &["by: Grace"], ""),
            ),
            (
                "tasks/task.blocked.md",
                &record_file(
                    "task.blocked",
                    "task",
                    "open",
                    &["blocked-by: task.free"],
                    "",
                ),
            ),
        ]);
        let status = Notebook::new(&mut storage)
            .with_identity(Some("Ada"))
            .status(TODAY, Budget::Unbounded, Some("Ada"), no_lost_proofs)
            .unwrap();
        assert!(!status.quiet, "{}", status.text);
        let lines: Vec<&str> = status.text.lines().collect();
        assert_eq!(
            lines[..3],
            [
                "ok: notebook — 2 tasks, 0 decisions, 0 notes, 0 questions",
                "by: Ada — anb status --team",
                "untaken: 1 — anb ready --untaken",
            ],
            "{}",
            status.text
        );
        assert!(lines[3].starts_with("budget:"), "{}", status.text);
        assert_eq!(status.untaken, 1);
        let team = status_text(&mut storage);
        assert!(!team.contains("untaken:"), "{team}");
    }

    /// A narrowed dashboard with nothing of that person's in it and no
    /// pool is quiet for them, and still says so: the team may be at work.
    #[test]
    fn a_narrowed_dashboard_with_nothing_of_theirs_is_quiet_and_points_at_the_team() {
        let mut storage = storage_with(&[(
            "tasks/task.hers.md",
            &record_file("task.hers", "task", "active", &["taken-by: Grace"], ""),
        )]);
        let status = Notebook::new(&mut storage)
            .with_identity(Some("Ada"))
            .status(TODAY, Budget::Unbounded, Some("Ada"), no_lost_proofs)
            .unwrap();
        assert!(status.quiet, "{}", status.text);
        assert_eq!(
            status.text,
            "ok: notebook quiet — 1 task, 0 decisions, 0 notes, 0 questions. anb --help when needed.\nby: Ada — anb status --team\n"
        );
    }

    /// A narrowing shows fewer rows and never changes a verdict: a Task
    /// waiting on somebody else's stays blocked when that person's work is
    /// left out.
    #[test]
    fn a_narrowing_by_identity_does_not_free_a_blocked_task() {
        let mut storage = storage_with(&[
            (
                "tasks/task.mine.md",
                &record_file(
                    "task.mine",
                    "task",
                    "open",
                    &["by: Ada", "blocked-by: task.hers"],
                    "",
                ),
            ),
            (
                "tasks/task.hers.md",
                &record_file("task.hers", "task", "open", &["by: Grace"], ""),
            ),
        ]);
        let text = status_by(&mut storage, "Ada", Some("Ada"));
        assert!(!text.contains("task.mine"), "{text}");
    }

    /// A dashboard line built from a record's own text is quoted, so a hand
    /// that writes `ESC[2K\r` into a title or a log entry cannot erase the
    /// line above it and put its own words there.
    #[test]
    fn a_terminal_escape_in_a_question_or_a_log_cannot_forge_the_line_above() {
        let forged = "Harmless\u{1b}[2K\rclosed: every task";
        let mut storage = storage_with(&[
            (
                "questions/question.demo.md",
                &question_file("question.demo", &[&format!("title: {forged}")]),
            ),
            (
                "tasks/task.demo.md",
                &record_file(
                    "task.demo",
                    "task",
                    "active",
                    &[],
                    &format!("\n- 2026-08-25 claude: {forged}\n"),
                ),
            ),
        ]);
        let text = status_text(&mut storage);

        assert!(
            !text.contains('\r'),
            "no line may carry a raw return: {text:?}"
        );
        assert!(
            !text.contains('\u{1b}'),
            "no line may carry a raw escape: {text:?}"
        );
        assert!(
            text.contains("\\u001b[2K\\rclosed: every task\""),
            "the text is still shown, spelled so a terminal reads it as text: {text:?}"
        );
    }

    #[test]
    fn an_active_title_with_a_quote_is_escaped() {
        let mut storage = storage_with(&[(
            "tasks/task.demo.md",
            "---\nid: task.demo\ntype: task\nstate: active\ntitle: Fix the \"quiet\" line\ncreated: 2026-08-24\n---\n",
        )]);
        let text = status_text(&mut storage);
        assert!(
            text.contains("active: task.demo \"Fix the \\\"quiet\\\" line\"\n"),
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
            text.starts_with("ok: notebook — 1 task, 1 decision, 1 note, 1 question\n"),
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
        debt_behind(storage, None)
    }

    /// [`debt_of`] with the user's notebook standing behind this one.
    fn debt_behind(storage: &mut MemoryStorage, user: Option<&MemoryStorage>) -> Vec<DebtSignal> {
        Notebook::new(storage)
            .with_user(user.map(|user| user as &dyn Storage))
            .debt(TODAY, no_lost_proofs)
            .unwrap()
    }

    fn debt_lines(storage: &mut MemoryStorage, user: Option<&MemoryStorage>) -> Vec<String> {
        lines_of(&debt_behind(storage, user))
    }

    /// [`debt_lines`] against a notebook of any medium, not only the twin.
    fn debt_behind_storage(storage: &mut MemoryStorage, user: &dyn Storage) -> Vec<String> {
        lines_of(
            &Notebook::new(storage)
                .with_user(Some(user))
                .debt(TODAY, no_lost_proofs)
                .unwrap(),
        )
    }

    fn lines_of(debt: &[DebtSignal]) -> Vec<String> {
        debt.iter().map(DebtSignal::line).collect()
    }

    /// A Decision of one notebook or the other, its author the one fact
    /// these cases vary beside its state.
    fn rule(id: &str, state: &str, by: &str, body: &str) -> String {
        record_file(
            id,
            "decision",
            state,
            &["kind: rule", &format!("by: {by}")],
            body,
        )
    }

    fn cites(target: &str) -> String {
        format!("This repository decides otherwise, against {target}.\n")
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
        let debt = debt_of(&mut storage);
        let pairs: Vec<&DebtSignal> = debt
            .iter()
            .filter(|signal| matches!(signal, DebtSignal::MayConflict { .. }))
            .collect();
        assert_eq!(pairs.len(), 1, "both directions of citation are one pair");
        assert!(
            lines_of(&debt).contains(
                &"may-conflict: decision.a (supolka) <-> decision.b (supolka/claude-code)"
                    .to_owned()
            ),
            "{debt:?}"
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
                DebtSignal::MayConflict { first, .. } => Some(first.id.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            pair_firsts,
            vec!["decision.old".to_owned(), "decision.newish".to_owned()]
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

    /// The user's notebook stands behind a project's: what it keeps
    /// standing is what a project rule can shadow.
    mod the_notebook_behind_this_one {
        use super::*;

        /// The tool prints both sides and stops: which rule governs is
        /// settled by residence, and who to argue with is the reader's to
        /// see.
        #[test]
        fn a_project_rule_citing_the_users_own_names_the_pair_and_its_authors() {
            let user = storage_with(&[(
                "decisions/decision.tabs.md",
                &rule("decision.tabs", "active", "Reader", ""),
            )]);
            let mut project = storage_with(&[(
                "decisions/decision.spaces.md",
                &rule(
                    "decision.spaces",
                    "active",
                    "Teammate",
                    &cites("decision.tabs"),
                ),
            )]);

            assert_eq!(
                debt_lines(&mut project, Some(&user)),
                vec!["shadow: decision.spaces (Teammate) <-> global decision.tabs (Reader)"],
                "the project rule leads: it is the one this repository follows"
            );
        }

        /// The edge a project rule declares to the user's is a link, since
        /// the envelope edges must be answered by this notebook; a shadow
        /// surfaces from it as from a citation in prose.
        #[test]
        fn a_project_rule_linking_the_users_own_names_the_pair_too() {
            let user = storage_with(&[(
                "decisions/decision.tabs.md",
                &rule("decision.tabs", "active", "Reader", ""),
            )]);
            let mut project = storage_with(&[(
                "decisions/decision.spaces.md",
                &record_file(
                    "decision.spaces",
                    "decision",
                    "active",
                    &["by: Teammate", "link: against decision.tabs"],
                    "",
                ),
            )]);
            assert_eq!(
                debt_lines(&mut project, Some(&user)),
                vec!["shadow: decision.spaces (Teammate) <-> global decision.tabs (Reader)"]
            );
        }

        /// One rule named in prose and declared as a link is one edge to the
        /// user's, so it is one pair, not two.
        #[test]
        fn a_rule_cited_in_prose_and_linked_names_the_pair_once() {
            let user = storage_with(&[(
                "decisions/decision.tabs.md",
                &rule("decision.tabs", "active", "Reader", ""),
            )]);
            let mut project = storage_with(&[(
                "decisions/decision.spaces.md",
                &record_file(
                    "decision.spaces",
                    "decision",
                    "active",
                    &["by: Teammate", "link: against decision.tabs"],
                    &cites("decision.tabs"),
                ),
            )]);
            assert_eq!(
                debt_lines(&mut project, Some(&user)),
                vec!["shadow: decision.spaces (Teammate) <-> global decision.tabs (Reader)"]
            );
        }

        /// A citation that resolves is not a citation of nothing, and which
        /// notebook resolves it does not change that.
        #[test]
        fn a_record_the_users_notebook_holds_is_no_dangling_mention() {
            let mut project = storage_with(&[(
                "decisions/decision.spaces.md",
                &rule(
                    "decision.spaces",
                    "active",
                    "Teammate",
                    &cites("note.practice"),
                ),
            )]);

            assert_eq!(
                debt_lines(&mut project, None),
                vec!["dangling-mention: decision.spaces -> note.practice"],
                "the project alone knows no such id"
            );
            let user = storage_with(&[(
                "notes/note.practice.md",
                &record_file("note.practice", "note", "active", &["kind: fact"], ""),
            )]);
            assert_eq!(
                debt_lines(&mut project, Some(&user)),
                Vec::<String>::new(),
                "and a reader standing here can open it"
            );
        }

        /// A rule that has been replaced or ended binds nobody, so nothing
        /// stands against it — but the id still names a record, and calling
        /// it missing would be false.
        #[test]
        fn a_rule_the_user_has_retired_is_shadowed_by_nothing_and_dangles_from_nothing() {
            let user = storage_with(&[(
                "decisions/decision.tabs.md",
                &rule("decision.tabs", "retired", "Reader", ""),
            )]);
            let mut project = storage_with(&[(
                "decisions/decision.spaces.md",
                &rule(
                    "decision.spaces",
                    "active",
                    "Teammate",
                    &cites("decision.tabs"),
                ),
            )]);

            assert_eq!(debt_lines(&mut project, Some(&user)), Vec::<String>::new());
        }

        /// A record the tool refuses to trust cannot be half of a pair it
        /// prints: the id it would be named by is one its own envelope
        /// disowns.
        #[test]
        fn an_invalid_rule_of_the_users_notebook_is_half_of_no_pair() {
            let user = storage_with(&[(
                "decisions/decision.tabs.md",
                &rule("decision.tabs", "active", "Reader", "")
                    .replace("state: active", "state: bogus"),
            )]);
            let mut project = storage_with(&[(
                "decisions/decision.spaces.md",
                &rule(
                    "decision.spaces",
                    "active",
                    "Teammate",
                    &cites("decision.tabs"),
                ),
            )]);

            assert_eq!(debt_lines(&mut project, Some(&user)), Vec::<String>::new());
        }

        /// Two rules of one notebook are that notebook's own pair, and a
        /// notebook read behind itself is still one notebook.
        #[test]
        fn a_pair_inside_one_notebook_is_never_a_shadow() {
            let mut project = storage_with(&[
                (
                    "decisions/decision.tabs.md",
                    &rule("decision.tabs", "active", "Reader", ""),
                ),
                (
                    "decisions/decision.spaces.md",
                    &rule(
                        "decision.spaces",
                        "active",
                        "Teammate",
                        &cites("decision.tabs"),
                    ),
                ),
            ]);
            let itself = project.clone();

            assert_eq!(
                debt_lines(&mut project, Some(&itself)),
                vec!["may-conflict: decision.spaces (Teammate) <-> decision.tabs (Reader)"],
                "a notebook read behind itself pairs with nobody across the scopes"
            );
        }

        /// A rule this project retired is history here, so it hides
        /// nothing: the user's rule still stands, and standing is what the
        /// pair is about.
        #[test]
        fn a_rule_this_project_archived_stops_hiding_the_users_own() {
            let user = storage_with(&[(
                "decisions/decision.tabs.md",
                &rule("decision.tabs", "active", "Reader", ""),
            )]);
            let mut project = storage_with(&[
                (
                    "archive/decisions/decision.tabs.md",
                    &rule("decision.tabs", "retired", "Teammate", ""),
                ),
                (
                    "decisions/decision.spaces.md",
                    &rule(
                        "decision.spaces",
                        "active",
                        "Teammate",
                        &cites("decision.tabs"),
                    ),
                ),
            ]);

            assert_eq!(
                debt_lines(&mut project, Some(&user)),
                vec!["shadow: decision.spaces (Teammate) <-> global decision.tabs (Reader)"]
            );
        }

        /// Debt is read top to bottom, and a body is written in an order
        /// its author chose.
        #[test]
        fn the_pairs_of_one_record_arrive_in_the_order_its_body_names_them() {
            let user = storage_with(&[
                (
                    "decisions/decision.zebra.md",
                    &rule("decision.zebra", "active", "Reader", ""),
                ),
                (
                    "decisions/decision.alpha.md",
                    &rule("decision.alpha", "active", "Reader", ""),
                ),
            ]);
            let mut project = storage_with(&[(
                "decisions/decision.spaces.md",
                &rule(
                    "decision.spaces",
                    "active",
                    "Teammate",
                    "Against decision.zebra first, then decision.alpha.\n",
                ),
            )]);

            assert_eq!(
                debt_lines(&mut project, Some(&user)),
                vec![
                    "shadow: decision.spaces (Teammate) <-> global decision.zebra (Reader)",
                    "shadow: decision.spaces (Teammate) <-> global decision.alpha (Reader)",
                ]
            );
        }

        /// A second root is read for a hint on somebody else's dashboard.
        /// A project whose session opens with a Status cannot be stopped by
        /// the state of a notebook it does not own.
        #[test]
        fn a_user_notebook_that_cannot_be_read_leaves_the_dashboard_standing() {
            let mut project = storage_with(&[(
                "decisions/decision.spaces.md",
                &rule(
                    "decision.spaces",
                    "active",
                    "Teammate",
                    &cites("decision.tabs"),
                ),
            )]);
            let alone = debt_lines(&mut project.clone(), None);

            assert_eq!(
                debt_behind_storage(&mut project, &UnreadableNotebook),
                alone,
                "the hint is left out, and nothing else changes"
            );
        }
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
