//! The `anb` method and heavy CLI references share one rendering so setup
//! and the committed skill teach the same commands. Worked replies execute
//! against a scratch notebook; CI checks the committed copy for drift.

use crate::cli::{Cli, Command, NARROWING};
use crate::recovery::subject;
use crate::reply::{Host, execute};
use crate::text;
use anb_core::{FindingCode, MemoryStorage, Severity, Storage as _, StorageError};
use clap::{Arg, CommandFactory as _, Parser as _};
use std::fmt::Write as _;
use std::path::Path;

/// The skill's directory name under a host's `skills/`.
pub const NAME: &str = "anb";

pub const SKILL_FILE: &str = "SKILL.md";
pub const COMMANDS_FILE: &str = "references/commands.md";
pub const SESSION_FILE: &str = "references/session.md";
pub const REFUSALS_FILE: &str = "references/refusals.md";

/// The day every example runs on, so the rendering is the same on every
/// machine and the ages in it read as written.
const TODAY: &str = "2026-01-15";
const AUTHOR: &str = "Ada";

/// The skill and the references installed with it.
#[derive(Debug, PartialEq, Eq)]
pub struct Skill {
    pub skill: String,
    pub commands: String,
    pub session: String,
    pub refusals: String,
}

impl Skill {
    #[must_use]
    pub fn files(&self) -> [(&'static str, &str); 4] {
        [
            (SKILL_FILE, &self.skill),
            (COMMANDS_FILE, &self.commands),
            (SESSION_FILE, &self.session),
            (REFUSALS_FILE, &self.refusals),
        ]
    }
}

#[must_use]
pub fn render() -> Skill {
    Skill {
        skill: skill_md(),
        commands: reference(
            "anb commands",
            "Every anb command with its flags, from the definitions anb --help prints. Open before a verb you have not used.",
            commands_section,
        ),
        session: reference(
            "anb worked session",
            "One notebook worked from empty to archived work, every reply as the tool printed it. Open to see what a reply looks like before you parse one.",
            session_section,
        ),
        refusals: reference(
            "anb refusals",
            "Every refusal code with its cause and recovery instruction, and the findings anb check raises. Open when a refusal's suggested correction is not enough.",
            |out| {
                refusals_section(out);
                findings_section(out);
            },
        ),
    }
}

// ---------------------------------------------------------------------------
// SKILL.md — the method
// ---------------------------------------------------------------------------

fn skill_md() -> String {
    let mut out = String::new();
    out.push_str("---\nname: anb\n");
    out.push_str("description: Use when working in a repository with .agent-notebook, capturing or shaping an idea, modeling a domain, planning or continuing Tasks and epics, recording project knowledge, reviewing status, or when a session hook reports active work.\n");
    out.push_str("metadata:\n  managed-by: anb\n---\n\n");
    out.push_str(SKILL_BODY);
    out
}

const SKILL_BODY: &str = r#"# anb

## Overview

Keep the problem, the reasoning and the work connected so another session can continue without reconstructing the conversation. Use the CLI for every notebook change; it maintains record state and relationships together.

## When to use

Use this method when capturing a request, shaping an idea, maintaining domain knowledge, planning delivery or continuing project work. For a status question, read and answer; a read does not need a new Task. Follow the user's chosen notebook location, workflow and sharing policy.

## Core pattern

Start a new change with an `idea` Note, or resume the existing idea. Keep the source, intended improvement, constraints, agreement status and next uncertainty in its body. Link the source with `--link "doc <path-or-url>"`; record missing evidence explicitly. Capturing a request does not authorize implementation or changes to its source.

Keep the idea when Tasks emerge: one proposal can lead to several deliveries. Create records `--from` what produced them, cite supporting records by bare id, and use `block` for execution prerequisites. An origin answers why a record exists; a mention supplies context; a dependency controls readiness.

For an agreed small change, the idea and one Task are enough:

```sh
anb add note "Name the CSV download" --id note.csv-download --kind idea --via codex --body "Agreed: rename Export to Download CSV so the label states the format. Preserve behavior and file contents. Implementation is queued for later."
anb add task "Rename the CSV download button" --id task.csv-download --from note.csv-download --via codex --body "Implement note.csv-download. Verify the label and that the same action produces unchanged CSV content."
anb check
```

These explicit ids make the example runnable. In ordinary work, use the ids returned by the CLI, including collision suffixes. Add `--via` to every agent `add` and `comment`, using your actual tool name, such as `codex` or `claude-code`. Leave `by` to the accountable person: the CLI signs it from `ANB_BY` or the git identity, and a comment is signed `by/via`, so the person stays in the trail beside the tool. Other verbs do not accept `--via`.

## Quick reference

Choose by what a later reader needs, with an explicit `--kind` for Notes and Decisions.

| Need | Record | Why keep it separately |
|---|---|---|
| Deliver or investigate a checkable result | Task | Progress and completion belong to the work |
| Settle an uncertainty that changes the work | Question | An unanswered choice must remain visible |
| Preserve an agreed requirement | Decision `rule` | Later work must respect its scope and reason |
| Explain a chosen design | Decision `shape` | Alternatives and the deciding constraint prevent repeated debate |
| Allow an agreed exception | Decision `drift` | The affected rule and revisit condition bound the departure |
| Reuse an observation | Note `fact` | Evidence and limits distinguish a finding from a guess |
| Define a word in context | Note `term` | Ambiguous vocabulary changes how requirements are read |
| Repeat a procedure | Note `guide` | Conditions and verification make it reusable |
| Develop a possibility | Note `idea` | Motivation outlives any one delivery |
| Explain a domain | Note `model` | Ownership, relationships and invariants need more than definitions |
| Specify expected behavior | Note `spec` | Scope, exclusions and acceptance criteria guide delivery |

## Working method

### Orient

Read `anb status` unless the hook supplied it. Status is the work: the active Tasks with the last log line of the first, work in review or on hold, the queue, the open Questions and a count of Debt. Follow the requested subject; otherwise resume the active Task or choose from `ready`. Before the work, read the standing rules with `anb list --type decision --kind rule` and open the ones its subject touches, then the knowledge the Task cites and the relevant code. Search before creating records: `anb list --match <text>` matches ids, titles, tags, the people named in the envelope and bodies, and `--archive` reaches history. Use `show --all` for a truncated body and narrowed lists for larger work; loading the whole notebook obscures the immediate decision.

Every listing narrows the same way: `--for <hub>`, `--tag`, `--match <text>`, `--by <name>`, `--mine` and `--team` compose on `list`, `ready` and `graph`, and `--type`, `--kind` and `--archive` on `list` and `graph`, each answering with the records every flag admits. `list --type note --tag domain-model` is the domain language; `ready --tag parser` is one area's queue; `anb debt` is the Debt that Status counts.

Several people can share one notebook, and nobody assigns work in it: a Task is taken. Status lists the user's own active Tasks first and marks another person's with their name, so resume only an unmarked line. In `ready`, the `taken-by` column names a Task someone already took; choose one nobody took, or one the user took, and read `ready --mine` when the queue is long. A notebook whose config sets `scope: mine` answers every read with the user's own records by default; `--team` widens one call to the whole project, which is how new work is chosen when the user's own queue is empty. `start` records the user as the Task's `taken-by` and refuses a Task another person took; handing it over is `edit <id> --taken-by <name>`, decided by the user, never by the agent.

### Shape the idea

Let the next uncertainty choose the investigation. Compare alternatives against the same outcome and constraints. Save reusable evidence as facts, unresolved choices as Questions, and settled choices as Decisions, each with its actual origin. Keep local progress in the Task log. Bring the user your findings, recommendation and the remaining question the evidence cannot answer.

For sustained research, start an investigation Task from the idea, with evidence or a discussable design as its result. For brief intake, leave the next step in the idea; Status can be quiet without an active Task. An idea or spec must state whether its direction is proposed or agreed and what establishes that agreement: a Note's `active` state means maintained, not approved or implemented.

Create a spec when expected behavior needs its own maintained document. Preserve canonical sources when importing existing material: record the useful conclusion and link to the original, keeping private evidence under the chosen sharing policy. Judge how much structure the work needs; a fixed set of documents adds maintenance without answering a question.

### Model the domain

Start from concrete scenarios and check them against the code. Explain who owns state, which changes must agree, what may happen and what information crosses contexts. Distinguish existing behavior from a proposed model. A directory or class name alone does not establish an aggregate or bounded context.

| A term defines | A model explains | Why the distinction matters |
|---|---|---|
| A candidate operation | Who approves it, what approval changes and what happens after rejection | Definitions alone cannot establish allowed behavior |
| A package within one context | How each context uses it and translates information for another | The same word need not describe the same concept |

Keep local definitions in the model; extract terms when they need independent lookup or reuse. Split models where language or responsibility differs. A context map cites those models and explains integration direction and meaning. Tag models `domain-model` and related records by context so search finds them. Cite governing Decisions from models and models from specs and Tasks; keep each ruling in one place. Write a model or a spec of several paragraphs from a file with `--body-file model.md`, or from a pipe with `--body-file -`; a document does not belong on a command line, and `edit --body-file` replaces a body without reading the record file by hand.

### Plan and execute

Make each Task a reviewable result with constraints, behavior to preserve and completion evidence. For an epic, create a hub `--from` the idea, tag it `epic`, create children `--from` the hub, and `block <hub> <child>` for each deliverable. Add child dependencies only where one result is required by another. Read `ready --for <hub>` to choose work. A ready hub still needs verification of the overall outcome.

Keep one Task in flight per person by default. Start it before work; when switching subjects, log the handoff and hold unfinished work with a reason, or hand the Task over with `edit --taken-by`. Comment with the result, evidence and next step, using `--via`. Update models and specs when their meaning changes. Supersede a Decision when its ruling changes; edit it when clarifying the same ruling. A Decision that cites another as context declares the relationship once, on `add` or later with `edit`: `--link "within decision.x"` for a rule that is part of a wider one, `--link "departs-from decision.x"` for a drift. `may-conflict` then names only the pair nobody has judged; read those records before deciding whether a conflict exists.

Close answered Questions with `--resolved-by <decision-or-task>`, or `--reason` citing a Note when knowledge answers them, then archive. A genuine review date can be set on a drift Decision with `edit --review-by`; leave it unset when none is known.

### Verify and leave a continuation

Verify the promised result before closing. If review is required, `submit` and wait for acceptance. Otherwise close with `--note <report.md>` by default: state the result, evidence and limits. Archive the Task immediately; its report travels with it. Keep reusable knowledge live. Cancel work with `--reason`; hold work that awaits something, naming what will unblock it.

Run `anb check`, address findings and recheck. When no CLI repair exists, report the obstruction. Leave unfinished work's result and next action in its log, or in the idea for brief shaping. Commit the notebook with the code by default, within the user's sharing policy and commit authorization.

## Common mistakes

| Temptation | Use instead | Reason |
|---|---|---|
| Put the whole proposal in a Task to save time | Keep the idea and create work from it | Archiving one delivery must not hide the proposal |
| Record a plausible answer as a Decision | Keep a Question until the choice is settled | Future agents treat Decisions as governing knowledge |
| Use a guide as a one-off handoff | Put the next step in the Task log or idea | Guides describe repeatable procedures |
| Turn related subjects into blockers | Cite bare ids for context; block real prerequisites | Artificial dependencies hide work that can start |
| Quote an id intended as a relationship | Cite it outside backticks | Quoted examples do not create mentions |
| Retry a refused command unchanged | Read its `try:` instruction and fill its placeholders | Refusals explain the required correction |
| Repeat `add` after an uncertain result | Inspect the notebook first | Creation without an explicit id can produce duplicates |
| Start a Task marked as another person's | Pick a Task nobody took, or ask the user before `edit --taken-by` | Two people working one Task learn of it from a merge conflict |
| Start work straight from Status | Read `list --type decision --kind rule` first | Status is the work; a rule is read before the work it binds |

## References

Before an unfamiliar command, read `anb <verb> --help` or [commands](references/commands.md). Read [the worked session](references/session.md) for literal replies and [refusals](references/refusals.md) when recovery is unclear. Use `--json` for programmatic reads; `list` and `ready` rows carry `by` and `taken-by` there. Use the installed `anb-atlas` skill for a visual review.

For a named personal practice, `list --match <name> --global` finds it and `show <id> --global` reads it; guides tagged `skill` are reusable practices. Global scope holds Decisions and Notes, while Tasks and Questions stay in the project. Cite a global rule's id when a project Decision departs from it so the relationship remains visible.
"#;

fn reference(name: &str, description: &str, section: impl FnOnce(&mut String)) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "---\nname: {name}\ndescription: {description}");
    out.push_str("metadata:\n  managed-by: anb\n---\n\n");
    let _ = writeln!(out, "# {name}\n");
    out.push_str("This reference is generated by `anb skill`. Command descriptions come from the CLI definitions; examples run against a scratch notebook. CI checks the committed copy for drift.\n\n");
    let mut body = String::new();
    section(&mut body);
    out.push_str("## Contents\n\n");
    for heading in headings(&body) {
        let _ = writeln!(out, "- {heading}");
    }
    out.push('\n');
    out.push_str(&body);
    out
}

