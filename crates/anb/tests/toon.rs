use anb::recall::{Audience, Recall, ScopedMemory};
use anb::recovery::Recovery;
use anb::reply::{Reply, SkillReply};
use anb::{json, text};
use anb_core::{ActiveTask, Attribution, Budget, Counts, Memory, ReadyTask, RecordType, Status};
use serde_json::Value;

const TODAY: &str = "2026-09-12";

fn decoded(reply: &Reply) -> Value {
    let toon = text::render(reply);
    let decoded = reddb_io_toon::decode(&toon).unwrap().to_json_value();
    assert_eq!(
        decoded,
        serde_json::from_str::<Value>(&json::render(reply)).unwrap()
    );
    decoded
}

fn status(budget: Budget, count: usize) -> Status {
    Status {
        budget,
        quiet: count == 0,
        by: None,
        counts: Counts {
            tasks: count * 2,
            ..Counts::default()
        },
        active: (0..count)
            .map(|n| ActiveTask {
                id: format!("task.active{n}"),
                title: "Working on the parser".to_owned(),
                attribution: Attribution::default(),
                log: Some("The parser passes; continue with the formatter.".to_owned()),
            })
            .collect(),
        review: vec![],
        held: vec![],
        ready: (0..count)
            .map(|n| ReadyTask {
                id: format!("task.ready{n}"),
                title: "Verify the next parser case".to_owned(),
                created: TODAY.to_owned(),
                priority: None,
                attribution: Attribution::default(),
            })
            .collect(),
        untaken: count,
        questions: vec![],
        debt: 0,
    }
}

#[test]
fn toon_and_json_preserve_control_characters_and_unicode_as_the_same_data() {
    let controls: String = (0..=31).map(|n| char::from_u32(n).unwrap()).collect();
    let reply = Reply::Viewed {
        view: anb_core::View {
            id: "note.demo".to_owned(),
            path: "notes/note.demo.md".to_owned(),
            archived: false,
            fields: vec![("title".to_owned(), "# Правило, 雪 🧠".to_owned())],
            body: format!("{controls}\n# a heading\n\"quoted\" and \\u001b"),
            mentions: vec![],
            mentioned_by: vec![],
            linked_by: vec![],
        },
        all: true,
    };
    let value = decoded(&reply);
    let output = text::render(&reply);
    assert!(!output.chars().any(|c| c <= '\u{1f}' && c != '\n'));
    assert_eq!(value["fields"]["rows"][0][1], "# Правило, 雪 🧠");
    assert!(
        value["body"]["head"]
            .as_str()
            .unwrap()
            .starts_with(&controls)
    );
    assert_eq!(output, text::render(&reply));
}

#[test]
fn truncated_arrays_declare_the_visible_count_and_report_where_to_read_the_rest() {
    let reply = Reply::Ready {
        rows: status(Budget::Unbounded, 25).ready,
        filter: anb_core::Filter::default(),
        all: false,
    };
    let value = decoded(&reply);
    assert_eq!(value["count"], 25);
    assert_eq!(value["omitted"], 5);
    assert_eq!(value["ready"].as_array().unwrap().len(), 20);
    assert_eq!(value["more"], "anb ready --all");
    assert!(text::render(&reply).contains("ready[20]"));
}

#[test]
fn recovery_uses_the_same_schema_and_lossless_codec() {
    let recovery = Recovery {
        context: vec![],
        code: "invalid-argument",
        message: "A value contains \"quotes\", commas and \u{1b}[31m.".to_owned(),
        details: vec!["# Not an instruction".to_owned()],
        tries: vec!["anb list --all".to_owned()],
    };
    let output = text::render_recovery(&recovery);
    let value = reddb_io_toon::decode(&output).unwrap().to_json_value();
    assert_eq!(
        value,
        serde_json::from_str::<Value>(&json::render_recovery(&recovery)).unwrap()
    );
    assert_eq!(value["message"], recovery.message);
    assert_eq!(value["findings"][0], "# Not an instruction");
}

#[test]
fn status_budget_measures_the_encoded_document_and_keeps_explicit_omissions() {
    for limit in [1, 100, 300, 600, 1500, 3000] {
        let reply = Reply::Status {
            status: status(Budget::Tokens(limit), 60),
        };
        let value = decoded(&reply);
        let spent = anb_core::estimate_tokens(&text::render(&reply));
        assert_eq!(value["budget"]["spent"], spent);
        assert_eq!(value["budget"]["limit"], limit);
        assert_eq!(value["active"]["count"], 60);
        assert_eq!(value["ready"]["count"], 60);
        for name in ["active", "ready"] {
            let visible = value[name]["rows"].as_array().unwrap().len();
            assert_eq!(value[name]["omitted"], 60 - visible);
        }
        if spent > limit {
            assert!(value["ready"]["rows"].as_array().unwrap().is_empty());
            assert_eq!(value["active"]["rows"].as_array().unwrap().len(), 1);
            assert!(value["active"]["rows"][0].get("log").is_none());
        }
        assert_eq!(value["more"], "anb status --budget 0");
        assert!(value.get("focus").is_none());
    }
}

