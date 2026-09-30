//! A record bound to a Task leaves for the archive with it and returns
//! with it; a record restored alone outlives it.

use crate::*;

/// A design Task in `state`, and a draft bound to it.
fn design_notebook(state: &str) -> MemoryStorage {
    storage_with(&[(
        "tasks/task.design.md",
        &record_file("task.design", "task", state, &[], ""),
    )])
}

fn bound_draft(record_type: RecordType, id: &str, task: &str) -> Draft {
    let mut draft = Draft::new(record_type, id);
    draft.id = Some(id.to_owned());
    draft.task = Some(task.to_owned());
    draft
}

fn bind(storage: &mut MemoryStorage, record_type: RecordType, id: &str) {
    Notebook::new(storage)
        .create(&bound_draft(record_type, id, "task.design"), TODAY)
        .unwrap();
}

fn close_design(storage: &mut MemoryStorage) {
    let mut notebook = Notebook::new(storage);
    notebook.start("task.design", TODAY).unwrap();
    notebook
        .close("task.design", None, "The design is settled.", TODAY)
        .unwrap();
}

fn refusal(storage: &mut MemoryStorage, draft: &Draft) -> NotebookError {
    Notebook::new(storage).create(draft, TODAY).unwrap_err()
}

mod writing_a_binding {
    use super::*;

    #[test]
    fn a_decision_a_note_and_a_question_can_serve_a_live_task() {
        let mut storage = design_notebook("active");
        for (record_type, id) in [
            (RecordType::Decision, "decision.shape"),
            (RecordType::Note, "note.spec"),
            (RecordType::Question, "question.open"),
        ] {
            bind(&mut storage, record_type, id);
            assert_eq!(
                Notebook::new(&mut storage).record(id).unwrap().task(),
                Some("task.design")
            );
        }
    }

    #[test]
    fn a_task_and_a_rule_are_never_bound() {
        let mut storage = design_notebook("active");
        let task = bound_draft(RecordType::Task, "task.step", "task.design");
        let mut rule = bound_draft(RecordType::Decision, "decision.rule", "task.design");
        rule.kind = Some("rule".to_owned());
        for draft in [task, rule] {
            assert!(matches!(
                refusal(&mut storage, &draft),
                NotebookError::InvalidArgument { .. }
            ));
        }
    }

    #[test]
    fn the_target_must_be_a_task_the_notebook_holds() {
        let mut storage = storage_with(&[(
            "notes/note.context.md",
            &record_file("note.context", "note", "active", &[], ""),
        )]);
        assert!(matches!(
            refusal(
                &mut storage,
                &bound_draft(RecordType::Note, "note.spec", "note.context")
            ),
            NotebookError::WrongType { .. }
        ));
        assert!(matches!(
            refusal(
                &mut storage,
                &bound_draft(RecordType::Note, "note.spec", "task.ghost")
            ),
            NotebookError::DanglingRef { field: "task", .. }
        ));
    }

    #[test]
    fn a_closed_task_still_takes_a_binding_until_it_is_archived() {
        let mut closed = design_notebook("closed");
        Notebook::new(&mut closed)
            .create(
                &bound_draft(RecordType::Note, "note.spec", "task.design"),
                TODAY,
            )
            .unwrap();
        let mut archived = storage_with(&[(
            "archive/tasks/task.design.md",
            &record_file("task.design", "task", "closed", &[], ""),
        )]);
        assert_eq!(
            refusal(
                &mut archived,
                &bound_draft(RecordType::Note, "note.spec", "task.design")
            ),
            NotebookError::Archived {
                id: "task.design".to_owned(),
            }
        );
    }