/// The headings of `body` at its shallowest level: the sections a reader
/// jumps to, listed before the text so a partial read still shows the scope.
fn headings(body: &str) -> Vec<&str> {
    let marked: Vec<(usize, &str)> = body
        .lines()
        .filter_map(|line| {
            let level = line.bytes().take_while(|byte| *byte == b'#').count();
            (level > 0 && line.as_bytes().get(level) == Some(&b' '))
                .then(|| (level, line[level + 1..].trim()))
        })
        .collect();
    let Some(shallowest) = marked.iter().map(|(level, _)| *level).min() else {
        return Vec::new();
    };
    marked
        .into_iter()
        .filter(|(level, _)| *level == shallowest)
        .map(|(_, text)| text)
        .collect()
}

/// Every verb with its flags, from the clap tree the binary parses with.
/// The narrowing flags are one table, since they mean the same on every
/// verb that takes them; a verb's section names the ones it takes.
fn commands_section(out: &mut String) {
    out.push_str("Global flags on every command: `--json` (compact JSON instead of text), `--notebook <PATH>` (where the notebook lives, outranking `ANB_NOTEBOOK`), `--global` (the user's notebook in the home directory; refused beside `--notebook`).\n\n");
    let cli = Cli::command();
    let verbs: Vec<&clap::Command> = cli
        .get_subcommands()
        .filter(|verb| verb.get_name() != "help")
        .collect();

    out.push_str("### Narrowing\n\n");
    out.push_str("A read answers with the records its narrowing admits. Each flag is a predicate over the same notebook, so two flags ask for the intersection, and a flag means the same on every verb that takes it; a verb's section names its own. Without `--by`, `--mine` or `--team`, the notebook's `scope` config key decides whose records a read answers with.\n\n");
    let mut named: Vec<String> = Vec::new();
    out.push_str("| Flag | Meaning |\n|---|---|\n");
    for flag in verbs.iter().flat_map(|verb| narrowing_flags(verb)) {
        if named.contains(&flag_spelling(flag)) {
            continue;
        }
        named.push(flag_spelling(flag));
        let _ = writeln!(out, "| `{}` | {} |", flag_spelling(flag), flag_help(flag));
    }
    out.push('\n');

    for verb in verbs {
        let _ = writeln!(out, "### anb {}", verb.get_name());
        if let Some(about) = verb.get_about() {
            let _ = writeln!(out, "\n{about}\n");
        }
        let positionals: Vec<String> = verb
            .get_positionals()
            .map(|arg| {
                let name = arg.get_id().as_str().to_uppercase();
                if arg.is_required_set() {
                    format!("<{name}>")
                } else {
                    format!("[{name}]")
                }
            })
            .collect();
        if !positionals.is_empty() {
            let _ = writeln!(out, "Arguments: `{}`\n", positionals.join(" "));
        }
        let mut flags = own_flags(verb).peekable();
        if flags.peek().is_some() {
            out.push_str("| Flag | Meaning |\n|---|---|\n");
            for flag in flags {
                let _ = writeln!(out, "| `{}` | {} |", flag_spelling(flag), flag_help(flag));
            }
            out.push('\n');
        }
        let narrowing: Vec<String> = narrowing_flags(verb)
            .map(|flag| format!("`--{}`", flag.get_long().unwrap_or_default()))
            .collect();
        if !narrowing.is_empty() {
            let _ = writeln!(out, "Narrowing: {}.\n", narrowing.join(", "));
        }
    }
}