#[test]
fn an_unbounded_dashboard_includes_every_row() {
    let reply = Reply::Status {
        status: status(Budget::Unbounded, 60),
    };
    let value = decoded(&reply);
    assert_eq!(value["active"]["rows"].as_array().unwrap().len(), 60);
    assert_eq!(value["ready"]["rows"].as_array().unwrap().len(), 60);
    assert_eq!(value["active"]["omitted"], 0);
    assert_eq!(value["budget"]["limit"], Value::Null);
}

#[test]
fn recall_budgets_work_and_memory_together() {
    let memories = (0..12)
        .map(|n| ScopedMemory {
            audience: Audience::Project,
            memory: Memory {
                id: format!("note.memory{n}"),
                path: format!("notes/note.memory{n}.md"),
                record_type: RecordType::Note,
                kind: Some("domain".to_owned()),
                title: "One term in the domain".to_owned(),
                body: "A long definition. ".repeat(1000),
                by: Some("Grace".to_owned()),
                links: vec![],
                related: false,
            },
        })
        .collect();
    let reply = Reply::Recalled(Box::new(Recall {
        work: status(Budget::Tokens(1500), 20),
        focus: None,
        memories,
        invalid: vec![],
        all: false,
        budget: Budget::Tokens(1500),
        more: "anb recall --all".to_owned(),
    }));
    let value = decoded(&reply);
    let spent = anb_core::estimate_tokens(&text::render(&reply));
    assert!(spent <= 1500, "{spent}");
    assert_eq!(value["budget"]["spent"], spent);
    assert_eq!(value["count"], 12);
    assert!(value["work"].get("budget").is_none());
    assert!(
        value["memories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|memory| memory["body"]["omitted"].as_u64().unwrap() > 0)
    );
}

#[test]
fn skill_printing_remains_a_markdown_artifact() {
    let skill = "---\nname: anb\n---\n# Workflow\n";
    assert_eq!(
        text::render(&Reply::Skill(SkillReply::Printed(skill.to_owned()))),
        skill
    );
}

#[test]
fn canonical_v4_strings_quote_comments_and_escape_control_bytes() {
    let value = serde_json::json!({"message":"# guidance", "control":"\0\u{1b}\r\n\t", "雪":"a Unicode key"});
    let encoded =
        reddb_io_toon::encode(&reddb_io_toon::Value::from_json_value(value.clone())).unwrap();
    assert_eq!(
        encoded,
        "message: \"# guidance\"\ncontrol: \"\\u0000\\u001b\\r\\n\\t\"\n\"雪\": a Unicode key"
    );
    assert_eq!(
        reddb_io_toon::decode(&encoded).unwrap().to_json_value(),
        value
    );
}

#[test]
fn recovery_reports_the_real_detail_count_without_a_fake_array_row() {
    let recovery = Recovery {
        code: "still-referenced",
        message: "A record is still referenced.".to_owned(),
        context: vec![],
        details: (0..22).map(|n| format!("task.ref{n}")).collect(),
        tries: vec!["anb show task.demo".to_owned()],
    };
    let value = reddb_io_toon::decode(&text::render_recovery(&recovery))
        .unwrap()
        .to_json_value();
    assert_eq!(value["count"], 22);
    assert_eq!(value["omitted"], 2);
    assert_eq!(
        value["findings"],
        serde_json::json!((0..20).map(|n| format!("task.ref{n}")).collect::<Vec<_>>())
    );
    assert!(text::render_recovery(&recovery).contains("findings[20]"));
}

#[test]
fn status_expansion_preserves_identity_and_widening_is_explicit() {
    let mut model = status(Budget::Tokens(1500), 60);
    model.by = Some("Grace Hopper".to_owned());
    let value = decoded(&Reply::Status { status: model });
    assert_eq!(value["by"], "Grace Hopper");
    assert_eq!(value["more"], "anb status --by 'Grace Hopper' --budget 0");
    assert_eq!(value["team"], "anb status --team");
}

#[test]
fn recall_selects_bounded_work_before_encoding_a_large_notebook() {
    let reply = Reply::Recalled(Box::new(Recall {
        work: status(Budget::Unbounded, 10_000),
        focus: None,
        memories: vec![],
        invalid: vec![],
        all: false,
        budget: Budget::Tokens(1500),
        more: "anb recall --all".to_owned(),
    }));
    let value = decoded(&reply);
    assert_eq!(value["work"]["active"]["count"], 10_000);
    assert!(value["work"]["active"]["rows"].as_array().unwrap().len() <= 5);
    assert!(value["budget"]["spent"].as_u64().unwrap() <= 1500);
}

#[test]
fn only_the_cli_bounds_summary_text_and_both_formats_keep_the_same_values() {
    let title = "Длинное название ".repeat(100);
    let mut model = status(Budget::Unbounded, 1);
    model.active[0].title = title.clone();
    model.active[0].log = Some(title.clone());
    model.ready[0].title = title.clone();
    let reply = Reply::Status { status: model };
    let value = decoded(&reply);
    for text in [
        &value["active"]["rows"][0]["title"],
        &value["active"]["rows"][0]["log"],
        &value["ready"]["rows"][0]["title"],
    ] {
        assert!(text.as_str().unwrap().chars().count() < title.chars().count());
        assert!(text.as_str().unwrap().contains("more characters"));
    }
    let Reply::Status { status, .. } = reply else {
        unreachable!()
    };
    assert_eq!(status.active[0].title, title);
}

#[test]
fn host_action_paths_are_applied_once_before_budget_measurement() {
    let reply = Reply::Status {
        status: status(Budget::Tokens(600), 60),
    };
    let transformations = std::cell::Cell::new(0);
    let path = "workspace/".repeat(50);
    let value = json::value_with(&reply, |value| {
        transformations.set(transformations.get() + 1);
        value["more"] = serde_json::json!(format!("anb status --notebook {path} --budget 0"));
    });
    let encoded = reddb_io_toon::encode(&reddb_io_toon::Value::from_json_value(value.clone()))
        .unwrap()
        + "\n";
    assert_eq!(transformations.get(), 1);
    assert_eq!(
        value["budget"]["spent"],
        anb_core::estimate_tokens(&encoded)
    );
    assert!(value["budget"]["spent"].as_u64().unwrap() <= 600);
    assert!(value["more"].as_str().unwrap().contains(&path));
}

#[test]
fn recall_invalid_files_keep_scope_path_and_repair_separate() {
    let reply = Reply::Recalled(Box::new(Recall {
        work: status(Budget::Unbounded, 0),
        focus: None,
        memories: vec![],
        invalid: vec![anb::recall::ScopedInvalid {
            audience: Audience::Global,
            path: "notes/note.broken.md".to_owned(),
        }],
        all: true,
        budget: Budget::Unbounded,
        more: "anb recall --all".to_owned(),
    }));
    let value = decoded(&reply);
    assert_eq!(
        value["invalid"],
        serde_json::json!({"count":1,"omitted":0,"rows":[{"scope":"global","path":"notes/note.broken.md","repair":"anb check --global"}]})
    );
}

#[test]
fn recall_budgets_the_focused_records_relationships_and_preserves_their_counts() {
    let references: Vec<_> = (0..60)
        .map(|n| format!("note.{n}{}", "reference".repeat(12)))
        .collect();
    let focus = anb_core::View {
        id: "task.focus".to_owned(),
        path: "tasks/task.focus.md".to_owned(),
        archived: false,
        fields: vec![],
        body: String::new(),
        mentions: references.clone(),
        mentioned_by: references.clone(),
        linked_by: references
            .iter()
            .map(|id| ("related".to_owned(), id.clone()))
            .collect(),
    };
    let mut reply = Reply::Recalled(Box::new(Recall {
        work: status(Budget::Unbounded, 0),
        focus: Some(focus),
        memories: vec![],
        invalid: vec![],
        all: false,
        budget: Budget::Tokens(600),
        more: "anb recall --focus task.focus --all".to_owned(),
    }));
    let value = decoded(&reply);
    assert!(value["budget"]["spent"].as_u64().unwrap() <= 600);
    assert_eq!(value["focus"]["id"], "task.focus");
    assert_eq!(value["focus"]["more"], "anb show task.focus --all");
    for name in ["mentions", "mentioned-by", "linked-by"] {
        let section = &value["focus"][name];
        assert_eq!(section["count"], 60);
        assert_eq!(
            section["omitted"],
            60 - section["rows"].as_array().unwrap().len()
        );
    }
    let Reply::Recalled(recall) = &mut reply else {
        unreachable!()
    };
    recall.all = true;
    recall.budget = Budget::Unbounded;
    let value = decoded(&reply);
    for name in ["mentions", "mentioned-by", "linked-by"] {
        assert_eq!(value["focus"][name]["rows"].as_array().unwrap().len(), 60);
        assert_eq!(value["focus"][name]["omitted"], 0);
    }
}