    #[test]
    fn edit_binds_and_clear_unbinds() {
        let mut storage = design_notebook("active");
        storage
            .write(
                "notes/note.spec.md",
                &record_file("note.spec", "note", "active", &[], ""),
            )
            .unwrap();
        let mut notebook = Notebook::new(&mut storage);
        let bound = Edit {
            task: Some("task.design".to_owned()),
            ..Edit::default()
        };
        assert_eq!(
            notebook.edit("note.spec", &bound, TODAY).unwrap().changed,
            ["task"]
        );
        let cleared = Edit {
            clear: vec!["task".to_owned()],
            ..Edit::default()
        };
        notebook.edit("note.spec", &cleared, TODAY).unwrap();
        assert_eq!(notebook.record("note.spec").unwrap().task(), None);
    }

    #[test]
    fn edit_refuses_to_bind_a_rule() {
        let mut storage = design_notebook("active");
        storage
            .write(
                "decisions/decision.rule.md",
                &record_file("decision.rule", "decision", "active", &["kind: rule"], ""),
            )
            .unwrap();
        let bound = Edit {
            task: Some("task.design".to_owned()),
            ..Edit::default()
        };
        assert!(matches!(
            Notebook::new(&mut storage)
                .edit("decision.rule", &bound, TODAY)
                .unwrap_err(),
            NotebookError::InvalidArgument { .. }
        ));
    }
}

mod archiving_the_task {
    use super::*;

    #[test]
    fn its_bound_records_leave_with_it_in_the_state_they_stand_in() {
        let mut storage = design_notebook("active");
        bind(&mut storage, RecordType::Decision, "decision.shape");
        bind(&mut storage, RecordType::Note, "note.spec");
        close_design(&mut storage);

        let archived = Notebook::new(&mut storage).archive("task.design").unwrap();

        assert_eq!(archived.bound, ["decision.shape", "note.spec"]);
        for path in [
            "archive/decisions/decision.shape.md",
            "archive/notes/note.spec.md",
        ] {
            assert!(
                storage.read(path).unwrap().contains("state: active\n"),
                "{path} keeps its state"
            );
        }
        assert_eq!(
            Notebook::new(&mut storage).check().unwrap(),
            [],
            "an active record in the archive is valid while its Task is archived"
        );
    }

    #[test]
    fn an_open_bound_question_holds_the_whole_move_back() {
        let mut storage = design_notebook("active");
        bind(&mut storage, RecordType::Note, "note.spec");
        bind(&mut storage, RecordType::Question, "question.open");
        close_design(&mut storage);

        assert_eq!(
            Notebook::new(&mut storage)
                .archive("task.design")
                .unwrap_err(),
            NotebookError::OpenQuestions {
                id: "task.design".to_owned(),
                questions: vec!["question.open".to_owned()],
            }
        );
        assert!(storage.read("tasks/task.design.md").is_ok());
        assert!(storage.read("notes/note.spec.md").is_ok(), "nothing moved");
    }

    #[test]
    fn a_record_merely_born_from_it_stays() {
        let mut storage = design_notebook("active");
        let mut finding = Draft::new(RecordType::Note, "A finding");
        finding.id = Some("note.finding".to_owned());
        finding.from = Some("task.design".to_owned());
        Notebook::new(&mut storage).create(&finding, TODAY).unwrap();
        close_design(&mut storage);

        let archived = Notebook::new(&mut storage).archive("task.design").unwrap();

        assert!(archived.bound.is_empty());
        assert!(storage.read("notes/note.finding.md").is_ok());
    }
}

mod archiving_what_cannot_be_bound {
    use super::*;

    #[test]
    fn a_task_or_a_rule_carrying_a_stray_task_line_stays_where_it_is() {
        let mut storage = design_notebook("closed");
        for (path, file) in [
            (
                "tasks/task.stray.md",
                record_file("task.stray", "task", "open", &["task: task.design"], ""),
            ),
            (
                "decisions/decision.rule.md",
                record_file(
                    "decision.rule",
                    "decision",
                    "active",
                    &["kind: rule", "task: task.design"],
                    "",
                ),
            ),
        ] {
            storage.write(path, &file).unwrap();
        }
        let archived = Notebook::new(&mut storage).archive("task.design").unwrap();
        assert!(archived.bound.is_empty(), "{:?}", archived.bound);
        assert!(storage.read("tasks/task.stray.md").is_ok());
        assert!(storage.read("decisions/decision.rule.md").is_ok());
    }
}

