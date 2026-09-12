//! Format-contract properties over the notebook's
//! verbs: a verb never touches another record's bytes, and a replayed verb
//! changes nothing — answering `already: true`, or refusing a move that was
//! never valid to repeat.

use anb_core::{CitedProof, MemoryStorage, Notebook, NotebookError, Storage};
use proptest::prelude::*;
const TODAY: &str = "2026-08-27";

/// A Status asked with nothing to settle against the world outside the
/// notebook: the host finds every cited proof still there.
fn no_lost_proofs(_cited: &[CitedProof]) -> Vec<CitedProof> {
    Vec::new()
}
const TASK_PATH: &str = "tasks/task.demo.md";
const BYSTANDER_PATH: &str = "notes/note.bystander.md";
const BYSTANDER: &str = "---\nid: note.bystander\ntype: note\nstate: active\ntitle: Untouched\ncreated: 2026-08-24\n---\n";
const BLOCKER_PATH: &str = "tasks/task.blocker.md";
const BLOCKER: &str =
    "---\nid: task.blocker\ntype: task\nstate: open\ntitle: A blocker\ncreated: 2026-08-24\n---\n";

#[derive(Debug, Clone, Copy)]
enum Verb {
    Start,
    Submit,
    Close,
    CloseWithReason,
    Reopen,
    Hold,
    Unhold,
    Block,
    Unblock,
}

fn apply(storage: &mut MemoryStorage, verb: Verb) -> Result<bool, NotebookError> {
    let mut notebook = Notebook::new(storage);
    let id = "task.demo";
    match verb {
        Verb::Start => notebook.start(id, TODAY).map(|reply| reply.already),
        Verb::Submit => notebook.submit(id, None, TODAY).map(|reply| reply.already),
        Verb::Close => notebook
            .close(id, None, "Verified f00dfeed.", TODAY)
            .map(|reply| reply.transition.already),
        Verb::CloseWithReason => notebook
            .close_with_reason(id, "overtaken", TODAY)
            .map(|reply| reply.transition.already),
        Verb::Reopen => notebook.reopen(id, TODAY).map(|reply| reply.already),
        Verb::Hold => notebook
            .hold(id, "a standing reason", None, TODAY)
            .map(|reply| reply.already),
        Verb::Unhold => notebook.unhold(id, TODAY).map(|reply| reply.already),
        Verb::Block => notebook
            .block(id, "task.blocker", TODAY)
            .map(|reply| reply.already),
        Verb::Unblock => notebook
            .unblock(id, "task.blocker", TODAY)
            .map(|reply| reply.already),
    }
}

fn verb() -> impl Strategy<Value = Verb> {
    prop_oneof![
        Just(Verb::Start),
        Just(Verb::Submit),
        Just(Verb::Close),
        Just(Verb::CloseWithReason),
        Just(Verb::Reopen),
        Just(Verb::Hold),
        Just(Verb::Unhold),
        Just(Verb::Block),
        Just(Verb::Unblock),
    ]
}

/// A parseable task in any state, with the quirks the splice must preserve:
/// an unknown field, an optional standing hold, an optional edge onto the
/// blocker, assorted body shapes, CRLF. Whether the edge stands is answered
/// beside the text, because the edge verbs are held to it.
fn generated_task() -> impl Strategy<Value = (String, bool)> {
    let state = prop_oneof![Just("open"), Just("active"), Just("review"), Just("closed")];
    let custom = proptest::option::of("[a-zA-Z0-9 ]{0,12}");
    let held = any::<bool>();
    let blocked = any::<bool>();
    let body = prop_oneof![
        Just(""),
        Just("body\n"),
        Just("no newline at the end"),
        Just("---\nfake: envelope\n---\n")
    ];
    let crlf = any::<bool>();
    (state, custom, held, blocked, body, crlf).prop_map(
        |(state, custom, held, blocked, body, crlf)| {
            let mut text = format!("---\nid: task.demo\ntype: task\nstate: {state}\n");
            if let Some(custom) = custom {
                text.push_str("custom: ");
                text.push_str(&custom);
                text.push('\n');
            }
            text.push_str("title: A generated task\n");
            if blocked {
                text.push_str("blocked-by: task.blocker\n");
            }
            if held {
                text.push_str("hold: a standing reason\n");
            }
            text.push_str("created: 2026-08-24\n---\n");
            if crlf {
                text = text.replace('\n', "\r\n");
            }
            text.push_str(body);
            (text, blocked)
        },
    )
}

