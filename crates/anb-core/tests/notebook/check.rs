use crate::*;

fn findings_for(storage: &mut MemoryStorage) -> Vec<(String, FindingCode)> {
    Notebook::new(storage)
        .check()
        .unwrap()
        .into_iter()
        .map(|located| (located.path, located.finding.code))
        .collect()
}

/// A finding quotes the value it condemns, and a hand can write a field
/// as long as it likes; the row that names it still answers at a fixed
/// size, with both ends of the value kept so the reason survives the cut.
#[test]
fn a_finding_quoting_a_long_value_keeps_both_ends_of_it() {
    let long = "x".repeat(2000);
    let mut storage = storage_with(&[(
        "tasks/task.demo.md",
        &format!(
            "---\nid: task.demo\ntype: task\nstate: {long}\ntitle: A demo record\ncreated: 2026-08-24\nupdated: 2026-08-25\n---\n"
        ),
    )]);

    let located = Notebook::new(&mut storage).check().unwrap();
    let message = &located.first().unwrap().finding.message;
    assert!(
        message.chars().count() <= anb_core::encode::TEXT_BOUND,
        "unbounded message: {message}"
    );
    assert!(message.starts_with("state: `x"), "lost the key: {message}");
    assert!(
        message.ends_with("` is not one of open, active, review, closed for a task"),
        "lost the reason: {message}"
    );
}

#[test]
fn a_clean_notebook_checks_empty() {
    let mut storage = storage_with(&[
        ("tasks/task.demo.md", &task_file("open", &[])),
        (
            "notes/note.demo.md",
            &record_file("note.demo", "note", "active", &[], ""),
        ),
    ]);
    assert_eq!(findings_for(&mut storage), vec![]);
}

#[test]
fn errors_lead_the_report_however_late_their_file_sorts() {
    // The reply prints a bounded head, and the settling verbs leave
    // warnings behind by the dozen: on file order alone an error in a
    // late-sorting file would fall off the end of a real notebook.
    let mut storage = storage_with(&[
        (
            "tasks/task.aaa.md",
            &record_file("task.aaa", "task", "closed", &[], ""),
        ),
        (
            "tasks/task.zzz.md",
            &record_file("task.zzz", "task", "open", &["blocked-by: task.nobody"], ""),
        ),
    ]);
    assert_eq!(
        findings_for(&mut storage),
        vec![
            ("tasks/task.zzz.md".to_owned(), FindingCode::DanglingRef),
            (
                "tasks/task.aaa.md".to_owned(),
                FindingCode::UnarchivedSettledRecord
            ),
        ]
    );
}

#[test]
fn a_task_stranded_in_the_archive_is_named_against_its_own_file() {
    let mut storage = storage_with(&[
        ("tasks/task.demo.md", &task_file("open", &[])),
        (
            "archive/tasks/task.stranded.md",
            &record_file("task.stranded", "task", "open", &[], ""),
        ),
    ]);
    assert_eq!(
        findings_for(&mut storage),
        vec![(
            "archive/tasks/task.stranded.md".to_owned(),
            FindingCode::ArchivedLiveRecord
        )],
        "the finding lands on the stranded file, not on the live record that is fine"
    );
}

#[test]
fn two_files_claiming_one_id_are_both_named() {
    // The shape an archive move leaves when it dies between its write
    // and its remove: identical settled bytes in both homes.
    let text = task_file("closed", &[]);
    let mut storage = storage_with(&[
        ("tasks/task.demo.md", &text),
        ("archive/tasks/task.demo.md", &text),
    ]);
    assert_eq!(
        findings_for(&mut storage),
        vec![
            (
                "archive/tasks/task.demo.md".to_owned(),
                FindingCode::DuplicateId
            ),
            ("tasks/task.demo.md".to_owned(), FindingCode::DuplicateId),
            (
                "tasks/task.demo.md".to_owned(),
                FindingCode::UnarchivedSettledRecord
            ),
        ]
    );
}

