use crate::*;

#[test]
fn create_mints_the_id_from_the_title_and_writes_the_canonical_file() {
    let mut storage = MemoryStorage::new();
    let mut draft = Draft::new(RecordType::Task, "Grammar parser accepts fenced envelopes");
    draft.by = Some("supolka".to_owned());
    draft.via = Some("claude-code".to_owned());
    draft.tags = vec!["core".to_owned(), "parser".to_owned()];
    draft.priority = Some(1);
    draft.body = "The why before the what.".to_owned();

    let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
    assert_eq!(created.id, "task.grammar-parser-accepts-fenced-envelopes");
    assert_eq!(created.superseded, None);
    assert_eq!(
        storage.read(&created.path).unwrap(),
        "---\n\
         id: task.grammar-parser-accepts-fenced-envelopes\n\
         type: task\n\
         state: open\n\
         title: Grammar parser accepts fenced envelopes\n\
         by: supolka\n\
         via: claude-code\n\
         tags: core, parser\n\
         priority: 1\n\
         created: 2026-08-27\n\
         updated: 2026-08-27\n\
         ---\n\
         \n\
         The why before the what.\n"
    );
}

#[test]
fn a_caller_supplied_id_that_is_taken_names_its_holder() {
    let mut storage = storage_with(&[("archive/tasks/task.demo.md", &task_file("closed", &[]))]);
    let mut draft = Draft::new(RecordType::Task, "Another demo");
    draft.id = Some("task.demo".to_owned());
    let error = Notebook::new(&mut storage)
        .create(&draft, TODAY)
        .unwrap_err();
    assert_eq!(
        error,
        NotebookError::DuplicateId {
            id: "task.demo".to_owned(),
            holder: "archive/tasks/task.demo.md".to_owned(),
        },
        "ids are never reused, archive included"
    );
}

#[test]
fn a_read_record_claims_the_name_in_it_as_well_as_the_one_on_it() {
    let mut storage = storage_with(&[(
        "notes/note.misnamed.md",
        &record_file("note.wanted", "note", "active", &[], ""),
    )]);
    let mut draft = Draft::new(RecordType::Note, "The wanted name");
    draft.id = Some("note.wanted".to_owned());
    assert_eq!(
        Notebook::new(&mut storage)
            .create(&draft, TODAY)
            .unwrap_err(),
        NotebookError::DuplicateId {
            id: "note.wanted".to_owned(),
            holder: "notes/note.misnamed.md".to_owned(),
        },
        "write-side uniqueness cannot trust the convention whose violation it guards against"
    );
}

#[test]
fn a_filed_record_claims_only_the_name_on_it_and_check_names_the_clash() {
    let mut storage = storage_with(&[(
        "archive/notes/note.misnamed.md",
        &record_file("note.wanted", "note", "retired", &[], ""),
    )]);
    let mut draft = Draft::new(RecordType::Note, "The wanted name");
    draft.id = Some("note.wanted".to_owned());
    let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
    assert_eq!(created.path, "notes/note.wanted.md");

    let clashing: Vec<String> = Notebook::new(&mut storage)
        .check()
        .unwrap()
        .into_iter()
        .filter(|located| located.finding.code == FindingCode::DuplicateId)
        .map(|located| located.path)
        .collect();
    assert_eq!(
        clashing,
        vec!["archive/notes/note.misnamed.md", "notes/note.wanted.md"],
        "the id the two files share is named against both of them"
    );
}

/// The id `create` mints for `title`, with nothing else in the way.
fn minted_from(title: &str) -> String {
    let mut storage = MemoryStorage::new();
    Notebook::new(&mut storage)
        .create(&Draft::new(RecordType::Task, title), TODAY)
        .unwrap()
        .id
}

#[test]
fn a_long_title_is_cut_at_a_word_boundary_so_every_kept_word_survives() {
    assert_eq!(
        minted_from("GitHub dev flow: Actions CI, fmt + clippy + tests"),
        "task.github-dev-flow-actions-ci-fmt-clippy"
    );
    assert_eq!(
        minted_from("Epic pattern: scoped queries + Status hub grouping"),
        "task.epic-pattern-scoped-queries-status-hub"
    );
}

#[test]
fn a_boundary_landing_on_the_cap_keeps_the_word_before_it() {
    // Forty characters of whole words, then one more word: the cut has
    // a boundary to take at the cap itself.
    let title = "aaaa bbbb cccc dddd eeee ffff gggg hhhhi jjjj";
    assert_eq!(
        minted_from(title),
        "task.aaaa-bbbb-cccc-dddd-eeee-ffff-gggg-hhhhi"
    );
}

