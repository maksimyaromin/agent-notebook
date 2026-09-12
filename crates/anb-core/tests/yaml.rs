//! Interoperability with readers that interpret Markdown frontmatter as YAML.

use anb_core::{Draft, Edit, Link, MemoryStorage, Notebook, RecordFile, RecordType, Storage};
use proptest::prelude::*;
use yaml_rust2::{Yaml, YamlLoader};

fn envelope(text: &str) -> Yaml {
    let header = text
        .strip_prefix("---\n")
        .unwrap()
        .split_once("\n---")
        .unwrap()
        .0;
    let mut documents = YamlLoader::load_from_str(header).expect("the envelope is valid YAML");
    documents.remove(0)
}

fn legacy_record(extra: &str, body: &str) -> String {
    format!(
        "---\nid: note.legacy\ntype: note\nstate: active\ntitle: Report: historical evidence\ncreated: 2020-03-04\n{extra}---\n{body}"
    )
}

#[test]
fn a_created_record_keeps_yaml_punctuation_inside_its_title() {
    let mut storage = MemoryStorage::default();
    let title = "Report: preserve # evidence and \"quotes\"";
    let mut draft = Draft::new(RecordType::Note, title);
    draft.id = Some("note.report".to_owned());
    let created = Notebook::new(&mut storage)
        .create(&draft, "2026-09-12")
        .unwrap();
    assert_eq!(
        envelope(&storage.read(&created.path).unwrap())["title"].as_str(),
        Some(title)
    );
}

#[test]
fn scalar_typing_and_escapes_do_not_change_a_title() {
    for title in [
        "null",
        "TRUE",
        "off",
        "2020-03-04",
        "0xFF",
        "1:20",
        "+0xFF",
        "~",
        "[a, b]",
        "&alias",
        "*alias",
        "!tag",
        "has # evidence",
        "back\\slash",
        "\"quoted\"",
        "'quoted'",
        "a\tb",
        "a\u{0085}b",
        "a\u{2028}b",
        "a\u{fffe}b",
        "Emoji 🧠",
    ] {
        let mut storage = MemoryStorage::default();
        let mut draft = Draft::new(RecordType::Note, title);
        draft.id = Some("note.scalar".to_owned());
        let created = Notebook::new(&mut storage)
            .create(&draft, "2026-09-12")
            .unwrap();
        let text = storage.read(&created.path).unwrap();
        assert_eq!(envelope(&text)["title"].as_str(), Some(title), "{title:?}");
        assert_eq!(RecordFile::parse(&text).field("title"), Some(title));
    }
}

#[test]
fn yaml_readers_receive_every_link_in_a_created_record() {
    let mut storage = MemoryStorage::default();
    let mut draft = Draft::new(RecordType::Note, "Evidence");
    draft.links = ["https://example.com/one", "https://example.com/two"]
        .into_iter()
        .map(|target| Link {
            kind: "doc".to_owned(),
            target: target.to_owned(),
        })
        .collect();
    let created = Notebook::new(&mut storage)
        .create(&draft, "2026-09-12")
        .unwrap();
    let header = envelope(&storage.read(&created.path).unwrap());
    let links: Vec<_> = header["link"]
        .as_vec()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(
        links,
        ["doc https://example.com/one", "doc https://example.com/two"]
    );
}

#[test]
fn quoted_values_and_block_sequences_read_without_losing_the_original_bytes() {
    let text = "---\nid: task.example\ntype: task\nstate: open\ntitle: \"Read: \\\"quoted\\\" values\"\nby: 'Ada''s agent'\nblocked-by:\n  - task.first\n  - task.second\ncreated: 2020-03-04\n---\nBody.\n";
    let file = RecordFile::parse(text);
    assert!(!file.has_errors(), "{:?}", file.findings());
    assert_eq!(file.field("title"), Some("Read: \"quoted\" values"));
    assert_eq!(file.field("by"), Some("Ada's agent"));
    assert_eq!(
        file.field_values("blocked-by").collect::<Vec<_>>(),
        ["task.first", "task.second"]
    );
    assert_eq!(file.render(), text);
}