#[test]
fn a_reference_into_nothing_is_a_dangling_ref_at_its_line() {
    let mut storage = storage_with(&[(
        "tasks/task.demo.md",
        &task_file("open", &["from: question.never-written"]),
    )]);
    let located = Notebook::new(&mut storage).check().unwrap();
    assert_eq!(located.len(), 1);
    assert_eq!(located[0].finding.code, FindingCode::DanglingRef);
    assert_eq!(located[0].finding.line, Some(6));
}

#[test]
fn a_supersession_without_its_back_pointer_names_both_files() {
    let mut storage = storage_with(&[
        (
            "decisions/decision.new.md",
            &record_file(
                "decision.new",
                "decision",
                "active",
                &["supersedes: decision.old"],
                "",
            ),
        ),
        (
            "archive/decisions/decision.old.md",
            &record_file("decision.old", "decision", "retired", &[], ""),
        ),
    ]);
    assert_eq!(
        findings_for(&mut storage),
        vec![
            (
                "archive/decisions/decision.old.md".to_owned(),
                FindingCode::BrokenSupersession
            ),
            (
                "decisions/decision.new.md".to_owned(),
                FindingCode::BrokenSupersession
            ),
        ]
    );
}

#[test]
fn a_record_marked_superseded_that_still_reads_live_is_broken() {
    let mut storage = storage_with(&[
        (
            "decisions/decision.old.md",
            &record_file(
                "decision.old",
                "decision",
                "active",
                &["superseded-by: decision.new"],
                "",
            ),
        ),
        (
            "decisions/decision.new.md",
            &record_file(
                "decision.new",
                "decision",
                "active",
                &["supersedes: decision.old"],
                "",
            ),
        ),
    ]);
    assert_eq!(
        findings_for(&mut storage),
        vec![(
            "decisions/decision.old.md".to_owned(),
            FindingCode::BrokenSupersession
        )],
        "the pair is coherent; the victim's live state alone is the defect"
    );
}

#[test]
fn a_hand_edited_cycle_is_named_on_every_member_at_its_edge_line() {
    let mut storage = storage_with(&[
        (
            "tasks/task.a.md",
            &record_file("task.a", "task", "open", &["blocked-by: task.b"], ""),
        ),
        (
            "tasks/task.b.md",
            &record_file("task.b", "task", "open", &["blocked-by: task.a"], ""),
        ),
    ]);
    let located = Notebook::new(&mut storage).check().unwrap();
    assert_eq!(
        located
            .iter()
            .map(|found| (found.path.as_str(), found.finding.code, found.finding.line))
            .collect::<Vec<_>>(),
        vec![
            ("tasks/task.a.md", FindingCode::DepCycle, Some(6)),
            ("tasks/task.b.md", FindingCode::DepCycle, Some(6)),
        ]
    );
    assert!(
        located[0]
            .finding
            .message
            .contains("task.a → task.b → task.a"),
        "the message walks the whole cycle: {}",
        located[0].finding.message
    );
}

#[test]
fn two_disjoint_cycles_are_both_named() {
    let mut storage = storage_with(&[
        (
            "tasks/task.a.md",
            &record_file("task.a", "task", "open", &["blocked-by: task.b"], ""),
        ),
        (
            "tasks/task.b.md",
            &record_file("task.b", "task", "open", &["blocked-by: task.a"], ""),
        ),
        (
            "tasks/task.c.md",
            &record_file("task.c", "task", "open", &["blocked-by: task.d"], ""),
        ),
        (
            "tasks/task.d.md",
            &record_file("task.d", "task", "open", &["blocked-by: task.c"], ""),
        ),
    ]);
    let named: Vec<String> = Notebook::new(&mut storage)
        .check()
        .unwrap()
        .into_iter()
        .map(|located| located.path)
        .collect();
    assert_eq!(
        named,
        vec![
            "tasks/task.a.md",
            "tasks/task.b.md",
            "tasks/task.c.md",
            "tasks/task.d.md"
        ]
    );
}