#[test]
fn a_first_word_longer_than_the_cap_is_cut_short_for_want_of_a_boundary() {
    let id = minted_from(&"z".repeat(60));
    assert_eq!(id, format!("task.{}", "z".repeat(40)));
}

#[test]
fn a_mint_collision_retries_with_a_two_character_suffix() {
    let mut storage = MemoryStorage::new();
    let draft = Draft::new(RecordType::Task, "A demo record");
    let first = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
    let second = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
    assert_eq!(first.id, "task.a-demo-record");
    assert_eq!(second.id.len(), first.id.len() + 3);
    assert!(second.id.starts_with("task.a-demo-record-"));
    assert!(storage.read(&second.path).is_ok());
}

#[test]
fn a_decision_created_with_supersedes_flips_its_victim_in_the_same_move() {
    let mut storage = storage_with(&[(
        "decisions/decision.go-for-the-cli.md",
        &record_file(
            "decision.go-for-the-cli",
            "decision",
            "active",
            &[],
            "Go.\n",
        ),
    )]);
    let mut draft = Draft::new(RecordType::Decision, "Rust for the CLI");
    draft.kind = Some("shape".to_owned());
    draft.supersedes = Some("decision.go-for-the-cli".to_owned());

    let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
    assert_eq!(
        created.superseded,
        Some("decision.go-for-the-cli".to_owned())
    );
    assert_eq!(
        storage
            .read("decisions/decision.go-for-the-cli.md")
            .unwrap(),
        "---\nid: decision.go-for-the-cli\ntype: decision\nstate: superseded\ntitle: A demo record\nsuperseded-by: decision.rust-for-the-cli\ncreated: 2026-08-24\nupdated: 2026-08-27\n---\nGo.\n",
        "the victim gains the back-pointer and can never again read as live"
    );
    let successor = storage.read(&created.path).unwrap();
    assert!(
        successor.contains("supersedes: decision.go-for-the-cli\n"),
        "the successor carries its claim: {successor}"
    );
    assert!(
        successor.contains("kind: shape\n"),
        "and the kind the draft asked for: {successor}"
    );
}

#[test]
fn a_note_superseded_by_its_successor_retires() {
    let mut storage = storage_with(&[(
        "notes/note.old-fact.md",
        &record_file("note.old-fact", "note", "active", &[], ""),
    )]);
    let mut draft = Draft::new(RecordType::Note, "The corrected fact");
    draft.kind = Some("fact".to_owned());
    draft.supersedes = Some("note.old-fact".to_owned());
    Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
    assert!(
        storage
            .read("notes/note.old-fact.md")
            .unwrap()
            .contains("state: retired")
    );
}

#[test]
fn a_victim_already_superseded_names_its_standing_superseder() {
    let mut storage = storage_with(&[
        (
            "decisions/decision.demo.md",
            &record_file(
                "decision.demo",
                "decision",
                "superseded",
                &["superseded-by: decision.newer"],
                "",
            ),
        ),
        (
            "decisions/decision.newer.md",
            &record_file(
                "decision.newer",
                "decision",
                "active",
                &["supersedes: decision.demo"],
                "",
            ),
        ),
    ]);
    let mut draft = Draft::new(RecordType::Decision, "A third ruling");
    draft.supersedes = Some("decision.demo".to_owned());
    let error = Notebook::new(&mut storage)
        .create(&draft, TODAY)
        .unwrap_err();
    let NotebookError::CannotSupersede { id, reason } = error else {
        panic!("expected CannotSupersede, got {error:?}");
    };
    assert_eq!(id, "decision.demo");
    assert!(reason.contains("decision.newer"));
}

#[test]
fn a_victim_excluded_from_mutation_cannot_be_superseded_either() {
    let victim = record_file(
        "decision.demo",
        "decision",
        "active",
        &["from: task.never-written"],
        "",
    );
    let mut storage = storage_with(&[("decisions/decision.demo.md", &victim)]);
    let mut draft = Draft::new(RecordType::Decision, "A newer ruling");
    draft.supersedes = Some("decision.demo".to_owned());
    let error = Notebook::new(&mut storage)
        .create(&draft, TODAY)
        .unwrap_err();
    assert!(matches!(error, NotebookError::InvalidRecord { .. }));
    assert_eq!(
        storage.read("decisions/decision.demo.md").unwrap(),
        victim,
        "a record excluded from mutation is never rewritten"
    );
    assert!(
        storage
            .read("decisions/decision.a-newer-ruling.md")
            .is_err(),
        "a refused supersession creates nothing"
    );
}

