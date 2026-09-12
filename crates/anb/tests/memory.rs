//! Memory follows its audience; work follows its owner.

use anb::cli::Cli;
use anb::reply::{Host, Reply, execute};
use anb_core::{Draft, MemoryStorage, Notebook, RecordType, Storage};
use clap::Parser;
use std::path::Path;

const TODAY: &str = "2026-09-12";

#[test]
fn every_work_read_uses_the_same_configured_audience() {
    for scope in ["team", "mine"] {
        let mut storage = shared_knowledge();
        storage
            .write("config", &format!("scope: {scope}\n"))
            .unwrap();
        for (id, owner) in [
            ("task.ada", Some("Ada")),
            ("task.grace", Some("Grace")),
            ("task.unassigned", None),
        ] {
            let mut draft = Draft::new(RecordType::Task, id);
            draft.id = Some(id.to_owned());
            draft.taken_by = owner.map(str::to_owned);
            Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
        }
        for flags in [vec![], vec!["--team"], vec!["--mine"], vec!["--by", "Ada"]] {
            let expected = if flags == ["--by", "Ada"] {
                vec!["task.ada"]
            } else if flags == ["--mine"] || (flags.is_empty() && scope == "mine") {
                vec!["task.grace"]
            } else {
                vec!["task.ada", "task.grace", "task.unassigned"]
            };
            for command in ["recall", "hook", "status", "list", "ready"] {
                if command == "hook" && !flags.is_empty() {
                    continue;
                }
                let mut arguments = vec![command];
                arguments.extend(flags.iter().copied());
                if command == "list" {
                    arguments.extend(["--type", "task"]);
                }
                let ids = match run(&mut storage, &arguments) {
                    Reply::Recalled(recalled) => {
                        assert_eq!(
                            recalled.memories.len(),
                            2,
                            "shared knowledge is independent of work audience"
                        );
                        recalled
                            .work
                            .ready
                            .into_iter()
                            .map(|row| row.id)
                            .collect::<Vec<_>>()
                    }
                    Reply::Status { status } => {
                        status.ready.into_iter().map(|row| row.id).collect()
                    }
                    Reply::Ready { rows, .. } => rows.into_iter().map(|row| row.id).collect(),
                    Reply::Listing { rows, .. } => rows.into_iter().map(|row| row.id).collect(),
                    other => panic!("unexpected reply: {other:?}"),
                };
                assert_eq!(ids, expected, "{scope}: {arguments:?}");
            }
        }
    }
}

fn run(storage: &mut MemoryStorage, args: &[&str]) -> Reply {
    run_with_practices(storage, args, None, None)
}

fn run_with_practices(
    storage: &mut MemoryStorage,
    args: &[&str],
    personal: Option<&dyn Storage>,
    global: Option<&dyn Storage>,
) -> Reply {
    let cli = Cli::try_parse_from(std::iter::once("anb").chain(args.iter().copied())).unwrap();
    execute(
        cli.command,
        storage,
        Host {
            session: cli.session.as_deref(),
            identity: || Some("Grace".to_owned()),
            read_file: &|_| panic!("this scenario reads no external file"),
            lost_proofs: &|_| Vec::new(),
            user_notebook: global,
            personal_notebook: personal,
            audience: anb::recall::Audience::Project,
            project_dir: Path::new("."),
            today: TODAY,
        },
    )
    .unwrap()
}

fn practice(body: &str) -> MemoryStorage {
    let mut storage = MemoryStorage::new();
    let mut draft = Draft::new(RecordType::Note, "Review practice");
    draft.id = Some("note.review".to_owned());
    body.clone_into(&mut draft.body);
    Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
    storage
}