/// The flags a verb declares for itself: not the global ones, not the
/// shared narrowing.
fn own_flags(verb: &clap::Command) -> impl Iterator<Item = &Arg> {
    verb.get_arguments()
        .filter(|arg| arg.get_long().is_some() && !arg.is_global_set())
        .filter(|arg| arg.get_id().as_str() != "help")
        .filter(|arg| arg.get_help_heading() != Some(NARROWING))
}

/// The narrowing flags a verb takes, under the heading that marks them.
fn narrowing_flags(verb: &clap::Command) -> impl Iterator<Item = &Arg> {
    verb.get_arguments()
        .filter(|arg| arg.get_help_heading() == Some(NARROWING))
}

fn flag_spelling(flag: &Arg) -> String {
    let long = flag.get_long().unwrap_or_default();
    if !flag.get_action().takes_values() {
        return format!("--{long}");
    }
    match flag.get_value_names() {
        Some(names) if !names.is_empty() => format!("--{long} <{}>", names[0].as_str()),
        _ => format!("--{long} <{}>", long.to_uppercase()),
    }
}

fn flag_help(flag: &Arg) -> String {
    flag.get_help()
        .map(|help| help.to_string().replace('\n', " "))
        .unwrap_or_default()
}

/// One notebook worked from empty to archived work, every reply rendered by
/// the tool. The steps are the examples; their order is the method.
fn session_section(out: &mut String) {
    out.push_str("Every reply below is what the tool printed, run on ");
    out.push_str(TODAY);
    out.push_str(
        " by an agent working as `Ada`, the identity the host resolved from `ANB_BY` or git.\n\n",
    );
    let mut notebook = Scratch::new();
    for step in SESSION {
        match step {
            Step::Head(text) => {
                let _ = writeln!(out, "## {text}\n");
            }
            Step::Say(text) => {
                let _ = writeln!(out, "{text}\n");
            }
            Step::Run(line) => notebook.example(out, line),
            Step::Report(path, body) => notebook.reports.push((path, body)),
        }
    }
}