proptest! {
    #[test]
    fn a_replayed_verb_changes_no_byte_and_never_touches_a_bystander(
        (task, _) in generated_task(),
        verb in verb(),
    ) {
        let mut storage = MemoryStorage::from_files([
            (TASK_PATH, task.as_str()),
            (BYSTANDER_PATH, BYSTANDER),
            (BLOCKER_PATH, BLOCKER),
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
        prop_assert_eq!(
            storage.read(BLOCKER_PATH).unwrap(),
            BLOCKER,
            "an edge lives on the dependent alone; the blocker's bytes stay"
        );
    }
}

fn edge_verb() -> impl Strategy<Value = Verb> {
    prop_oneof![Just(Verb::Block), Just(Verb::Unblock)]
}

proptest! {
    /// The edge verbs answer for the edge as the file carries it: `block`
    /// finds one already standing, `unblock` erases one that stands. Replay
    /// identity alone cannot tell an edge a quirk hid from the verb apart
    /// from a replay, since both leave every byte alone and answer
    /// `already`.
    #[test]
    fn an_edge_verb_answers_for_the_edge_the_file_carries(
        (task, carries_edge) in generated_task(),
        verb in edge_verb(),
    ) {
        let mut storage =
            MemoryStorage::from_files([(TASK_PATH, task.as_str()), (BLOCKER_PATH, BLOCKER)]);
        let leaves_it_standing = matches!(verb, Verb::Block);
        prop_assert_eq!(
            apply(&mut storage, verb),
            Ok(carries_edge == leaves_it_standing),
            "an edge verb answers for the edge the file carries"
        );
    }
}

/// Two promises the Budget makes. However small the ceiling, the rendered
/// Status fits it — or the ladder has reached its floor, which ships
/// regardless because the first active line is never dropped. And under
/// the default ceiling nothing is cut at all: every section is bounded, so
/// no notebook can grow a dashboard past the budget it was given. Section
/// content and section count vary freely; neither promise may.
mod status_fits_its_budget {
    use super::*;
    use anb_core::Budget;

    fn task(index: usize, state: &str, extra: &str) -> (String, String) {
        (
            format!("tasks/task.t{index}.md"),
            format!(
                "---\nid: task.t{index}\ntype: task\nstate: {state}\ntitle: Task number {index} of this notebook\n{extra}created: 2026-08-1{}\nupdated: 2026-08-2{}\n---\n",
                index % 10,
                index % 8,
            ),
        )
    }

    proptest! {
        #[test]
        fn any_budget_preserves_the_complete_status_model(
            open_tasks in 0usize..40,
            active_tasks in 0usize..20,
            aged_questions in 0usize..30,
            epics in 0usize..15,
            ceiling in 1u32..600,
        ) {
            let mut files: Vec<(String, String)> = Vec::new();
            for index in 0..open_tasks {
                files.push(task(index, "open", ""));
            }
            // Each hub waits on one open Task: the child reaches the queue
            // and the hub stays out of it, so the fixture holds blocked open
            // Tasks beside the dispatchable ones.
            for index in 0..epics.min(open_tasks) {
                files.push(task(200 + index, "open", &format!("blocked-by: task.t{index}\n")));
                files[index].1 = files[index]
                    .1
                    .replace("\ncreated:", &format!("\nfrom: task.t{}\ncreated:", 200 + index));
            }
            for index in 0..active_tasks {
                files.push(task(100 + index, "active", ""));
            }
            for index in 0..aged_questions {
                files.push((
                    format!("questions/question.q{index}.md"),
                    format!(
                        "---\nid: question.q{index}\ntype: question\nstate: open\ntitle: An open doubt\ncreated: 2026-08-01\nupdated: 2026-08-01\n---\n"
                    ),
                ));
            }
            let mut storage = MemoryStorage::from_files(files);
            let notebook = Notebook::new(&mut storage);
            let status = notebook
                .status(TODAY, Budget::Tokens(ceiling), None, no_lost_proofs)
                .unwrap();
            prop_assert_eq!(status.budget, Budget::Tokens(ceiling));
            prop_assert_eq!(status.active.len(), active_tasks);
            prop_assert_eq!(status.ready.len(), open_tasks);
            prop_assert_eq!(status.questions.len(), aged_questions);
            prop_assert_eq!(status.counts.tasks, open_tasks + active_tasks + epics.min(open_tasks));
            let unbounded = notebook.status(TODAY, Budget::Unbounded, None, no_lost_proofs).unwrap();
            prop_assert_eq!(status.active, unbounded.active);
            prop_assert_eq!(status.ready, unbounded.ready);
            prop_assert_eq!(status.questions, unbounded.questions);
        }
    }
}

mod minting {
    use super::*;
    use anb_core::{Draft, Record, RecordType};

    proptest! {
        /// A title is free text a caller types; the record minted from it
        /// is not. Whatever the title, the file that lands reads back clean.
        #[test]
        fn a_minted_record_reads_back_clean(title in "\\PC{0,200}") {
            let mut storage = MemoryStorage::new();
            let draft = Draft::new(RecordType::Task, &title);
            match Notebook::new(&mut storage).create(&draft, TODAY) {
                Ok(created) => {
                    let text = storage.read(&created.path).unwrap();
                    let record = Record::parse(&created.path, &text);
                    prop_assert!(
                        !record.has_errors(),
                        "{title:?} minted {}: {:?}",
                        created.id,
                        record.findings()
                    );
                }
                // A title a mint cannot slug is refused as the bad argument
                // it is, and a refusal writes nothing.
                Err(refusal) => {
                    prop_assert!(
                        matches!(refusal, NotebookError::InvalidArgument { .. }),
                        "{title:?} was refused as {refusal:?}"
                    );
                    prop_assert!(storage.list("tasks").unwrap().is_empty());
                }
            }
        }

        /// The cut lands on a boundary whenever one exists: a slug that was
        /// shortened either kept whole words or, its first word already over
        /// the cap, had no hyphen to stop at. The first word ranges past the
        /// cap so both halves are generated.
        #[test]
        fn a_shortened_slug_keeps_whole_words(title in "[a-z]{1,60}( [a-z]{1,12}){0,20}") {
            let mut storage = MemoryStorage::new();
            let draft = Draft::new(RecordType::Task, &title);
            let created = Notebook::new(&mut storage).create(&draft, TODAY).unwrap();
            let slug = created.id.strip_prefix("task.").expect("a task id");
            let words: Vec<&str> = title.split(' ').collect();
            prop_assert!(
                words.starts_with(&slug.split('-').collect::<Vec<&str>>())
                    || words[0].len() > slug.len(),
                "`{slug}` is neither a whole-word prefix of {words:?} nor a cut first word"
            );
        }
    }
}
