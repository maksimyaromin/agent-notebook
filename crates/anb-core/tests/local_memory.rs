use anb_core::{
    Draft, Filter, Focus, GraphSlice, Link, MemoryStorage, Notebook, NotebookError, RecordType,
};
use std::collections::BTreeSet;

#[test]
fn a_hub_can_be_started_before_its_children_but_not_completed_before_them() {
    let mut storage = MemoryStorage::new();
    let mut notebook = Notebook::new(&mut storage);
    for (id, origin) in [("task.hub", None), ("task.child", Some("task.hub"))] {
        let mut draft = Draft::new(RecordType::Task, id);
        draft.id = Some(id.to_owned());
        draft.from = origin.map(str::to_owned);
        notebook.create(&draft, "2026-09-12").unwrap();
    }
    notebook
        .block("task.hub", "task.child", "2026-09-12")
        .unwrap();
    notebook.start("task.hub", "2026-09-12").unwrap();
    assert_eq!(notebook.record("task.hub").unwrap().state(), Some("active"));
    let before = notebook.record("task.hub").unwrap().file().render();
    assert_eq!(
        notebook
            .close(
                "task.hub",
                None,
                "The overall result is ready.",
                "2026-09-12"
            )
            .unwrap_err(),
        NotebookError::UnfinishedDependencies {
            id: "task.hub".to_owned(),
            blockers: vec!["task.child".to_owned()],
        }
    );
    assert_eq!(notebook.record("task.hub").unwrap().file().render(), before);
    notebook.start("task.child", "2026-09-12").unwrap();
    notebook
        .close("task.child", None, "Child verified.", "2026-09-12")
        .unwrap();
    notebook.archive("task.child").unwrap();
    notebook
        .close("task.hub", None, "Overall result verified.", "2026-09-12")
        .unwrap();
    let closed = notebook.record("task.hub").unwrap().file().render();
    notebook.restore("task.child").unwrap();
    notebook.reopen("task.child", "2026-09-12").unwrap();
    assert!(
        notebook
            .close("task.hub", None, "Retry.", "2026-09-12")
            .unwrap()
            .transition
            .already
    );
    assert_eq!(notebook.record("task.hub").unwrap().file().render(), closed);
}

#[test]
fn subject_membership_excludes_dependencies_and_context_links_but_keeps_descendants() {
    let mut storage = MemoryStorage::new();
    let mut notebook = Notebook::new(&mut storage);
    for (id, origin, priority) in [
        ("task.subject", None, 2),
        ("task.external", None, 0),
        ("task.linked", None, 0),
        ("task.blocked", Some("task.subject"), 1),
        ("task.member", Some("task.subject"), 2),
        ("task.leaf", Some("task.member"), 4),
    ] {
        let mut draft = Draft::new(RecordType::Task, id);
        draft.id = Some(id.to_owned());
        draft.from = origin.map(str::to_owned);
        draft.priority = Some(priority);
        if id == "task.linked" {
            draft.links.push(Link {
                kind: "context".to_owned(),
                target: "task.subject".to_owned(),
            });
        }
        notebook.create(&draft, "2026-09-12").unwrap();
    }
    notebook.start("task.subject", "2026-09-12").unwrap();
    notebook
        .block("task.blocked", "task.external", "2026-09-12")
        .unwrap();
    let filter = Filter {
        hub: Some("task.subject".to_owned()),
        ..Filter::default()
    };

    assert_eq!(
        notebook
            .list(&filter)
            .unwrap()
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        ["task.blocked", "task.leaf", "task.member", "task.subject"]
    );
    assert_eq!(
        notebook
            .ready(&filter)
            .unwrap()
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        ["task.member", "task.leaf"]
    );
    assert_eq!(
        notebook
            .start_next(&filter, &BTreeSet::new(), "2026-09-12")
            .unwrap()
            .unwrap()
            .id,
        "task.member"
    );
    assert_eq!(
        notebook.record("task.external").unwrap().state(),
        Some("open")
    );
    assert_eq!(
        notebook.record("task.linked").unwrap().state(),
        Some("open")
    );
}

#[test]
fn invalid_archived_focus_is_reported_without_ranking_its_claims() {
    let mut storage = MemoryStorage::from_files([(
        "archive/tasks/task.history.md",
        "---\nid: task.history\n---\n\nFollow note.evidence.\n",
    )]);
    let mut notebook = Notebook::new(&mut storage);
    let mut note = Draft::new(RecordType::Note, "Maintained evidence");
    note.id = Some("note.evidence".to_owned());
    notebook.create(&note, "2026-09-12").unwrap();

    let knowledge = notebook.recall(None, Some("task.history")).unwrap();

    assert_eq!(knowledge.invalid, ["archive/tasks/task.history.md"]);
    assert_eq!(knowledge.records.len(), 1);
    assert_eq!(knowledge.records[0].id, "note.evidence");
    assert!(!knowledge.records[0].related);
}