#[test]
fn editing_a_legacy_record_preserves_its_history_and_unknown_fields_in_valid_yaml() {
    let mut storage = MemoryStorage::default();
    let body = "Historical evidence.\r\n\r\n---\r\nNo final newline";
    storage.write("notes/note.legacy.md", &legacy_record("custom: Source: original # evidence\nlink: doc https://example.com/one\nlink: doc https://example.com/two\n", body)).unwrap();
    Notebook::new(&mut storage)
        .edit(
            "note.legacy",
            &Edit {
                add_tags: vec!["history".to_owned()],
                ..Edit::default()
            },
            "2026-09-12",
        )
        .unwrap();
    let text = storage.read("notes/note.legacy.md").unwrap();
    let header = envelope(&text);
    assert_eq!(header["id"].as_str(), Some("note.legacy"));
    assert_eq!(header["created"].as_str(), Some("2020-03-04"));
    assert_eq!(
        header["custom"].as_str(),
        Some("Source: original # evidence")
    );
    assert_eq!(header["link"].as_vec().unwrap().len(), 2);
    assert_eq!(RecordFile::parse(&text).body(), body);
}

#[test]
fn a_no_op_edit_keeps_a_legacy_envelope_byte_exact() {
    let text = legacy_record("custom: Original: evidence\n", "No final newline");
    let mut storage = MemoryStorage::from_files([("notes/note.legacy.md", text.clone())]);
    Notebook::new(&mut storage)
        .edit(
            "note.legacy",
            &Edit {
                title: Some("Report: historical evidence".to_owned()),
                ..Edit::default()
            },
            "2026-09-12",
        )
        .unwrap();
    assert_eq!(storage.read("notes/note.legacy.md").unwrap(), text);
}

#[test]
fn unknown_keys_and_repeated_values_survive_canonical_yaml() {
    let text = legacy_record("null: First: value\nnull: Second # value\n", "Evidence");
    let canonical = RecordFile::parse(&text).normalize();
    let yaml = envelope(&canonical);
    assert_eq!(yaml["null"][0].as_str(), Some("First: value"));
    assert_eq!(yaml["null"][1].as_str(), Some("Second # value"));
    assert_eq!(
        RecordFile::parse(&canonical)
            .field_values("null")
            .collect::<Vec<_>>(),
        ["First: value", "Second # value"]
    );
}

#[test]
fn unsupported_nested_content_refuses_migration_without_creating_a_backup() {
    let text = legacy_record("custom:\n  nested: value\n", "Evidence");
    let mut storage = MemoryStorage::from_files([("notes/note.legacy.md", text.clone())]);
    assert!(Notebook::new(&mut storage).migrate(false).is_err());
    assert_eq!(storage.read("notes/note.legacy.md").unwrap(), text);
    assert!(storage.list(".migrations.tmp").unwrap().is_empty());
}

proptest! {
    #[test]
    fn every_single_line_title_round_trips_through_an_independent_yaml_reader(
        characters in proptest::collection::vec(any::<char>().prop_filter("single line", |character| !matches!(character, '\r' | '\n')), 0..96)
    ) {
        let title = format!("Value {} end", characters.into_iter().collect::<String>());
        let mut draft = Draft::new(RecordType::Note, &title);
        draft.id = Some("note.arbitrary".to_owned());
        let mut storage = MemoryStorage::new();
        let created = Notebook::new(&mut storage).create(&draft, "2026-09-12").unwrap();
        let text = storage.read(&created.path).unwrap();
        let yaml = envelope(&text);
        let parsed = RecordFile::parse(&text);
        prop_assert_eq!(yaml["title"].as_str(), Some(title.as_str()));
        prop_assert_eq!(parsed.field("title"), Some(title.as_str()));
    }
}