#[test]
fn a_task_cannot_be_superseded() {
    let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
    let mut draft = Draft::new(RecordType::Decision, "A ruling over a task");
    draft.supersedes = Some("task.demo".to_owned());
    let error = Notebook::new(&mut storage)
        .create(&draft, TODAY)
        .unwrap_err();
    assert!(matches!(error, NotebookError::CannotSupersede { .. }));
}

#[test]
fn a_draft_of_a_type_that_cannot_die_by_supersession_cannot_declare_it() {
    let mut draft = Draft::new(RecordType::Task, "A task claiming supersession");
    draft.supersedes = Some("task.demo".to_owned());
    let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("open", &[]))]);
    let error = Notebook::new(&mut storage)
        .create(&draft, TODAY)
        .unwrap_err();
    assert!(matches!(error, NotebookError::InvalidArgument { .. }));
}

#[test]
fn an_origin_that_does_not_exist_is_a_dangling_ref() {
    let mut storage = MemoryStorage::new();
    let mut draft = Draft::new(RecordType::Question, "A doubt from nowhere");
    draft.from = Some("task.never-written".to_owned());
    let error = Notebook::new(&mut storage)
        .create(&draft, TODAY)
        .unwrap_err();
    assert_eq!(
        error,
        NotebookError::DanglingRef {
            field: "from",
            target: "task.never-written".to_owned(),
        }
    );
}

#[test]
fn an_archived_origin_still_counts_as_existing() {
    let mut storage = storage_with(&[("archive/tasks/task.shipped.md", &task_file("closed", &[]))]);
    let mut draft = Draft::new(RecordType::Note, "Knowledge born from shipped work");
    draft.kind = Some("fact".to_owned());
    draft.from = Some("task.shipped".to_owned());
    assert!(Notebook::new(&mut storage).create(&draft, TODAY).is_ok());
}

/// An envelope field is one line, and every identity a caller may set goes
/// onto one: a value carrying a newline would write envelope lines of its
/// own — a `by` that closes the record and reopens it as something else.
#[test]
fn an_identity_field_carrying_a_second_line_is_refused() {
    for (field, forged) in [
        ("by", "Maks\nstate: closed"),
        ("via", "claude-code\nstate: closed"),
        ("kind", "rule\nstate: closed"),
    ] {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Decision, "A ruling");
        match field {
            "by" => draft.by = Some(forged.to_owned()),
            "via" => draft.via = Some(forged.to_owned()),
            _ => draft.kind = Some(forged.to_owned()),
        }
        assert!(
            matches!(
                Notebook::new(&mut storage)
                    .create(&draft, TODAY)
                    .unwrap_err(),
                NotebookError::InvalidArgument { .. }
            ),
            "{field}"
        );
        assert_eq!(storage.list("decisions").unwrap(), Vec::<String>::new());
    }
}

#[test]
fn a_kind_outside_the_types_enum_is_refused_naming_the_set() {
    let mut storage = MemoryStorage::new();
    let mut draft = Draft::new(RecordType::Decision, "A ruling");
    draft.kind = Some("law".to_owned());
    let error = Notebook::new(&mut storage)
        .create(&draft, TODAY)
        .unwrap_err();
    let NotebookError::InvalidArgument { reason } = error else {
        panic!("expected InvalidArgument, got {error:?}");
    };
    assert!(reason.contains("rule, shape, drift"));
}

#[test]
fn a_kind_on_a_kindless_type_is_refused() {
    let mut storage = MemoryStorage::new();
    let mut draft = Draft::new(RecordType::Task, "A task");
    draft.kind = Some("feature".to_owned());
    assert!(matches!(
        Notebook::new(&mut storage)
            .create(&draft, TODAY)
            .unwrap_err(),
        NotebookError::InvalidArgument { .. }
    ));
}

#[test]
fn links_render_as_kind_target_lines() {
    let mut storage = MemoryStorage::new();
    let mut draft = Draft::new(RecordType::Task, "A linked task");
    draft.links = vec![Link {
        kind: "doc".to_owned(),
        target: "docs/format.md".to_owned(),
    }];
    let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
    assert!(
        storage
            .read(&created.path)
            .unwrap()
            .contains("link: doc docs/format.md\n")
    );
}

mod conflict_nudge {
    use crate::*;
    use anb_core::Cited;

    fn decision_draft(tags: &[&str]) -> Draft {
        let mut draft = Draft::new(RecordType::Decision, "A second ruling");
        draft.tags = tags.iter().map(|tag| (*tag).to_owned()).collect();
        draft
    }