#[test]
fn repairing_a_named_cycle_surfaces_the_one_overlapping_it() {
    let entangled = [
        (
            "tasks/task.a.md",
            record_file(
                "task.a",
                "task",
                "open",
                &["blocked-by: task.b", "blocked-by: task.c"],
                "",
            ),
        ),
        (
            "tasks/task.b.md",
            record_file("task.b", "task", "open", &["blocked-by: task.c"], ""),
        ),
        (
            "tasks/task.c.md",
            record_file("task.c", "task", "open", &["blocked-by: task.a"], ""),
        ),
    ];
    let mut storage = storage_with(
        &entangled
            .iter()
            .map(|(path, text)| (*path, text.as_str()))
            .collect::<Vec<_>>(),
    );
    let first_pass: Vec<FindingCode> = Notebook::new(&mut storage)
        .check()
        .unwrap()
        .into_iter()
        .map(|located| located.finding.code)
        .collect();
    assert_eq!(
        first_pass,
        vec![
            FindingCode::DepCycle,
            FindingCode::DepCycle,
            FindingCode::DepCycle
        ],
        "one cycle per back edge: the walk names task.a → task.b → task.c → task.a"
    );

    Notebook::new(&mut storage)
        .unblock("task.a", "task.b", TODAY)
        .unwrap();
    assert_eq!(
        findings_for(&mut storage),
        vec![
            ("tasks/task.a.md".to_owned(), FindingCode::DepCycle),
            ("tasks/task.c.md".to_owned(), FindingCode::DepCycle),
        ],
        "the cycle hidden behind the repaired one surfaces on the next walk"
    );
}

#[test]
fn a_task_waiting_on_itself_is_a_dep_cycle_at_its_own_line() {
    let mut storage = storage_with(&[(
        "tasks/task.demo.md",
        &task_file("open", &["blocked-by: task.demo"]),
    )]);
    let located = Notebook::new(&mut storage).check().unwrap();
    assert_eq!(located.len(), 1);
    assert_eq!(located[0].finding.code, FindingCode::DepCycle);
    assert_eq!(located[0].finding.line, Some(6));
}

#[test]
fn a_cycle_finding_points_at_the_live_file_when_the_archive_holds_the_name_too() {
    let mut storage = storage_with(&[
        (
            "tasks/task.a.md",
            &record_file("task.a", "task", "open", &["blocked-by: task.b"], ""),
        ),
        (
            "tasks/task.b.md",
            &record_file("task.b", "task", "open", &["blocked-by: task.a"], ""),
        ),
        (
            "archive/tasks/task.a.md",
            &record_file("task.a", "task", "closed", &["closed: 2026-08-25"], ""),
        ),
    ]);
    let located = Notebook::new(&mut storage).check().unwrap();
    assert_eq!(
        located
            .iter()
            .filter(|found| found.finding.code == FindingCode::DepCycle)
            .map(|found| (found.path.as_str(), found.finding.line))
            .collect::<Vec<_>>(),
        vec![("tasks/task.a.md", Some(6)), ("tasks/task.b.md", Some(6))],
        "the edge a reader must erase sits in the live file, not its archived twin"
    );
}

#[test]
fn a_hand_edited_lineage_loop_is_named_on_every_member_at_its_from_line() {
    let mut storage = storage_with(&[
        (
            "tasks/task.a.md",
            &record_file("task.a", "task", "open", &["from: task.b"], ""),
        ),
        (
            "tasks/task.b.md",
            &record_file("task.b", "task", "open", &["from: task.a"], ""),
        ),
    ]);
    let located = Notebook::new(&mut storage).check().unwrap();
    assert_eq!(
        located
            .iter()
            .map(|found| (found.path.as_str(), found.finding.code, found.finding.line))
            .collect::<Vec<_>>(),
        vec![
            ("tasks/task.a.md", FindingCode::OriginCycle, Some(6)),
            ("tasks/task.b.md", FindingCode::OriginCycle, Some(6)),
        ]
    );
    assert!(
        located[0]
            .finding
            .message
            .contains("task.a → task.b → task.a"),
        "the message walks the whole lineage: {}",
        located[0].finding.message
    );
}

