//! The skill: what an agent that has never seen this repository needs to
//! work the notebook, generated from the binary itself so it cannot drift.
//!
//! `SKILL.md` carries the method — the session, the Task loop, the shape of
//! work, knowledge, the user's notebook, the reply contract — in the words
//! an agent reads once and keeps. Three references under `references/` carry
//! the depth, one file per need so a session loads only the one it has: every
//! command with its flags from the same definitions `--help` prints, a
//! worked session with every reply rendered by running the command, and the
//! refusal catalog rendered the same way. A committed copy is diffed against
//! this rendering in CI. The atlas skill, written by hand, lives in
//! [`atlas`] and is installed by `setup` beside this one.

use crate::cli::{Cli, Command};
use crate::recovery::subject;
use crate::reply::{Host, execute};
use crate::text;
use anb_core::{FindingCode, MemoryStorage, Severity, Storage as _, StorageError};
use clap::{Arg, CommandFactory as _, Parser as _};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

pub mod atlas;

/// The frontmatter line by which `setup` knows a skill file as its own. A
/// user who deletes the line owns the file from then on: setup rewrites only
/// what still carries it.
pub const MANAGED_MARK: &str = "managed-by: anb";

pub const SKILL_FILE: &str = "SKILL.md";
pub const COMMANDS_FILE: &str = "references/commands.md";
pub const SESSION_FILE: &str = "references/session.md";
pub const REFUSALS_FILE: &str = "references/refusals.md";

/// The day every example runs on, so the rendering is the same on every
/// machine and the ages in it read as written.
const TODAY: &str = "2026-01-15";
const AUTHOR: &str = "Ada";

/// The skill, rendered: the method and its three references.
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

/// A skill `setup` installs: its directory name under a host's `skills/`
/// and its files, paths relative to that directory.
pub struct Installable {
    pub name: &'static str,
    pub files: Vec<(&'static str, String)>,
}

/// Both skills the binary carries: the one rendered from itself and the
/// atlas, written by hand.
#[must_use]
pub fn installable() -> [Installable; 2] {
    let rendered = render();
    [
        Installable {
            name: "anb",
            files: rendered
                .files()
                .iter()
                .map(|(file, text)| (*file, (*text).to_owned()))
                .collect(),
        },
        Installable {
            name: atlas::NAME,
            files: atlas::files()
                .iter()
                .map(|(file, text)| (*file, (*text).to_owned()))
                .collect(),
        },
    ]
}

/// Which committed file differs from the rendering.
#[derive(Debug, PartialEq, Eq)]
pub struct Drift {
    pub file: &'static str,
    pub reason: &'static str,
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
            "Every refusal code with its cause and the try: line that repairs it, and the findings anb check raises. Open when a refusal's try: line is not enough.",
            |out| {
                refusals_section(out);
                findings_section(out);
            },
        ),
    }
}

/// # Errors
/// The storage failure of the write.
pub fn write_into(dir: &Path, skill: &Skill) -> Result<(), StorageError> {
    for (file, text) in skill.files() {
        let path = dir.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| io_failure(parent, &error))?;
        }
        fs::write(&path, text).map_err(|error| io_failure(&path, &error))?;
    }
    Ok(())
}