/// The refusal catalog: one example per stable code, rendered by the tool.
fn refusals_section(out: &mut String) {
    out.push_str("Refusals begin with `error[<code>]: <message>`. When a next action is available, `try:` lines provide a command or an argument template to fill in. Codes are stable; messages describe the failed condition.\n\n");
    let mut notebook = Scratch::new();
    for step in REFUSALS {
        match step {
            Step::Head(text) => {
                let _ = writeln!(out, "## {text}\n");
            }
            Step::Say(text) => {
                let _ = writeln!(out, "{text}\n");
            }
            Step::Run(line) => notebook.example(out, line),
            Step::Report(path, body) => notebook.reports.push((path, body)),
        }
    }
    out.push_str("## Codes without an example\n\n");
    out.push_str("Other refusal codes describe command or storage failures: `unknown-command` (a verb anb does not have; the reply offers `anb --help`), `storage` (the file system failed the read or write, in the message) and `not-utf8` (a record file is not UTF-8; `anb check` names it).\n\n");
}

/// The check findings, code by code, with the weight each carries.
fn findings_section(out: &mut String) {
    out.push_str("## Check findings\n\n");
    out.push_str("`anb check` prints `findings[N]{file,line,severity,code,repair,message}`; an `error` fails the command, a `warning` does not. The `repair` column carries the command that erases the finding when the tool has one.\n\n");
    out.push_str("| Severity | Codes |\n|---|---|\n");
    for severity in [Severity::Error, Severity::Warning] {
        let codes: Vec<String> = FindingCode::ALL
            .iter()
            .filter(|code| code.severity() == severity)
            .map(|code| format!("`{}`", code.as_str()))
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} |",
            match severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
            codes.join(", ")
        );
    }
    out.push('\n');
}