#[test]
fn a_record_born_from_itself_is_an_origin_cycle() {
    let mut storage = storage_with(&[(
        "tasks/task.demo.md",
        &task_file("open", &["from: task.demo"]),
    )]);
    let located = Notebook::new(&mut storage).check().unwrap();
    assert_eq!(
        located
            .iter()
            .map(|found| (found.path.as_str(), found.finding.code))
            .collect::<Vec<_>>(),
        vec![("tasks/task.demo.md", FindingCode::OriginCycle)]
    );
}

#[test]
fn a_lineage_loop_leaves_its_records_open_to_repair() {
    let mut storage = storage_with(&[
        (
            "tasks/task.a.md",
            &record_file("task.a", "task", "open", &["from: task.b"], ""),
        ),
        (
            "tasks/task.b.md",
            &record_file("task.b", "task", "open", &["from: task.a"], ""),
        ),
    ]);
    assert_eq!(
        Notebook::new(&mut storage).check().unwrap().len(),
        2,
        "the loop stands before the repair"
    );
    let edit = Edit {
        clear: vec!["from".to_owned()],
        ..Edit::default()
    };
    let edited = Notebook::new(&mut storage)
        .edit("task.a", &edit, TODAY)
        .expect("a cycle only two files together carry must not freeze either of them");
    assert_eq!(edited.changed, vec!["from"]);
    assert!(
        Notebook::new(&mut storage).check().unwrap().is_empty(),
        "the loop is gone once the line that closes it is erased"
    );
}

#[test]
fn a_dependency_edge_into_a_non_task_is_a_bad_value() {
    let mut storage = storage_with(&[
        (
            "tasks/task.demo.md",
            &task_file("open", &["blocked-by: note.a-fact"]),
        ),
        (
            "notes/note.a-fact.md",
            &record_file("note.a-fact", "note", "active", &[], ""),
        ),
    ]);
    assert_eq!(
        findings_for(&mut storage),
        vec![("tasks/task.demo.md".to_owned(), FindingCode::BadValue)]
    );
}

#[test]
fn a_routed_question_pointing_at_nothing_has_lost_its_thread() {
    let mut storage = storage_with(&[(
        "archive/questions/question.demo.md",
        &record_file(
            "question.demo",
            "question",
            "routed",
            &["routed-to: decision.never-written"],
            "",
        ),
    )]);
    assert_eq!(
        findings_for(&mut storage),
        vec![(
            "archive/questions/question.demo.md".to_owned(),
            FindingCode::BrokenRouting
        )]
    );
}

mod unreadable_files {
    use crate::*;

    /// [`MemoryStorage`] holds strings, so the adapter's duty is simulated:
    /// the marked paths answer reads with [`StorageError::NotUtf8`].
    struct BinaryHolding {
        inner: MemoryStorage,
        binary: Vec<String>,
    }

    impl BinaryHolding {
        fn with_binary_at(path: &str, files: &[(&str, &str)]) -> Self {
            let mut all: Vec<(&str, &str)> = files.to_vec();
            all.push((path, ""));
            BinaryHolding {
                inner: MemoryStorage::from_files(all),
                binary: vec![path.to_owned()],
            }
        }
    }

    impl Storage for BinaryHolding {
        fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
            self.inner.list(dir)
        }

        fn read(&self, path: &str) -> Result<String, StorageError> {
            if self.binary.iter().any(|held| held == path) {
                return Err(StorageError::NotUtf8 {
                    path: path.to_owned(),
                });
            }
            self.inner.read(path)
        }

        fn write(&mut self, path: &str, content: &str) -> Result<(), StorageError> {
            self.inner.write(path, content)
        }