#[test]
fn recall_composes_shared_knowledge_and_both_personal_scopes() {
    let mut shared = shared_knowledge();
    let personal = practice("Ask before running slow project integration tests.");
    let global = practice("Review behavior before formatting.");
    let Reply::Recalled(recalled) =
        run_with_practices(&mut shared, &["recall"], Some(&personal), Some(&global))
    else {
        panic!("expected composed recall");
    };
    assert_eq!(recalled.work.by.as_deref(), Some("Grace"));
    let memories = recalled
        .memories
        .iter()
        .map(|item| {
            (
                item.audience.word(),
                item.memory.id.as_str(),
                item.memory.body.trim(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        memories,
        [
            ("project", "decision.tenant-scope", ""),
            ("project", "note.workspace", ""),
            (
                "personal",
                "note.review",
                "Ask before running slow project integration tests."
            ),
            (
                "global",
                "note.review",
                "Review behavior before formatting."
            ),
        ]
    );
}

#[test]
fn a_colleague_without_the_private_stores_recalls_only_shared_knowledge() {
    let mut shared = shared_knowledge();
    let Reply::Recalled(recalled) = run(&mut shared, &["recall"]) else {
        panic!("expected composed recall");
    };
    assert_eq!(recalled.memories.len(), 2);
    assert!(
        recalled
            .memories
            .iter()
            .all(|item| item.audience == anb::recall::Audience::Project)
    );
}

#[test]
fn recall_searches_personal_practices_without_needing_to_know_their_location() {
    let mut shared = shared_knowledge();
    let personal = practice("Review the EXPОRT scenario with the customer.");
    let global = practice("Review behavior before formatting.");
    let Reply::Recalled(recalled) = run_with_practices(
        &mut shared,
        &["recall", "customer"],
        Some(&personal),
        Some(&global),
    ) else {
        panic!("expected composed recall");
    };
    assert!(
        recalled
            .memories
            .iter()
            .any(|item| item.audience == anb::recall::Audience::Personal)
    );
    assert!(
        recalled
            .memories
            .iter()
            .all(|item| item.audience != anb::recall::Audience::Global)
    );
    assert!(recalled.more.contains("customer"));
}

#[test]
fn a_large_project_does_not_starve_personal_practices_from_bounded_recall() {
    let mut shared = MemoryStorage::new();
    for index in 0..40 {
        let mut draft = Draft::new(RecordType::Note, &format!("Project component {index}"));
        draft.body = "Component context remains readable. ".repeat(40);
        Notebook::new(&mut shared).create(&draft, TODAY).unwrap();
    }
    let personal = practice("Ask before running slow project integration tests.");
    let global = practice("Review behavior before formatting.");
    let reply = run_with_practices(&mut shared, &["recall"], Some(&personal), Some(&global));
    let document = anb::json::value(&reply);
    let memories = document["memories"].as_array().unwrap();
    for (scope, body) in [
        (
            "personal",
            "Ask before running slow project integration tests.",
        ),
        ("global", "Review behavior before formatting."),
    ] {
        let memory = memories
            .iter()
            .find(|memory| memory["scope"] == scope)
            .expect("private practice was starved by the project");
        assert!(memory["body"]["head"].as_str().unwrap().contains(body));
        assert_eq!(memory["body"]["omitted"], 0);
    }
    let project = document["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|source| source["scope"] == "project")
        .unwrap();
    assert_eq!(project["count"], 40);
    assert_eq!(
        project["omitted"].as_u64().unwrap(),
        40 - memories
            .iter()
            .filter(|memory| memory["scope"] == "project")
            .count() as u64
    );
    assert!(document["budget"]["spent"].as_u64().unwrap() <= 1500);
}

#[test]
fn recall_keeps_retired_knowledge_out_of_the_working_set_without_erasing_it() {
    let mut shared = practice("An old review rule.");
    let mut notebook = Notebook::new(&mut shared);
    notebook.retire("note.review", TODAY).unwrap();
    notebook.archive("note.review").unwrap();
    let Reply::Recalled(recalled) = run(&mut shared, &["recall"]) else {
        panic!("expected composed recall");
    };
    assert!(recalled.memories.is_empty());
    let archived = Notebook::new(&mut shared).record("note.review").unwrap();
    assert!(archived.file().body().contains("An old review rule."));
}

#[test]
fn malformed_knowledge_is_reported_instead_of_recalled_as_a_fact() {
    let mut shared = MemoryStorage::from_files([(
        "notes/note.broken.md",
        "---\nid: note.broken\ntype: note\nstate: impossible\ntitle: Broken\ncreated: 2026-09-12\nupdated: 2026-09-12\n---\nNot established knowledge.\n",
    )]);
    let Reply::Recalled(recalled) = run(&mut shared, &["recall"]) else {
        panic!("expected composed recall");
    };
    assert!(recalled.memories.is_empty());
    assert_eq!(
        recalled.invalid,
        [anb::recall::ScopedInvalid {
            audience: anb::recall::Audience::Project,
            path: "notes/note.broken.md".to_owned(),
        }]
    );
}

#[test]
fn an_unreadable_knowledge_envelope_is_not_silently_an_empty_memory() {
    let mut shared = MemoryStorage::from_files([(
        "notes/note.broken.md",
        "This file has no envelope, so its claims cannot be classified.\n",
    )]);
    let Reply::Recalled(recalled) = run(&mut shared, &["recall"]) else {
        panic!("expected composed recall");
    };
    assert!(recalled.memories.is_empty());
    assert_eq!(
        recalled.invalid,
        [anb::recall::ScopedInvalid {
            audience: anb::recall::Audience::Project,
            path: "notes/note.broken.md".to_owned(),
        }]
    );
}

#[test]
fn recall_rejects_an_empty_search_like_other_queries() {
    let mut shared = shared_knowledge();
    let error = Notebook::new(&mut shared)
        .recall(Some("   "), None)
        .unwrap_err();
    assert!(matches!(
        error,
        anb_core::NotebookError::InvalidArgument { .. }
    ));
}

#[test]
fn a_focus_prioritizes_related_knowledge_without_hiding_other_standing_rules() {
    let mut shared = shared_knowledge();
    let mut notebook = Notebook::new(&mut shared);
    let mut task = Draft::new(RecordType::Task, "Verify customer exports");
    task.id = Some("task.exports".to_owned());
    notebook.create(&task, TODAY).unwrap();
    let mut finding = Draft::new(RecordType::Note, "Export boundary evidence");
    finding.id = Some("note.export-evidence".to_owned());
    finding.from = Some("task.exports".to_owned());
    notebook.create(&finding, TODAY).unwrap();
    let Reply::Recalled(recalled) = run(&mut shared, &["recall", "--for", "task.exports"]) else {
        panic!("expected focused recall");
    };
    assert_eq!(recalled.focus.unwrap().id, "task.exports");
    assert_eq!(recalled.memories[0].memory.id, "note.export-evidence");
    assert!(recalled.memories[0].memory.related);
    assert!(
        recalled
            .memories
            .iter()
            .any(|item| item.memory.id == "decision.tenant-scope")
    );
}

#[test]
fn recall_does_not_rewrite_legacy_records_or_personal_practices() {
    let mut shared = shared_knowledge();
    let personal = practice("Review behavior first.");
    let before = shared.read("notes/note.workspace.md").unwrap();
    let before_personal = personal.read("notes/note.review.md").unwrap();
    run_with_practices(&mut shared, &["recall"], Some(&personal), None);
    assert_eq!(shared.read("notes/note.workspace.md").unwrap(), before);
    assert_eq!(
        personal.read("notes/note.review.md").unwrap(),
        before_personal
    );
}

fn shared_knowledge() -> MemoryStorage {
    let mut storage = MemoryStorage::new();
    storage.write("config", "scope: mine\n").unwrap();
    let mut notebook = Notebook::new(&mut storage).with_identity(Some("Ada"));
    let mut rule = Draft::new(RecordType::Decision, "Keep customer exports tenant-scoped");
    rule.kind = Some("rule".to_owned());
    rule.id = Some("decision.tenant-scope".to_owned());
    notebook.create(&rule, TODAY).unwrap();
    let mut model = Draft::new(RecordType::Note, "A workspace owns its customer exports");
    model.kind = Some("model".to_owned());
    model.id = Some("note.workspace".to_owned());
    notebook.create(&model, TODAY).unwrap();
    storage
}

#[test]
fn personal_work_scope_does_not_hide_a_colleagues_project_rules() {
    let mut storage = shared_knowledge();
    let Reply::Listing { rows, filter, .. } = run(
        &mut storage,
        &["list", "--type", "decision", "--kind", "rule"],
    ) else {
        panic!("expected a record listing");
    };
    assert_eq!(
        rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
        ["decision.tenant-scope"]
    );
    assert_eq!(filter.by, None);
}

#[test]
fn personal_work_scope_does_not_hide_a_colleagues_domain_model() {
    let mut storage = shared_knowledge();
    let Reply::Listing { rows, .. } = run(&mut storage, &["list", "--type", "note"]) else {
        panic!("expected a record listing");
    };
    assert_eq!(
        rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
        ["note.workspace"]
    );
}

#[test]
fn explicit_personal_knowledge_filter_still_selects_its_author() {
    let mut storage = shared_knowledge();
    let Reply::Listing { rows, filter, .. } =
        run(&mut storage, &["list", "--type", "note", "--mine"])
    else {
        panic!("expected a record listing");
    };
    assert!(rows.is_empty());
    assert_eq!(filter.by.as_deref(), Some("Grace"));
}