enum Step {
    /// A section heading; the reference lists them at its top.
    Head(&'static str),
    /// Prose between examples.
    Say(&'static str),
    /// A command line, run and rendered.
    Run(&'static [&'static str]),
    /// A report file the next `close --note` reads.
    Report(&'static str, &'static str),
}

const SESSION: &[Step] = &[
    Step::Head("The hub and the work born inside it"),
    Step::Say("An idea becomes a hub, and the work inside it is born from the hub:"),
    Step::Run(&[
        "add",
        "task",
        "Ship the parser",
        "--tag",
        "epic",
        "--body",
        "The parser reads every record file byte for byte.",
    ]),
    Step::Run(&[
        "add",
        "task",
        "Grammar parser accepts fences",
        "--from",
        "task.ship-the-parser",
        "--priority",
        "1",
    ]),
    Step::Run(&[
        "add",
        "task",
        "Negative corpus wired into CI",
        "--from",
        "task.ship-the-parser",
        "--priority",
        "2",
    ]),
    Step::Run(&[
        "block",
        "task.ship-the-parser",
        "task.grammar-parser-accepts-fences",
    ]),
    Step::Run(&[
        "block",
        "task.ship-the-parser",
        "task.negative-corpus-wired-into-ci",
    ]),
    Step::Run(&[
        "block",
        "task.negative-corpus-wired-into-ci",
        "task.grammar-parser-accepts-fences",
    ]),
    Step::Head("The queue"),
    Step::Say(
        "The queue shows what can start now; the blocked child waits, and the `taken-by` column would name a Task someone already took:",
    ),
    Step::Run(&["ready"]),
    Step::Run(&["ready", "--for", "task.ship-the-parser"]),
    Step::Head("A session at work"),
    Step::Say(
        "A session takes the top of the queue, which records who took it as `taken-by`, logs as it goes with the entry signed `by/via`, and parks a doubt without widening its scope:",
    ),
    Step::Run(&["start", "task.grammar-parser-accepts-fences"]),
    Step::Run(&[
        "comment",
        "task.grammar-parser-accepts-fences",
        "fences parse; the indented-body case is next",
        "--via",
        "codex",
    ]),
    Step::Run(&[
        "add",
        "question",
        "Do fences nest?",
        "--from",
        "task.grammar-parser-accepts-fences",
    ]),
    Step::Head("Decisions and Notes"),
    Step::Say(
        "A ruling is a Decision; a term is a Note, here recorded by a colleague. A second Decision on the same ground is nudged about the first, so read it before going on:",
    ),
    Step::Run(&[
        "add",
        "decision",
        "Fences never nest",
        "--kind",
        "rule",
        "--tag",
        "parser",
        "--tag",
        "grammar",
        "--body",
        "A fence closes at the first closing marker. Answers question.do-fences-nest.",
    ]),
    Step::Run(&[
        "add",
        "decision",
        "A fence body is opaque",
        "--kind",
        "rule",
        "--tag",
        "parser",
        "--tag",
        "grammar",
        "--body",
        "Nothing inside a fence is parsed.",
    ]),
    Step::Run(&[
        "add",
        "note",
        "Fence",
        "--kind",
        "term",
        "--by",
        "Grace",
        "--body",
        "A fence is a pair of triple-backtick lines; the parser treats the lines between as one opaque body.",
    ]),
    Step::Say(
        "The standing rules are one listing, read before the work they bind; the same flags narrow `ready` and `graph`:",
    ),
    Step::Run(&["list", "--type", "decision", "--kind", "rule"]),
    Step::Head("Status"),
    Step::Say(
        "Status opens the session with the work: the active Task and its last log line, the queue and the open Questions; an active Task another person took would carry their name after its title:",
    ),
    Step::Run(&["status", "--budget", "0"]),
    Step::Say(
        "Narrowed to the user's own, with `--mine` or the config key `scope: mine`, it says whose it is and how to widen it:",
    ),
    Step::Run(&["status", "--mine", "--budget", "0"]),
    Step::Head("Closing a Question"),
    Step::Say("The doubt closes into the record that settled it, and is archived right after:"),
    Step::Run(&[
        "close",
        "question.do-fences-nest",
        "--resolved-by",
        "decision.fences-never-nest",
    ]),
    Step::Run(&["archive", "question.do-fences-nest"]),
    Step::Head("Closing a Task"),
    Step::Say(
        "The work closes with its report as a Note, and is archived right after; the reply names what the close unblocked:",
    ),
    Step::Report(
        "report.md",
        "# Fences\n\nThe parser accepts fenced bodies; the indented case is covered by the corpus.\n",
    ),
    Step::Run(&[
        "close",
        "task.grammar-parser-accepts-fences",
        "--note",
        "report.md",
    ]),
    Step::Run(&["archive", "task.grammar-parser-accepts-fences"]),
    Step::Head("Ending without work, pausing"),
    Step::Say(
        "A Task overtaken before it started ends with its reason, from open, and is archived like any closed record; a pause carries its reason too:",
    ),
    Step::Run(&["add", "task", "Port the parser to Go"]),
    Step::Run(&[
        "close",
        "task.port-the-parser-to-go",
        "--reason",
        "overtaken by decision.a-fence-body-is-opaque",
    ]),
    Step::Run(&["archive", "task.port-the-parser-to-go"]),
    Step::Run(&[
        "hold",
        "task.negative-corpus-wired-into-ci",
        "--reason",
        "waits for the CI runner",
        "--until",
        "2026-01-20",
    ]),
    Step::Head("Reading back, and the gate"),
    Step::Say(
        "Reading back: one record, the whole notebook, the records that are Ada's own, the records holding a text with the archive reached, and the gate, which is clean because every settled record was archived as it settled:",
    ),
    Step::Run(&["show", "task.ship-the-parser"]),
    Step::Run(&["list"]),
    Step::Run(&["list", "--mine"]),
    Step::Run(&["list", "--match", "fence", "--archive"]),
    Step::Run(&["check"]),
];

const REFUSALS: &[Step] = &[
    Step::Head("The scratch notebook"),
    Step::Say("The scratch notebook the refusals below run against:"),
    Step::Run(&["add", "task", "Ship the parser", "--tag", "epic"]),
    Step::Run(&[
        "add",
        "task",
        "Grammar parser accepts fences",
        "--from",
        "task.ship-the-parser",
    ]),
    Step::Run(&["add", "decision", "Fences never nest", "--kind", "rule"]),
    Step::Run(&["add", "note", "Fence", "--kind", "term"]),
    Step::Head("unknown-id"),
    Step::Say("No record carries the id."),
    Step::Run(&["start", "task.parser"]),
    Step::Head("invalid-transition"),
    Step::Say(
        "The record's state does not allow the move; the valid moves are listed, each with its command.",
    ),
    Step::Run(&["close", "task.ship-the-parser", "--no-proof"]),
    Step::Head("taken"),
    Step::Say(
        "Another person took the Task. `start` takes work, so a Task already taken changes hands through `edit --taken-by` first, on purpose.",
    ),
    Step::Run(&[
        "edit",
        "task.grammar-parser-accepts-fences",
        "--taken-by",
        "Grace",
    ]),
    Step::Run(&["start", "task.grammar-parser-accepts-fences"]),
    Step::Head("invalid-argument"),
    Step::Say(
        "A flag or value is invalid for this command or record type. The reply suggests a valid form.",
    ),
    Step::Run(&["add", "decision", "Tabs", "--priority", "2"]),
    Step::Run(&["close", "task.ship-the-parser"]),
    Step::Head("dangling-ref"),
    Step::Say("An envelope reference names a record that does not exist."),
    Step::Run(&["add", "task", "A child", "--from", "task.ghost"]),
    Step::Head("would-cycle"),
    Step::Say("The proposed dependency would create a cycle. The reply prints the cycle."),
    Step::Run(&[
        "block",
        "task.ship-the-parser",
        "task.grammar-parser-accepts-fences",
    ]),
    Step::Run(&[
        "block",
        "task.grammar-parser-accepts-fences",
        "task.ship-the-parser",
    ]),
    Step::Head("duplicate-id"),
    Step::Say(
        "An existing record reserves its id, including in the archive. Only deleting a record frees its id.",
    ),
    Step::Run(&[
        "add",
        "task",
        "Ship the parser again",
        "--id",
        "task.ship-the-parser",
    ]),
    Step::Head("wrong-type"),
    Step::Say("The id names a type the command does not act on."),
    Step::Run(&["comment", "note.fence", "a line"]),
    Step::Head("still-referenced"),
    Step::Say(
        "Other records reference this id. The reply identifies the references that prevent deletion.",
    ),
    Step::Run(&["delete", "task.ship-the-parser"]),
    Step::Head("cannot-supersede"),
    Step::Say(
        "The target is not eligible for supersession. Only an active Decision or Note can be replaced.",
    ),
    Step::Run(&["retire", "decision.fences-never-nest"]),
    Step::Run(&[
        "add",
        "decision",
        "Fences nest once",
        "--kind",
        "rule",
        "--supersedes",
        "decision.fences-never-nest",
    ]),
    Step::Head("archived"),
    Step::Say("An archived record is read, never mutated in place; `restore` brings it back."),
    Step::Run(&["archive", "decision.fences-never-nest"]),
    Step::Run(&["edit", "decision.fences-never-nest", "--title", "Fences"]),
    Step::Head("invalid-record"),
    Step::Say(
        "The operation requires a valid record. Read the findings with `check` and follow the suggested repair before retrying.",
    ),
    Step::Run(&["add", "task", "Broken", "--id", "task.broken"]),
    Step::Run(&["start", "task.broken"]),
];

/// The scratch notebook the examples run on, with the report files a
/// `--note` may read.
struct Scratch {
    storage: MemoryStorage,
    reports: Vec<(&'static str, &'static str)>,
}

impl Scratch {
    fn new() -> Self {
        Scratch {
            storage: MemoryStorage::new(),
            reports: Vec::new(),
        }
    }

