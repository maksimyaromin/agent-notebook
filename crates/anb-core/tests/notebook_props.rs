//! Format-contract properties 3 and 4 (format spec §4) over the notebook's
//! verbs: a verb never touches another record's bytes, and a replayed verb
//! changes nothing — answering `already: true`, or refusing a move that was
//! never valid to repeat.

use anb_core::{MemoryStorage, Notebook, NotebookError, Proof, Storage};
use proptest::prelude::*;

const TODAY: &str = "2026-08-27";
const TASK_PATH: &str = "tasks/task.demo.md";
const BYSTANDER_PATH: &str = "notes/note.bystander.md";
const BYSTANDER: &str = "---\nid: note.bystander\ntype: note\nstate: active\ntitle: Untouched\ncreated: 2026-08-24\n---\n";

#[derive(Debug, Clone, Copy)]
enum Verb {
    Start,
    Submit,
    Close,
    Return,
    Reopen,
    Hold,
    Unhold,
}

fn apply(storage: &mut MemoryStorage, verb: Verb) -> Result<bool, NotebookError> {
    let mut notebook = Notebook::new(storage);
    let id = "task.demo";
    match verb {
        Verb::Start => notebook.start(id, TODAY).map(|reply| reply.already),
        Verb::Submit => notebook.submit(id, TODAY).map(|reply| reply.already),
        Verb::Close => notebook
            .close(id, &Proof::Sha("f00dfeed".to_owned()), TODAY)
            .map(|reply| reply.transition.already),
        Verb::Return => notebook.return_task(id, TODAY).map(|reply| reply.already),
        Verb::Reopen => notebook.reopen(id, TODAY).map(|reply| reply.already),
        Verb::Hold => notebook
            .hold(id, "a standing reason", None, TODAY)
            .map(|reply| reply.already),
        Verb::Unhold => notebook.unhold(id, TODAY).map(|reply| reply.already),
    }
}

fn verb() -> impl Strategy<Value = Verb> {
    prop_oneof![
        Just(Verb::Start),
        Just(Verb::Submit),
        Just(Verb::Close),
        Just(Verb::Return),
        Just(Verb::Reopen),
        Just(Verb::Hold),
        Just(Verb::Unhold),
    ]
}

/// A parseable task in any state, with the quirks the splice must preserve:
/// an unknown field, an optional standing hold, assorted body shapes, CRLF.
fn generated_task() -> impl Strategy<Value = String> {
    let state = prop_oneof![Just("open"), Just("active"), Just("review"), Just("closed")];
    let custom = proptest::option::of("[a-zA-Z0-9 ]{0,12}");
    let held = any::<bool>();
    let body = prop_oneof![
        Just(""),
        Just("body\n"),
        Just("no newline at the end"),
        Just("---\nfake: envelope\n---\n")
    ];
    let crlf = any::<bool>();
    (state, custom, held, body, crlf).prop_map(|(state, custom, held, body, crlf)| {
        let mut text = format!("---\nid: task.demo\ntype: task\nstate: {state}\n");
        if let Some(custom) = custom {
            text.push_str("custom: ");
            text.push_str(&custom);
            text.push('\n');
        }
        text.push_str("title: A generated task\n");
        if held {
            text.push_str("hold: a standing reason\n");
        }
        text.push_str("created: 2026-08-24\n---\n");
        if crlf {
            text = text.replace('\n', "\r\n");
        }
        text.push_str(body);
        text
    })
}

proptest! {
    #[test]
    fn a_replayed_verb_changes_no_byte_and_never_touches_a_bystander(
        task in generated_task(),
        verb in verb(),
    ) {
        let mut storage = MemoryStorage::from_files([
            (TASK_PATH, task.as_str()),
            (BYSTANDER_PATH, BYSTANDER),
        ]);

        let first = apply(&mut storage, verb);
        let after_first = storage.read(TASK_PATH).unwrap();
        if first.is_err() {
            prop_assert_eq!(&after_first, &task, "a refused verb must not write");
        }

        let second = apply(&mut storage, verb);
        match (first, second) {
            (Ok(_), Ok(already)) => prop_assert!(already, "a replay must say so"),
            // `return` refuses its own replay — active is not proof of a
            // return, it may never have been submitted — and a refused verb
            // refuses again.
            (Ok(_), Err(NotebookError::InvalidTransition { .. })) | (Err(_), Err(_)) => {}
            (first, second) => prop_assert!(
                false,
                "replay changed the outcome class: {first:?} then {second:?}"
            ),
        }
        prop_assert_eq!(
            storage.read(TASK_PATH).unwrap(),
            after_first,
            "a replay must be byte-identical"
        );
        prop_assert_eq!(
            storage.read(BYSTANDER_PATH).unwrap(),
            BYSTANDER,
            "no verb may touch another record's bytes"
        );
    }
}
