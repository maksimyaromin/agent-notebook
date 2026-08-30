//! The format contract as properties: parsing is total and
//! lossless over any input, and normalize is idempotent.

use anb_core::RecordFile;
use proptest::prelude::*;

proptest! {
    #[test]
    fn any_input_renders_back_byte_exact(input in any_file_text()) {
        prop_assert_eq!(RecordFile::parse(&input).render(), input);
    }

    #[test]
    fn normalize_is_idempotent_on_any_input(input in any_file_text()) {
        let once = RecordFile::parse(&input).normalize();
        let twice = RecordFile::parse(&once).normalize();
        prop_assert_eq!(twice, once);
    }

    #[test]
    fn a_well_formed_envelope_with_any_body_is_accepted_and_preserved(
        record in well_formed_record()
    ) {
        let file = RecordFile::parse(&record.text);
        prop_assert!(!file.has_errors(), "findings: {:?}", file.findings());
        prop_assert_eq!(&file.render(), &record.text);
        prop_assert_eq!(file.body(), record.body.as_str());

        let once = file.normalize();
        prop_assert!(
            RecordFile::parse(&once).findings().is_empty(),
            "canonical form must reparse clean; got {:?}",
            RecordFile::parse(&once).findings()
        );
        prop_assert_eq!(RecordFile::parse(&once).normalize(), once);
    }
}

/// Any text a file could hold, as lines that may be fences, field lines, or
/// anything at all, under either line terminator.
///
/// `any::<String>()` is proptest's `\PC*` and so carries no newline: a file
/// built from one is a body with no envelope, and a property over it never
/// reaches the envelope at all.
fn any_file_text() -> impl Strategy<Value = String> {
    let line = prop_oneof![
        Just("---".to_owned()),
        Just(String::new()),
        "[a-z-]{1,10}:[ \t]{0,3}[^\n\r]{0,24}",
        any::<String>(),
    ];
    (
        proptest::collection::vec(line, 0..12),
        prop_oneof![Just("\n"), Just("\r\n")],
        any::<bool>(),
    )
        .prop_map(|(lines, separator, trailing)| {
            let mut text = lines.join(separator);
            if trailing {
                text.push_str(separator);
            }
            text
        })
}

#[derive(Debug)]
struct GeneratedRecord {
    text: String,
    body: String,
}

/// A record the write side could have produced — required fields with valid
/// values, optional extras, any field order — over an arbitrary body.
fn well_formed_record() -> impl Strategy<Value = GeneratedRecord> {
    let type_word = prop_oneof![
        Just("task"),
        Just("decision"),
        Just("note"),
        Just("question")
    ];
    let slug = "[a-z0-9]([a-z0-9-]{0,20}[a-z0-9])?";
    let state = "[a-z]{1,12}";
    let title = "[a-zA-Z0-9][a-zA-Z0-9 :#'()-]{0,38}[a-zA-Z0-9]";
    let date =
        (1990u16..=2100, 1u8..=12, 1u8..=28).prop_map(|(y, m, d)| format!("{y:04}-{m:02}-{d:02}"));
    let tags = proptest::option::of("[a-z0-9]{1,8}(, [a-z0-9]{1,8}){0,3}");
    let link_values = proptest::collection::vec("[a-z]{1,6} [a-zA-Z0-9./_-]{1,20}", 0..3);

    (type_word, slug, state, title, date, tags, link_values)
        .prop_flat_map(|(ty, slug, state, title, date, tags, link_values)| {
            let mut lines = vec![
                format!("id: {ty}.{slug}"),
                format!("type: {ty}"),
                format!("state: {state}"),
                format!("title: {title}"),
                format!("created: {date}"),
            ];
            if let Some(tags) = tags {
                lines.push(format!("tags: {tags}"));
            }
            for value in link_values {
                lines.push(format!("link: {value}"));
            }
            (Just(lines).prop_shuffle(), any::<String>())
        })
        .prop_map(|(lines, body)| {
            let mut text = String::from("---\n");
            for line in &lines {
                text.push_str(line);
                text.push('\n');
            }
            text.push_str("---\n");
            text.push_str(&body);
            GeneratedRecord { text, body }
        })
}
