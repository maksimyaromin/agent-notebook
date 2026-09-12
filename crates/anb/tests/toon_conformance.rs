//! The pinned official fixture suite, invoked explicitly with `ANB_TOON_SPEC`.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

const SPEC_REVISION: &str = "d6db4b04303bdea132351ce45aed612311c850b2";

#[test]
#[ignore = "requires a pinned ANB_TOON_SPEC checkout; see tools/evaluation/README.md"]
fn official_v4_1_fixtures_and_adversarial_round_trips() {
    let spec = PathBuf::from(
        std::env::var_os("ANB_TOON_SPEC").expect("set ANB_TOON_SPEC to the spec checkout"),
    );
    assert_eq!(git(&spec, &["rev-parse", "HEAD"]).trim(), SPEC_REVISION);
    assert!(
        git(
            &spec,
            &[
                "status",
                "--porcelain",
                "--untracked-files=all",
                "--",
                "tests/fixtures"
            ]
        )
        .is_empty(),
        "the official fixture tree must be unchanged"
    );
    let mut cases = 0;
    let mut failures = Vec::new();
    let mut files = 0;
    for mode in ["encode", "decode"] {
        let directory = spec.join("tests/fixtures").join(mode);
        let mut paths: Vec<_> = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        paths.sort();
        for path in paths {
            assert_eq!(
                path.extension().and_then(|value| value.to_str()),
                Some("json")
            );
            files += 1;
            let fixture: Value =
                serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            assert_eq!(fixture["category"], mode);
            for (index, case) in fixture["tests"].as_array().unwrap().iter().enumerate() {
                cases += 1;
                let result = run_case(mode, case);
                let accepted = if case["shouldError"] == true {
                    result.is_err()
                } else {
                    result.as_ref().ok() == Some(&case["expected"])
                };
                if !accepted {
                    failures.push(format!(
                        "{mode}/{}[{index}]: expected {}, got {result:?}",
                        path.file_name().unwrap().to_string_lossy(),
                        case["expected"]
                    ));
                }
            }
        }
    }
    assert_eq!(files, 23, "every official fixture file must run");
    assert_eq!(cases, 538, "no fixture may be skipped");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    adversarial_round_trips();
    println!(
        "538 official fixtures passed; 4 adversarial round-trips passed; spec {SPEC_REVISION}"
    );
}

fn git(directory: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .args(arguments)
        .current_dir(directory)
        .output()
        .expect("git must be available to verify the fixture revision");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn run_case(mode: &str, case: &Value) -> Result<Value, String> {
    let options = &case["options"];
    let known = if mode == "encode" {
        ["indentSize", "delimiter"]
    } else {
        ["indentSize", "strict"]
    };
    if let Some(options) = options.as_object() {
        for key in options.keys() {
            assert!(
                known.contains(&key.as_str()),
                "unhandled official option: {key}"
            );
        }
    }
    let indent = options["indentSize"]
        .as_u64()
        .map(|indent| usize::try_from(indent).unwrap());
    if mode == "encode" {
        let mut configuration = reddb_io_toon::EncodeOptions::default();
        if let Some(indent) = indent {
            configuration.indent_size = indent;
        }
        if let Some(delimiter) = options["delimiter"].as_str() {
            assert_eq!(delimiter.chars().count(), 1);
            configuration.delimiter = delimiter.chars().next().unwrap();
        }
        let input = reddb_io_toon::Value::from_json_value(case["input"].clone());
        reddb_io_toon::encode_with_options(&input, configuration)
            .map(Value::String)
            .map_err(|error| error.to_string())
    } else {
        let mut configuration = reddb_io_toon::DecodeOptions::default();
        if let Some(indent) = indent {
            configuration.indent = indent;
        }
        if let Some(strict) = options["strict"].as_bool() {
            configuration.strict = strict;
        }
        reddb_io_toon::decode_with_options(case["input"].as_str().unwrap(), &configuration)
            .map(|value| value.to_json_value())
            .map_err(|error| error.to_string())
    }
}

fn adversarial_round_trips() {
    let controls: String = (0..=31).map(|n| char::from_u32(n).unwrap()).collect();
    for value in [
        json!({"body": controls, "title": "# comment", "non-ascii": "Жёлтый 🧠", "雪": "☃"}),
        json!({"rows": [{"id":"task.a","title":"# heading, with comma"},{"id":"task.b","title":"a\\\"\nb"}]}),
        json!({"numbers": [0, 9_223_372_036_854_775_808_u64, u64::MAX]}),
        json!({"bodies": ["# heading", "- list", "\u{1b}[31mred", "\\u001b"]}),
    ] {
        let encoded = anb::json::toon(&value);
        assert_eq!(encoded, anb::json::toon(&value));
        assert!(!encoded.chars().any(|c| c.is_control() && c != '\n'));
        assert_eq!(
            reddb_io_toon::decode(&encoded).unwrap().to_json_value(),
            value
        );
    }
}