    /// Run `line` and append the example: the command, then the reply as
    /// the tool prints it, refusals included.
    fn example(&mut self, out: &mut String, line: &[&str]) {
        let mut args = vec!["anb"];
        args.extend_from_slice(line);
        let cli = Cli::try_parse_from(&args).expect("an example is a well-formed command line");
        let subject = subject(&cli.command);
        let reports = &self.reports;
        let read_file = |path: &str| -> Result<String, StorageError> {
            reports
                .iter()
                .find(|(named, _)| *named == path)
                .map(|(_, body)| (*body).to_owned())
                .ok_or_else(|| StorageError::NotFound {
                    path: path.to_owned(),
                })
        };
        let no_lost = |_: &[anb_core::CitedProof]| Vec::new();
        let host = Host {
            identity: || Some(AUTHOR.to_owned()),
            read_file: &read_file,
            lost_proofs: &no_lost,
            user_notebook: None,
            project_dir: Path::new("."),
            today: TODAY,
        };
        if matches!(cli.command, Command::Add { .. })
            && line.get(1) == Some(&"task")
            && line.get(2) == Some(&"Broken")
        {
            // The one example no command can produce: a file corrupted by
            // hand, so `invalid-record` has something to show.
            let _ = execute(cli.command, &mut self.storage, host);
            self.storage
                .write(
                    "tasks/task.broken.md",
                    "---\nid: task.broken\ntype: task\nstate: bogus\ntitle: Broken\ncreated: 2026-01-15\n---\n",
                )
                .expect("the scratch notebook is writable");
            return;
        }
        let rendered = match execute(cli.command, &mut self.storage, host) {
            Ok(reply) => text::render(&reply, TODAY),
            Err(error) => text::render_error(&error, &subject),
        };
        let _ = writeln!(out, "```\n$ anb {}\n{}```\n", shell_words(line), rendered);
    }
}

/// The command line as a reader types it: a word with a space is quoted.
fn shell_words(line: &[&str]) -> String {
    line.iter()
        .map(|word| {
            if word.contains(' ') || word.is_empty() {
                format!("\"{word}\"")
            } else {
                (*word).to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rendering_is_deterministic() {
        assert_eq!(render(), render());
    }

    #[test]
    fn a_reference_opens_with_the_list_of_its_sections() {
        let skill = render();
        for (file, text) in &skill.files()[1..] {
            let (_, after) = text
                .split_once("## Contents\n\n")
                .unwrap_or_else(|| panic!("{file} has no contents list"));
            let listed: Vec<&str> = after
                .lines()
                .map_while(|line| line.strip_prefix("- "))
                .collect();
            let body = &after[listed.iter().map(|item| item.len() + 3).sum::<usize>()..];
            let marks = |line: &str| line.bytes().take_while(|byte| *byte == b'#').count();
            let shallowest = body
                .lines()
                .filter(|line| line.starts_with('#'))
                .map(marks)
                .min()
                .unwrap_or_else(|| panic!("{file} has no sections"));
            let sections: Vec<&str> = body
                .lines()
                .filter(|line| {
                    marks(line) == shallowest && line.as_bytes().get(shallowest) == Some(&b' ')
                })
                .map(|line| line[shallowest + 1..].trim())
                .collect();
            assert_eq!(listed, sections, "{file} lists other sections than it has");
        }
    }

    #[test]
    fn every_verb_but_help_has_a_commands_entry() {
        let commands = render().commands;
        for verb in Cli::command().get_subcommands() {
            if verb.get_name() == "help" {
                continue;
            }
            assert!(
                commands.contains(&format!("### anb {}\n", verb.get_name())),
                "{} is missing from the commands reference",
                verb.get_name()
            );
        }
    }

    #[test]
    fn every_example_ran_and_no_example_broke_by_accident() {
        let Skill {
            session, refusals, ..
        } = render();
        // A worked-session step that failed would render as a refusal in
        // the session; the refusals belong to their own file.
        assert!(
            !session.contains("error["),
            "a session step was refused:\n{session}"
        );
        assert!(refusals.contains("error[unknown-id]"), "{refusals}");
    }
}