/// # Errors
/// The storage failure of a read; a missing file is drift, not a failure.
pub fn drift(dir: &Path, skill: &Skill) -> Result<Vec<Drift>, StorageError> {
    let mut drifted = Vec::new();
    for (file, expected) in skill.files() {
        let path = dir.join(file);
        match fs::read_to_string(&path) {
            Ok(text) if text == expected => {}
            Ok(_) => drifted.push(Drift {
                file,
                reason: "differs from the rendering",
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => drifted.push(Drift {
                file,
                reason: "missing",
            }),
            Err(error) => return Err(io_failure(&path, &error)),
        }
    }
    Ok(drifted)
}

/// Whether a skill file on disk is one setup wrote and may rewrite.
#[must_use]
pub fn is_managed(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("---\n") else {
        return false;
    };
    let Some(end) = rest.find("\n---\n") else {
        return false;
    };
    rest[..end].lines().any(|line| line.trim() == MANAGED_MARK)
}

fn io_failure(path: &Path, error: &std::io::Error) -> StorageError {
    StorageError::Io {
        path: path.display().to_string(),
        detail: error.to_string(),
    }
}

// ---------------------------------------------------------------------------
// SKILL.md — the method
// ---------------------------------------------------------------------------

fn skill_md() -> String {
    let mut out = String::new();
    out.push_str("---\nname: anb\n");
    out.push_str("description: Use when working in a repository that has an .agent-notebook directory: when asked to continue a task or an epic, to pick the next piece of work, to record a decision, a doubt or a finding, to close work with its proof, or to say where the project stands. Also when a session starts and a status line beginning with active: was printed.\n");
    out.push_str("metadata:\n  managed-by: anb\n---\n\n");
    out.push_str(SKILL_BODY);
    out
}

const SKILL_BODY: &str = r#"# anb

## Overview

`anb` keeps a project's working memory as typed records in `.agent-notebook/`: Tasks (open → active → review → closed), Decisions (active → superseded | retired), Notes (active → retired), Questions (open → closed). Every record has an id you type, such as `task.parser-fences` or `decision.no-mise-toml`, and a markdown file a human can read. Change the notebook only through `anb`, never by editing the files. The notebook is the developer's memory of the project, and keeping it clean is your job.

Every reply is short plain text an agent parses at a glance: `ok: <verb> <id> — <what changed>`, tables as `name[N]{fields}:` with comma rows, and refusals as `error[<code>]: <message>` followed by `try:` lines. A `try:` line is a command that runs as printed, so run one instead of guessing. Listings are bounded; `--all` lifts the bound, and `--json` gives the same data as compact JSON.

## When to use

- The session starts in a repository with `.agent-notebook/`, or a hook printed a line beginning with `active:`.
- Someone asks to continue, resume or pick up a task or an epic, or to take the next piece of work.
- You are about to record a decision, a doubt, a term, a fact or a finding. The notebook is where it goes, not a chat message or a stray file.
- Work is done and needs closing with its proof, or the developer asks where the project stands.

A repository without a notebook is outside this skill until `anb setup` wires one and `anb add` creates the first record. Editing record files is outside it always: every change is a verb.

## Quick reference

| Situation | Command |
|---|---|
| Where did the last session stop? | `anb status` |
| What can start now? | `anb ready`, or inside one epic `anb ready --for <hub>` |
| Take a Task into work | `anb start <id>` |
| Log progress | `anb comment <id> "<one line>"` |
| A doubt while working | `anb add question "<title>" --from <task>` |
| A ruling | `anb add decision "<title>" --kind rule` (or `shape`, `drift`) |
| A fact, a term, a guide | `anb add note "<title>" --kind fact` (or `term`, `guide`) |
| Hand work to a human | `anb submit <id>` |
| Close with proof, then file | `anb close <id> --note <report.md>`, then `anb archive <id>` |
| A Question settled | `anb close <question> --resolved-by <id>`, or `--reason "<why>"` |
| Pause, resume | `anb hold <id> --reason "<why>"`, `anb unhold <id>` |
| Find a record | `anb search <words>`, then `anb show <id>` |
| Verify the notebook | `anb check` |

Three references sit under `references/`, one per need, so open only the one you have. Before a verb you have not used, open [commands](references/commands.md): every verb with its flags and what each means. To see what a reply looks like before you parse one, open the [worked session](references/session.md): a notebook from empty to archived work, every reply as printed. When a refusal's `try:` line is not enough, open [refusals](references/refusals.md): every error code with its cause and repair, and the findings `anb check` raises.

## The session

1. Run `anb status`. The `active` line is where the last session stopped, with its last log entry. If a hook already printed it, do not print it again.
2. Resume the active Task. If none is active, take the top of `anb ready` and run `anb start <id>`.
3. Asked to continue an epic: find the hub with `anb search <words>`, which looks through ids, titles and tags. Then take the active Task inside it, or else the top of `anb ready --for <hub>`, and start it.

## The Task loop

- `anb start <id>` takes a Task into work, and takes it back from review.
- `anb comment <id> "<one line>"` as you go: where you stopped, what you decided, what you found. The next session resumes from the log rather than from scratch.
- Every friction met on the way becomes a record instead of a workaround: a doubt is `anb add question "<title>" --from <task>`, a ruling is `anb add decision`, a fact is `anb add note`.
- `anb submit <id>` hands the work to a human when one accepts it; the human's `anb start <id>` takes it back.
- `anb close <id> --note <report.md>` closes with the report ingested as a Note. That is the default proof because it travels with the notebook. The others are its equals: `--pr <url>`, `--sha <sha>`, `--report <path>` for a living document left where it lies, and `--no-proof` when the work is done and there is nothing to show. Work that will never happen ends with `anb close <id> --reason "<why>"`, which is legal from open.
- Run `anb archive <id>` right after the close. A closed record left live is a leftover somebody else has to find.

Hygiene is your job, not the developer's. Close Questions the moment they are settled: `anb close <question> --resolved-by <decision-or-task>` when the answer became a record, `--reason "<why>"` when it did not. Pause with `anb hold <id> --reason "<why>"` and resume with `unhold`; a hold without a reason is where work rots. Leave nothing for the developer to tidy.

## The shape of work

Work is almost never a flat sheet. An idea gets a hub Task tagged `epic`, and every Task born inside it is created `--from <hub>`. The hub is blocked by each child (`anb block <hub> <child>`), so it is not ready until the last child closes; then close and archive the hub like any Task. `anb ready --for <hub>` and `anb list --for <hub>` see one epic's work, and `anb status` shows every epic's progress and its next Task. Create children this way by default: origin is written at `add` time and cannot be added later.

## Knowledge

- Decisions carry a kind: `rule` for how we work, `shape` for a design ruling, `drift` for a deviation that stands until superseded. Replace one with `anb add decision "<title>" --supersedes <old>`; end one without a successor with `anb retire <id>`. The reply names a live Decision yours may conflict with. Read it before going on.
- Notes are curated knowledge corrected in place with `anb edit`; their kinds are `fact`, `term` and `guide`.
- In prose, a bare id is a reference and a backticked one is a quotation. Cite records by id in bodies and comments. A citation of an id that exists nowhere comes back as `dangling-mention`: a forward reference is legal, a typo is not.

## The user's own notebook

`--global` names the user's notebook in the home directory, for knowledge that outlives one repository. Decisions and Notes live there; Tasks and Questions are refused, because work stays in the project. A practice is a global Note tagged `skill`, addressed by name with `anb search <name> --global` and then `anb show <id> --global`, so "use my skill X" resolves through anb rather than through a pasted path. A project rule that stands against one of the user's cites the user's id in its body, and Status then names the pair.

## Before you stop

`anb check` must be green. It verifies every file and names the move that repairs each finding. Commit the notebook with the code it describes.

## Common mistakes

Each of these is easy because of how the tool behaves, and each has a cost the next session pays.

| Mistake | What it costs | Instead |
|---|---|---|
| A child Task added without `--from`, and no `block` edge from the hub | The epic never sees it: `ready --for <hub>` and `list --for <hub>` follow the hub's scope, and the hub's `closed/total` does not count it | `anb add task "<title>" --from <hub>` at creation; later `anb edit <id> --from <hub>`, and `anb block <hub> <id>` when the hub waits on it |
| A backticked id where a reference was meant, or a bare id as an example | A backticked id is a quotation the mention scan skips, so a typo in it is never caught and `show` lists no relation; a bare example id becomes a `dangling-mention` Debt line that stays until the record exists | Bare ids for references, backticks for quotations |
| A Question answered in a comment on the Task | The Question stays open; when the Task closes, Status raises `origin-closed`, and the answer sits in a log nobody rereads | `anb close <question> --resolved-by <id>` when the answer became a record, `--reason "<why>"` when it did not |
| A ruling changed with `anb edit --body` | The Decision's text says one thing and its history another; nobody is told the rule changed, and `may-conflict` cannot warn about a body that was rewritten in place | `anb edit` for a correction of the same ruling; a different ruling is `anb add decision "<title>" --supersedes <old>` |
| A collapsed Status section read as empty | Under budget, a section keeps its count and drops its rows (`ready: 7` with the command that lists them); the queue is not empty, the budget was | `anb status --budget 0` shows every row; `anb ready` is the queue itself |
| A held Task read as gone | A hold removes the Task from `active:` and from `ready`; it waits in `held[N]` with its reason. A session that reads only the first lines starts a second Task on the same work | Read `held` before starting anything; `anb unhold <id>` when the reason has lifted |
| `--no-proof` because the proof is somewhere else | The record says forever that there was nothing to show, while a pull request, a commit or a report existed | `--pr <url>`, `--sha <sha>`, `--note <report.md>`; `--no-proof` only when there is nothing |
| A record recreated because it was not in `list` | `list` and `ready` show the working set; the settled record lives in the archive with its id, so the new one is minted as `<id>-00` beside it and the notebook holds the same thing twice (`duplicate-id` fires only when `--id` names the taken id) | `anb search <words>` reaches the archive; `anb restore <id>` brings a record back |
"#;

// ---------------------------------------------------------------------------
// The references — the depth, one file per need
// ---------------------------------------------------------------------------

/// A reference file: frontmatter carrying the generated mark, a title, and
/// the body `section` renders.
fn reference(name: &str, description: &str, section: impl FnOnce(&mut String)) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "---\nname: {name}\ndescription: {description}");
    out.push_str("metadata:\n  managed-by: anb\n---\n\n");
    let _ = writeln!(out, "# {name}\n");
    out.push_str("Generated from the binary: the same definitions `anb --help` prints, and every example run on a scratch notebook. A committed copy is checked against this rendering in CI.\n\n");
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
fn commands_section(out: &mut String) {
    out.push_str("Global flags on every command: `--json` (compact JSON instead of text), `--notebook <PATH>` (where the notebook lives, outranking `ANB_NOTEBOOK`), `--global` (the user's notebook in the home directory; refused beside `--notebook`).\n\n");
    let cli = Cli::command();
    for verb in cli
        .get_subcommands()
        .filter(|verb| verb.get_name() != "help")
    {
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
        let mut flags = verb
            .get_arguments()
            .filter(|arg| arg.get_long().is_some() && !arg.is_global_set())
            .filter(|arg| arg.get_id().as_str() != "help")
            .peekable();
        if flags.peek().is_some() {
            out.push_str("| Flag | Meaning |\n|---|---|\n");
            for flag in flags {
                let _ = writeln!(out, "| `{}` | {} |", flag_spelling(flag), flag_help(flag));
            }
            out.push('\n');
        }
    }
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
    out.push_str(" by an agent whose git identity is `Ada`.\n\n");
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
    out.push_str("Every refusal is `error[<code>]: <message>` followed by `try:` lines, which are commands that run as printed. The codes are stable; the messages name the record and the fact.\n\n");
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
    out.push_str("Three more codes reach the command line without a notebook to show them on: `unknown-command` (a verb anb does not have; the reply offers `anb --help`), `storage` (the file system failed the read or write, in the message) and `not-utf8` (a record file is not UTF-8; `anb check` names it).\n\n");
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
    Step::Say("The queue shows what can start now; the blocked child waits:"),
    Step::Run(&["ready"]),
    Step::Run(&["ready", "--for", "task.ship-the-parser"]),
    Step::Head("A session at work"),
    Step::Say(
        "A session takes the top of the queue, logs as it goes, and parks a doubt without widening its scope:",
    ),
    Step::Run(&["start", "task.grammar-parser-accepts-fences"]),
    Step::Run(&[
        "comment",
        "task.grammar-parser-accepts-fences",
        "fences parse; the indented-body case is next",
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
        "A ruling is a Decision; a term is a Note. A second Decision on the same ground is nudged about the first, so read it before going on:",
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
        "--body",
        "A fence is a pair of triple-backtick lines; the parser treats the lines between as one opaque body.",
    ]),
    Step::Head("Closing a Question"),
    Step::Say("The doubt closes into the record that settled it, and is archived right after:"),
    Step::Run(&[
        "close",
        "question.do-fences-nest",
        "--resolved-by",
        "decision.fences-never-nest",
    ]),
    Step::Run(&["archive", "question.do-fences-nest"]),
    Step::Head("Status"),
    Step::Say(
        "Status opens the session with the active Task and its last log line, the rules, the queue and the epics:",
    ),
    Step::Run(&["status", "--budget", "0"]),
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
        "Reading back: one record, the whole notebook, a search that reaches the archive, and the gate, which is clean because every settled record was archived as it settled:",
    ),
    Step::Run(&["show", "task.ship-the-parser"]),
    Step::Run(&["list"]),
    Step::Run(&["search", "fence"]),
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
    Step::Head("invalid-argument"),
    Step::Say("A flag or value is malformed or foreign to the record; the retry shape is given."),
    Step::Run(&["add", "decision", "Tabs", "--priority", "2"]),
    Step::Run(&["close", "task.ship-the-parser"]),
    Step::Head("dangling-ref"),
    Step::Say("An envelope reference names a record that does not exist."),
    Step::Run(&["add", "task", "A child", "--from", "task.ghost"]),
    Step::Head("would-cycle"),
    Step::Say("The edge would close a dependency cycle, which is walked in full."),
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
    Step::Say("Ids are never reused, the archive included."),
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
    Step::Say("A delete would leave the notebook pointing at nothing; every holder is named."),
    Step::Run(&["delete", "task.ship-the-parser"]),
    Step::Head("cannot-supersede"),
    Step::Say("The record named by `--supersedes` cannot die by supersession."),
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
        "The file carries error findings, which close it to every verb until `check` is answered.",
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
        let read_report = |path: &str| -> Result<String, StorageError> {
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
            git_by: || Some(AUTHOR.to_owned()),
            read_report: &read_report,
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
    fn a_generated_file_is_known_by_its_frontmatter_mark() {
        let skill = render();
        for (file, text) in skill.files() {
            assert!(is_managed(text), "{file} lacks the mark");
        }
        assert!(!is_managed(&skill.skill.replace("  managed-by: anb\n", "")));
        assert!(!is_managed("# No frontmatter at all\n"));
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
    fn setup_installs_the_rendered_skill_and_the_atlas() {
        let [anb, atlas] = installable();
        assert_eq!(anb.name, "anb");
        let rendered = render();
        assert_eq!(
            anb.files,
            rendered
                .files()
                .iter()
                .map(|(file, text)| (*file, (*text).to_owned()))
                .collect::<Vec<_>>()
        );
        assert_eq!(atlas.name, "anb-atlas");
        assert_eq!(
            atlas.files,
            atlas::files()
                .iter()
                .map(|(file, text)| (*file, (*text).to_owned()))
                .collect::<Vec<_>>()
        );
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
