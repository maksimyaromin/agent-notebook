mod status_budget {
    use crate::*;

    #[test]
    fn core_keeps_complete_ordered_lists_at_every_display_budget() {
        let files = (0..60).map(|n| {
            let id = format!("task.ready{n:02}");
            (
                format!("tasks/{id}.md"),
                record_file(&id, "task", "open", &[], ""),
            )
        });
        let mut storage = MemoryStorage::from_files(files);
        for budget in [Budget::Tokens(1), Budget::Tokens(1500), Budget::Unbounded] {
            let seen = Notebook::new(&mut storage)
                .status(TODAY, budget, None, no_lost_proofs)
                .unwrap();
            assert_eq!(seen.budget, budget);
            assert_eq!(seen.ready.len(), 60);
            assert_eq!(seen.ready[0].id, "task.ready00");
            assert_eq!(seen.ready[59].id, "task.ready59");
        }
    }

    #[test]
    fn display_budgets_do_not_truncate_core_record_text() {
        let title = "Long title ".repeat(1000);
        let source = record_file(
            "task.demo",
            "task",
            "active",
            &[],
            "- 2026-08-26 Ada: continue here\n",
        )
        .replace("title: A demo record", &format!("title: {title}"));
        let mut storage = storage_with(&[("tasks/task.demo.md", &source)]);
        let seen = Notebook::new(&mut storage)
            .status(TODAY, Budget::Tokens(1), None, no_lost_proofs)
            .unwrap();
        assert_eq!(seen.active[0].title, title.trim_end());
        assert_eq!(
            seen.active[0].log.as_deref(),
            Some("- 2026-08-26 Ada: continue here")
        );
    }

    #[test]
    fn a_zero_ceiling_disables_the_display_budget() {
        assert_eq!(Budget::from_ceiling(0), Budget::Unbounded);
        assert_eq!(Budget::from_ceiling(1), Budget::Tokens(1));
    }
}