mod restoring {
    use super::*;

    fn archived_design() -> MemoryStorage {
        let mut storage = design_notebook("active");
        bind(&mut storage, RecordType::Decision, "decision.shape");
        bind(&mut storage, RecordType::Note, "note.spec");
        close_design(&mut storage);
        Notebook::new(&mut storage).archive("task.design").unwrap();
        storage
    }

    #[test]
    fn the_task_brings_its_bound_records_back() {
        let mut storage = archived_design();
        let restored = Notebook::new(&mut storage)
            .restore("task.design", TODAY)
            .unwrap();
        assert_eq!(restored.bound, ["decision.shape", "note.spec"]);
        assert!(storage.read("decisions/decision.shape.md").is_ok());
        assert!(storage.read("notes/note.spec.md").is_ok());
    }

    #[test]
    fn a_record_settled_and_archived_before_its_task_stays_in_the_archive() {
        let mut storage = design_notebook("active");
        bind(&mut storage, RecordType::Note, "note.old-fact");
        let mut notebook = Notebook::new(&mut storage);
        notebook.retire("note.old-fact", TODAY).unwrap();
        notebook.archive("note.old-fact").unwrap();
        close_design(&mut storage);
        Notebook::new(&mut storage).archive("task.design").unwrap();

        let restored = Notebook::new(&mut storage)
            .restore("task.design", TODAY)
            .unwrap();

        assert!(restored.bound.is_empty(), "{:?}", restored.bound);
        assert!(storage.read("archive/notes/note.old-fact.md").is_ok());
    }

    #[test]
    fn a_record_restored_alone_outlives_its_archived_task() {
        let mut storage = archived_design();
        let restored = Notebook::new(&mut storage)
            .restore("decision.shape", TODAY)
            .unwrap();
        assert_eq!(restored.unbound.as_deref(), Some("task.design"));
        let file = storage.read("decisions/decision.shape.md").unwrap();
        assert!(!file.contains("task:"), "{file}");
        assert!(file.contains(&format!("updated: {TODAY}\n")), "{file}");
        assert!(
            storage.read("archive/notes/note.spec.md").is_ok(),
            "the other bound record stays with its Task"
        );
    }

    #[test]
    fn a_record_whose_task_is_in_play_keeps_its_binding() {
        let mut storage = design_notebook("active");
        storage
            .write(
                "archive/decisions/decision.old.md",
                &record_file(
                    "decision.old",
                    "decision",
                    "retired",
                    &["task: task.design"],
                    "",
                ),
            )
            .unwrap();
        let restored = Notebook::new(&mut storage)
            .restore("decision.old", TODAY)
            .unwrap();
        assert_eq!(restored.unbound, None);
        assert_eq!(
            Notebook::new(&mut storage)
                .record("decision.old")
                .unwrap()
                .task(),
            Some("task.design")
        );
    }
}

mod closing_the_task {
    use super::*;

    #[test]
    fn names_what_was_born_from_it_and_will_outlive_it() {
        let mut storage = design_notebook("active");
        bind(&mut storage, RecordType::Note, "note.spec");
        let mut notebook = Notebook::new(&mut storage);
        for (record_type, id, kind) in [
            (RecordType::Note, "note.finding", None),
            (RecordType::Decision, "decision.rule", Some("rule")),
            (RecordType::Task, "task.step", None),
        ] {
            let mut draft = Draft::new(record_type, id);
            draft.id = Some(id.to_owned());
            draft.from = Some("task.design".to_owned());
            draft.kind = kind.map(str::to_owned);
            notebook.create(&draft, TODAY).unwrap();
        }
        notebook.start("task.design", TODAY).unwrap();

        let closed = notebook
            .close("task.design", None, "The design is settled.", TODAY)
            .unwrap();

        assert_eq!(
            closed.unbound,
            ["note.finding"],
            "a bound record, a rule and a Task are not candidates"
        );
    }
}