    fn standing_decision(extra_lines: &[&str]) -> (&'static str, String) {
        (
            "decisions/decision.first.md",
            record_file("decision.first", "decision", "active", extra_lines, ""),
        )
    }

    #[test]
    fn a_decision_sharing_two_tags_with_the_draft_is_named_with_its_authors() {
        let (path, text) =
            standing_decision(&["by: supolka", "via: claude-code", "tags: parser, grammar"]);
        let mut storage = storage_with(&[(path, &text)]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["grammar", "cli", "parser"]), TODAY)
            .unwrap();
        assert_eq!(
            created.may_conflict,
            vec![Cited {
                id: "decision.first".to_owned(),
                by: Some("supolka".to_owned()),
                via: Some("claude-code".to_owned()),
            }]
        );
    }

    #[test]
    fn a_decision_cited_in_the_draft_body_is_named() {
        let (path, text) = standing_decision(&[]);
        let mut storage = storage_with(&[(path, &text)]);
        let mut draft = decision_draft(&[]);
        draft.body = "Refines decision.first for fenced blocks.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.may_conflict.len(), 1);
        assert_eq!(created.may_conflict[0].id, "decision.first");
    }

    /// One tag in common is not a conflict, however either side counts it:
    /// a duplicate is one tag, on the draft or on the record standing.
    #[test]
    fn one_shared_tag_is_not_a_conflict_hint() {
        for (case, standing, drafted) in [
            ("one each", "parser, grammar", vec!["parser", "cli"]),
            (
                "twice on the draft",
                "parser, grammar",
                vec!["parser", "parser"],
            ),
            ("twice on the record", "parser, parser", vec!["parser"]),
        ] {
            let (path, text) = standing_decision(&[&format!("tags: {standing}")]);
            let mut storage = storage_with(&[(path, &text)]);
            let created = Notebook::new(&mut storage)
                .create(&decision_draft(&drafted), TODAY)
                .unwrap();
            assert_eq!(created.may_conflict, vec![], "{case}");
        }
    }

    #[test]
    fn a_declared_supersession_carries_no_nudge() {
        let (path, text) = standing_decision(&["tags: parser, grammar"]);
        let mut storage = storage_with(&[(path, &text)]);
        let mut draft = decision_draft(&["parser", "grammar"]);
        draft.supersedes = Some("decision.first".to_owned());
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.superseded, Some("decision.first".to_owned()));
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn only_a_live_decision_is_named() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.retired.md",
                &record_file(
                    "decision.retired",
                    "decision",
                    "retired",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
            (
                "archive/decisions/decision.archived.md",
                &record_file(
                    "decision.archived",
                    "decision",
                    "active",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
            (
                "decisions/decision.live.md",
                &record_file(
                    "decision.live",
                    "decision",
                    "active",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
        ]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser", "grammar"]), TODAY)
            .unwrap();
        assert_eq!(created.may_conflict.len(), 1);
        assert_eq!(created.may_conflict[0].id, "decision.live");
    }

    #[test]
    fn a_decision_excluded_from_derived_queries_is_not_named() {
        let (path, text) =
            standing_decision(&["from: task.never-written", "tags: parser, grammar"]);
        let mut storage = storage_with(&[(path, &text)]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser", "grammar"]), TODAY)
            .unwrap();
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn a_note_draft_is_never_nudged() {
        let (path, text) = standing_decision(&["tags: parser, grammar"]);
        let mut storage = storage_with(&[(path, &text)]);
        let mut draft = Draft::new(RecordType::Note, "A fact beside the rulings");
        draft.kind = Some("fact".to_owned());
        draft.tags = vec!["parser".to_owned(), "grammar".to_owned()];
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn only_decisions_are_candidates() {
        let mut storage = storage_with(&[(
            "notes/note.first.md",
            &record_file(
                "note.first",
                "note",
                "active",
                &["tags: parser, grammar"],
                "",
            ),
        )]);
        let mut draft = decision_draft(&["parser", "grammar"]);
        draft.body = "See note.first.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.may_conflict, vec![]);
    }

    #[test]
    fn candidates_arrive_oldest_first() {
        let older = record_file(
            "decision.z-old",
            "decision",
            "active",
            &["tags: parser, grammar"],
            "",
        )
        .replace("created: 2026-08-24", "created: 2026-08-20");
        let mut storage = storage_with(&[
            ("decisions/decision.z-old.md", &older),
            (
                "decisions/decision.a-new.md",
                &record_file(
                    "decision.a-new",
                    "decision",
                    "active",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
        ]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser", "grammar"]), TODAY)
            .unwrap();
        let named: Vec<&str> = created
            .may_conflict
            .iter()
            .map(|cited| cited.id.as_str())
            .collect();
        assert_eq!(named, vec!["decision.z-old", "decision.a-new"]);
    }

    #[test]
    fn candidates_laid_down_the_same_day_order_by_id() {
        let mut storage = storage_with(&[
            (
                "decisions/decision.second.md",
                &record_file(
                    "decision.second",
                    "decision",
                    "active",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
            (
                "decisions/decision.first.md",
                &record_file(
                    "decision.first",
                    "decision",
                    "active",
                    &["tags: parser, grammar"],
                    "",
                ),
            ),
        ]);
        let created = Notebook::new(&mut storage)
            .create(&decision_draft(&["parser", "grammar"]), TODAY)
            .unwrap();
        let named: Vec<&str> = created
            .may_conflict
            .iter()
            .map(|cited| cited.id.as_str())
            .collect();
        assert_eq!(named, vec!["decision.first", "decision.second"]);
    }
}

mod mention_nudge {
    use crate::*;

    #[test]
    fn a_body_citing_no_record_warns_in_the_reply_and_still_lands() {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Task, "A demo record");
        draft.body = "Blocked by task.ghost until the spike lands.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.dangling_mentions, vec!["task.ghost"]);
        assert!(
            storage.read(&created.path).is_ok(),
            "a forward reference is legal, so the record is written"
        );
    }

    #[test]
    fn a_citation_that_resolves_live_or_archived_warns_nothing() {
        let mut storage = storage_with(&[
            ("tasks/task.live.md", &task_file("open", &[])),
            (
                "archive/tasks/task.done.md",
                &record_file("task.done", "task", "closed", &[], ""),
            ),
        ]);
        let mut draft = Draft::new(RecordType::Task, "A demo record");
        draft.body = "Follows task.live and task.done.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.dangling_mentions, Vec::<String>::new());
    }

    #[test]
    fn a_body_citing_the_record_it_creates_warns_nothing() {
        let mut storage = MemoryStorage::new();
        let mut draft = Draft::new(RecordType::Task, "A demo record");
        draft.id = Some("task.selfware".to_owned());
        draft.body = "task.selfware tracks its own scope.".to_owned();
        let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        assert_eq!(created.dangling_mentions, Vec::<String>::new());
    }

    #[test]
    fn a_comment_citing_no_record_warns_in_the_reply_and_still_logs() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        let reply = Notebook::new(&mut storage)
            .comment("task.demo", None, "waits on task.ghost", TODAY)
            .unwrap();
        assert_eq!(reply.dangling_mentions, vec!["task.ghost"]);
        assert!(
            storage
                .read("tasks/task.demo.md")
                .unwrap()
                .contains("waits on task.ghost"),
            "a warning is not a rejection"
        );
    }

    #[test]
    fn a_replayed_comment_carries_the_same_nudge() {
        let mut storage = storage_with(&[("tasks/task.demo.md", &task_file("active", &[]))]);
        Notebook::new(&mut storage)
            .comment("task.demo", None, "waits on task.ghost", TODAY)
            .unwrap();
        let replay = Notebook::new(&mut storage)
            .comment("task.demo", None, "waits on task.ghost", TODAY)
            .unwrap();
        assert!(replay.already);
        assert_eq!(
            replay.dangling_mentions,
            vec!["task.ghost"],
            "the entry stands as the trail's tail, so its citations stand too"
        );
    }

    #[test]
    fn a_drop_reason_citing_no_record_warns_in_the_reply() {
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
            &record_file("question.demo", "question", "open", &[], ""),
        )]);
        let dropped = Notebook::new(&mut storage)
            .drop_question("question.demo", "absorbed into task.ghost", TODAY)
            .unwrap();
        assert_eq!(dropped.dangling_mentions, vec!["task.ghost"]);
    }

    #[test]
    fn a_replayed_drop_carries_no_nudge_for_a_reason_that_never_landed() {
        let mut storage = storage_with(&[(
            "questions/question.demo.md",
            &record_file("question.demo", "question", "open", &[], ""),
        )]);
        Notebook::new(&mut storage)
            .drop_question("question.demo", "a plain reason", TODAY)
            .unwrap();
        let replay = Notebook::new(&mut storage)
            .drop_question("question.demo", "absorbed into task.ghost", TODAY)
            .unwrap();
        assert!(replay.transition.already);
        assert_eq!(
            replay.dangling_mentions,
            Vec::<String>::new(),
            "the replay's reason wrote nothing, so it cites nothing"
        );
    }
}