#[test]
fn an_archived_focus_prioritizes_live_knowledge_without_returning_history() {
    for record_type in [RecordType::Task, RecordType::Note] {
        let mut storage = MemoryStorage::new();
        let mut notebook = Notebook::new(&mut storage);
        for (kind, id) in [
            (RecordType::Note, "note.evidence"),
            (RecordType::Decision, "decision.rule"),
            (RecordType::Note, "note.history"),
        ] {
            let mut draft = Draft::new(kind, id);
            draft.id = Some(id.to_owned());
            notebook.create(&draft, "2026-09-12").unwrap();
        }
        notebook.retire("note.history", "2026-09-12").unwrap();
        notebook.archive("note.history").unwrap();
        let mut root = Draft::new(record_type, "The completed investigation");
        let id = format!("{}.investigation", record_type.word());
        root.id = Some(id.clone());
        root.body = "The maintained conclusion is in note.evidence.".to_owned();
        notebook.create(&root, "2026-09-12").unwrap();
        if record_type == RecordType::Task {
            notebook.start(&id, "2026-09-12").unwrap();
            notebook
                .close(&id, None, "Recorded the conclusion.", "2026-09-12")
                .unwrap();
        } else {
            notebook.retire(&id, "2026-09-12").unwrap();
        }
        notebook.archive(&id).unwrap();

        let knowledge = notebook.recall(None, Some(&id)).unwrap();

        assert_eq!(
            knowledge
                .records
                .iter()
                .map(|record| (record.id.as_str(), record.related))
                .collect::<Vec<_>>(),
            [("note.evidence", true), ("decision.rule", false)]
        );
        assert!(knowledge.invalid.is_empty());
    }
}

#[test]
fn prose_citations_connect_focus_but_quoted_examples_do_not() {
    let mut storage = MemoryStorage::new();
    let mut notebook = Notebook::new(&mut storage);
    for id in ["note.context", "note.quoted", "note.fenced"] {
        let mut draft = Draft::new(RecordType::Note, id);
        draft.id = Some(id.to_owned());
        notebook.create(&draft, "2026-09-12").unwrap();
    }
    let mut task = Draft::new(RecordType::Task, "Use the maintained context");
    task.id = Some("task.work".to_owned());
    task.body = "Follow note.context. The example `note.quoted` is not a relation.\n```text\nnote.fenced\n```\n".to_owned();
    notebook.create(&task, "2026-09-12").unwrap();

    let graph = notebook
        .graph(&GraphSlice {
            focus: Some(Focus {
                id: "task.work".to_owned(),
                depth: 1,
            }),
            ..GraphSlice::default()
        })
        .unwrap();

    assert_eq!(
        graph
            .nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        ["task.work", "note.context"]
    );
    assert_eq!(
        graph
            .edges()
            .iter()
            .map(|edge| (edge.from, edge.to, edge.kind.word()))
            .collect::<Vec<_>>(),
        [("task.work", "note.context", "mentions")]
    );
    assert_eq!(
        notebook
            .recall(None, Some("task.work"))
            .unwrap()
            .records
            .iter()
            .map(|record| (record.id.as_str(), record.related))
            .collect::<Vec<_>>(),
        [
            ("note.context", true),
            ("note.fenced", false),
            ("note.quoted", false)
        ]
    );
}

#[test]
fn recall_prioritizes_knowledge_from_the_same_origin_without_hiding_other_rules() {
    let mut storage = MemoryStorage::new();
    let mut notebook = Notebook::new(&mut storage);
    for (record_type, id, origin) in [
        (RecordType::Task, "task.hub", None),
        (RecordType::Task, "task.child", Some("task.hub")),
        (RecordType::Note, "note.evidence", Some("task.hub")),
        (RecordType::Decision, "decision.rule", None),
    ] {
        let mut draft = Draft::new(record_type, id);
        draft.id = Some(id.to_owned());
        draft.from = origin.map(str::to_owned);
        notebook.create(&draft, "2026-09-12").unwrap();
    }

    let knowledge = notebook.recall(None, Some("task.child")).unwrap();

    assert_eq!(
        knowledge
            .records
            .iter()
            .map(|record| (record.id.as_str(), record.related))
            .collect::<Vec<_>>(),
        [("note.evidence", true), ("decision.rule", false)]
    );
}

#[test]
fn citing_a_related_decision_does_not_imply_a_conflict() {
    let mut storage = MemoryStorage::new();
    let mut notebook = Notebook::new(&mut storage);
    let mut first = Draft::new(RecordType::Decision, "Keep account exports tenant-scoped");
    first.id = Some("decision.scope".to_owned());
    first.tags = vec!["exports".to_owned(), "security".to_owned()];
    notebook.create(&first, "2026-09-12").unwrap();
    let mut second = Draft::new(RecordType::Decision, "Audit every export request");
    second.tags = first.tags;
    second.body = "Supports decision.scope with an audit trail.".to_owned();
    notebook.create(&second, "2026-09-12").unwrap();
    assert!(
        notebook
            .debt("2026-09-12", |_| Vec::new())
            .unwrap()
            .is_empty()
    );
}