mod checking {
    use super::*;

    fn codes(storage: &mut MemoryStorage) -> Vec<(String, FindingCode, Option<Repair>)> {
        Notebook::new(storage)
            .check()
            .unwrap()
            .into_iter()
            .map(|found| (found.path, found.finding.code, found.repair))
            .collect()
    }

    #[test]
    fn a_record_left_behind_by_its_filed_task_is_a_broken_binding() {
        let mut storage = storage_with(&[
            (
                "archive/tasks/task.design.md",
                &record_file("task.design", "task", "closed", &[], ""),
            ),
            (
                "notes/note.spec.md",
                &record_file("note.spec", "note", "active", &["task: task.design"], ""),
            ),
        ]);
        assert_eq!(
            codes(&mut storage),
            [(
                "notes/note.spec.md".to_owned(),
                FindingCode::BrokenBinding,
                Some(Repair::Clear("task"))
            )]
        );
    }

    #[test]
    fn a_live_record_binding_from_the_archive_while_its_task_is_back_is_restored() {
        let mut storage = storage_with(&[
            (
                "tasks/task.design.md",
                &record_file("task.design", "task", "open", &[], ""),
            ),
            (
                "archive/notes/note.spec.md",
                &record_file("note.spec", "note", "active", &["task: task.design"], ""),
            ),
        ]);
        assert_eq!(
            codes(&mut storage),
            [(
                "archive/notes/note.spec.md".to_owned(),
                FindingCode::ArchivedLiveRecord,
                Some(Repair::Restore)
            )]
        );
    }

    #[test]
    fn a_binding_to_another_type_or_on_a_rule_is_a_bad_value() {
        let mut storage = storage_with(&[
            (
                "notes/note.context.md",
                &record_file("note.context", "note", "active", &[], ""),
            ),
            (
                "tasks/task.design.md",
                &record_file("task.design", "task", "open", &[], ""),
            ),
            (
                "notes/note.spec.md",
                &record_file("note.spec", "note", "active", &["task: note.context"], ""),
            ),
            (
                "decisions/decision.rule.md",
                &record_file(
                    "decision.rule",
                    "decision",
                    "active",
                    &["kind: rule", "task: task.design"],
                    "",
                ),
            ),
        ]);
        assert_eq!(
            codes(&mut storage),
            [
                (
                    "decisions/decision.rule.md".to_owned(),
                    FindingCode::BadValue,
                    Some(Repair::Clear("task"))
                ),
                (
                    "notes/note.spec.md".to_owned(),
                    FindingCode::BadValue,
                    Some(Repair::Clear("task"))
                ),
            ]
        );
    }
}

#[test]
fn a_task_with_bound_records_is_not_deleted_from_under_them() {
    let mut storage = design_notebook("open");
    bind(&mut storage, RecordType::Note, "note.spec");
    assert_eq!(
        Notebook::new(&mut storage)
            .delete("task.design")
            .unwrap_err(),
        NotebookError::StillReferenced {
            id: "task.design".to_owned(),
            blockers: vec![Blocker {
                carrier: "note.spec".to_owned(),
                through: "task",
            }],
        }
    );
}

#[test]
fn the_graph_draws_a_binding_out_of_its_task() {
    let mut storage = design_notebook("open");
    bind(&mut storage, RecordType::Note, "note.spec");
    let graph = Notebook::new(&mut storage)
        .graph(&GraphSlice::default())
        .unwrap();
    assert_eq!(
        graph
            .edges()
            .iter()
            .map(|edge| (edge.from, edge.to, edge.kind.word()))
            .collect::<Vec<_>>(),
        [("task.design", "note.spec", "bound")]
    );
}