        fn remove(&mut self, path: &str) -> Result<(), StorageError> {
            self.inner.remove(path)
        }
    }

    #[test]
    fn check_names_the_file_with_the_not_utf8_finding() {
        let storage = &mut BinaryHolding::with_binary_at("tasks/task.binary.md", &[]);
        let findings = Notebook::new(storage).check().unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].path, "tasks/task.binary.md");
        assert_eq!(findings[0].finding.code, FindingCode::NotUtf8);
    }

    #[test]
    fn an_unreadable_archived_copy_refuses_the_move_as_an_invalid_record() {
        let storage = &mut BinaryHolding::with_binary_at("archive/tasks/task.demo.md", &[]);
        let refusal = Notebook::new(storage)
            .archive("task.demo", TODAY)
            .unwrap_err();
        match refusal {
            NotebookError::InvalidRecord { path, findings } => {
                assert_eq!(path, "archive/tasks/task.demo.md");
                assert_eq!(
                    findings.iter().map(|f| f.code).collect::<Vec<_>>(),
                    vec![FindingCode::NotUtf8]
                );
            }
            other => panic!("bytes that cannot be read name a record, not the medium: {other:?}"),
        }
    }

    #[test]
    fn the_listing_shows_the_file_as_invalid_instead_of_aborting() {
        let text = record_file("task.a", "task", "open", &[], "");
        let storage = &mut BinaryHolding::with_binary_at(
            "tasks/task.binary.md",
            &[("tasks/task.a.md", &text)],
        );
        let rows = Notebook::new(storage).list().unwrap();
        let states: Vec<(&str, &str)> = rows
            .iter()
            .map(|row| (row.id.as_str(), row.state.as_str()))
            .collect();
        assert_eq!(states, vec![("task.a", "open"), ("task.binary", "invalid")]);
    }

    #[test]
    fn a_mutation_is_refused_with_the_not_utf8_finding() {
        let storage = &mut BinaryHolding::with_binary_at("tasks/task.binary.md", &[]);
        let error = Notebook::new(storage)
            .start("task.binary", TODAY)
            .unwrap_err();
        let NotebookError::InvalidRecord { findings, .. } = error else {
            panic!("the refusal must carry the finding, got {error:?}");
        };
        assert_eq!(findings[0].code, FindingCode::NotUtf8);
    }

    /// The format stamp is the notebook's claim about which anb wrote it.
    /// A version this build does not read is named rather than assumed,
    /// because reading unknown bytes as if they were this format is how a
    /// reader invents history; the version it does read passes silently.
    #[test]
    fn a_config_naming_another_format_is_a_named_check_finding() {
        for (config, expected) in [("format: 1\n", None), ("format: 2\n", Some(1))] {
            let storage = &mut super::storage_with(&[("config", config)]);
            let findings = Notebook::new(storage).check().unwrap();
            let named: Vec<usize> = findings
                .iter()
                .filter(|found| found.finding.code == FindingCode::BadValue)
                .map(|found| found.finding.line.unwrap())
                .collect();
            assert_eq!(
                named,
                expected.into_iter().collect::<Vec<usize>>(),
                "on {config:?}"
            );
        }
    }

    #[test]
    fn a_binary_config_is_a_named_check_finding() {
        let storage = &mut BinaryHolding::with_binary_at("config", &[]);
        let findings = Notebook::new(storage).check().unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].path, "config");
        assert_eq!(findings[0].finding.code, FindingCode::NotUtf8);
    }

    #[test]
    fn an_id_held_by_an_unreadable_file_is_still_taken() {
        let storage = &mut BinaryHolding::with_binary_at("tasks/task.binary.md", &[]);
        let mut draft = Draft::new(RecordType::Task, "A demo record");
        draft.id = Some("task.binary".to_owned());
        assert!(matches!(
            Notebook::new(storage).create(&draft, TODAY).unwrap_err(),
            NotebookError::DuplicateId { .. }
        ));
    }
}
