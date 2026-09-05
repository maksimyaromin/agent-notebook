//! The skill: what an agent that has never seen this repository needs to
//! work the notebook, generated from the binary itself so it cannot drift.
//!
//! `SKILL.md` carries the method — the session, the Task loop, the shape of
//! work, knowledge, the user's notebook, the reply contract — in the words
//! an agent reads once and keeps. Three references beside it carry the
//! depth, one file per need so a session loads only the one it has: every
//! command with its flags from the same definitions `--help` prints, a
//! worked session with every reply rendered by running the command, and the
//! refusal catalog rendered the same way. A committed copy is diffed against
//! this rendering in CI.

use crate::cli::{Cli, Command};
use crate::recovery::subject;
use crate::reply::{Host, execute};
use crate::text;
use anb_core::{FindingCode, MemoryStorage, Severity, Storage as _, StorageError};
use clap::{Arg, CommandFactory as _, Parser as _};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// The frontmatter key by which `setup` knows a skill file as its own. A
/// user who deletes the line owns the file from then on: setup rewrites only
/// what still carries it.
pub const GENERATED_MARK: &str = "generated: anb";

pub const SKILL_FILE: &str = "SKILL.md";
pub const COMMANDS_FILE: &str = "commands.md";
pub const SESSION_FILE: &str = "session.md";
pub const REFUSALS_FILE: &str = "refusals.md";

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
    fs::create_dir_all(dir).map_err(|error| io_failure(dir, &error))?;
    for (file, text) in skill.files() {
        let path = dir.join(file);
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
pub fn is_generated(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("---\n") else {
        return false;
    };
    let Some(end) = rest.find("\n---\n") else {
        return false;
    };
    rest[..end]
        .lines()
        .any(|line| line.trim() == GENERATED_MARK)
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
    out.push_str("description: Work a project's notebook — its Tasks, Decisions, Notes and Questions — through the anb CLI: resume the active Task, take the next ready one, record a decision or a question, close work with its proof, and leave the notebook clean. Use whenever the session touches .agent-notebook, when asked to continue a task or an epic, and before recording a decision, a doubt or a finding.\n");
    out.push_str("metadata:\n  generated: anb\n---\n\n");
    out.push_str(SKILL_BODY);
    out
}

const SKILL_BODY: &str = r#"# The notebook

`anb` keeps a project's working memory as typed records in `.agent-notebook/`: **Tasks** (open → active → review → closed), **Decisions** (active → superseded | retired), **Notes** (active → retired), **Questions** (open → closed). Every record has an id you type — `task.parser-fences`, `decision.no-mise-toml` — and a markdown file a human can read. Mutate the notebook only through `anb`; never edit the files.

Every reply is short plain text an agent parses at a glance: `ok: <verb> <id> — <what changed>`, tables as `name[N]{fields}:` with comma rows, and refusals as `error[<code>]: <message>` followed by `try:` lines. **A `try:` line is a command that runs as printed** — run one instead of guessing. Listings are bounded; `--all` lifts the bound, `--json` gives the same data as compact JSON. Three references sit beside this file, one per need, so open only the one you have: before a verb you have not used, [commands](commands.md) — every verb with its flags and what each means; to see what a reply looks like before you parse one, the [worked session](session.md) — a notebook from empty to archived work, every reply as printed; when a refusal's `try:` line is not enough, [refusals](refusals.md) — every error code with its cause and repair, and the findings `anb check` raises.

## The session

1. `anb status`. The **active** line is where the last session stopped, with its last log entry. If a hook already printed it, do not print it again.
2. Resume the active Task. If none is active, take the top of `anb ready` and `anb start <id>`.
3. Asked to *continue an epic*: find the hub — `anb search <words>` over ids, titles and tags — then take the active Task inside it, else the top of `anb ready --for <hub>`, and start it.

## The Task loop

- `anb start <id>` takes a Task into work (also back from review).
- `anb comment <id> "<one line>"` as you go: where you stopped, what you decided, what you found. The next session resumes from the log, not from scratch.
- Every friction met on the way is a record, never a workaround: a doubt is `anb add question "<title>" --from <task>`, a ruling is `anb add decision`, a fact is `anb add note`.
- `anb submit <id>` hands the work to a human when one accepts it; the human's `anb start <id>` takes it back.
- `anb close <id> --note <report.md>` closes with the report ingested as a Note — the default proof, because it travels with the notebook. Equals: `--pr <url>`, `--sha <sha>`, `--report <path>` (a living document left where it lies), `--no-proof` (done, nothing to show). Work that will never happen ends with `anb close <id> --reason "<why>"`, legal from open.
- Run `anb archive <id>` right after the close. A closed record left live is a leftover somebody else has to find.

**Hygiene is your job, not the developer's.** Close Questions the moment they are settled — `anb close <question> --resolved-by <decision-or-task>` when the answer became a record, `--reason "<why>"` when it did not. Pause with `anb hold <id> --reason "<why>"` and resume with `unhold`; a hold without a reason is where work rots. Leave nothing for the developer to tidy.

## The shape of work

Work is almost never a flat sheet. An idea gets a **hub** Task tagged `epic`; every Task born inside it is created `--from <hub>`, and the hub is blocked by each child — `anb block <hub> <child>` — so it is not ready until the last child closes; then close and archive the hub like any Task. `anb ready --for <hub>` and `anb list --for <hub>` see one epic's work; `anb status` shows every epic's progress and its next Task. Create children this way by default, not as an option: origin is written at `add` time.

## Knowledge

- **Decisions** carry a kind: `rule` (how we work), `shape` (a design ruling), `drift` (a deviation that stands until superseded). Replace one with `anb add decision "<title>" --supersedes <old>`; end one without a successor with `anb retire <id>`. The reply names a live Decision yours may conflict with — read it before going on.
- **Notes** are curated knowledge corrected in place (`anb edit`), kinds `fact`, `term`, `guide`.
- In prose, a bare id is a reference and a backticked one is a quotation. Cite records by id in bodies and comments; a citation of an id that exists nowhere is nudged back as `dangling-mention` — a forward reference is legal, a typo is not.

## The user's own notebook

`--global` names the user's notebook in the home directory: knowledge that outlives one repository. Decisions and Notes live there; Tasks and Questions are refused, work stays in the project. A practice is a global Note tagged `skill`, addressed by name — `anb search <name> --global`, then `anb show <id> --global` — so "use my skill X" resolves through anb, never through a pasted path. A project rule that stands against one of the user's cites the user's id in its body; Status then names the pair.

## Before you stop

`anb check` must be green — it verifies every file and names the move that repairs each finding. Commit the notebook with the code it describes.
"#;

// ---------------------------------------------------------------------------
// The references — the depth, one file per need
// ---------------------------------------------------------------------------

/// A reference file: frontmatter carrying the generated mark, a title, and
/// the body `section` renders.
fn reference(name: &str, description: &str, section: impl FnOnce(&mut String)) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "---\nname: {name}\ndescription: {description}");
    out.push_str("metadata:\n  generated: anb\n---\n\n");
    let _ = writeln!(out, "# {name}\n");
    out.push_str("Generated from the binary — the same definitions `anb --help` prints, every example run on a scratch notebook; a committed copy is checked against this rendering in CI.\n\n");
    section(&mut out);
    out
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
    out.push_str("Every refusal is `error[<code>]: <message>` and then `try:` lines — commands that run as printed. The codes are stable; the messages name the record and the fact.\n\n");
    let mut notebook = Scratch::new();
    for step in REFUSALS {
        match step {
            Step::Say(text) => {
                let _ = writeln!(out, "{text}\n");
            }
            Step::Run(line) => notebook.example(out, line),
            Step::Report(path, body) => notebook.reports.push((path, body)),
        }
    }
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
    /// Prose between examples.
    Say(&'static str),
    /// A command line, run and rendered.
    Run(&'static [&'static str]),
    /// A report file the next `close --note` reads.
    Report(&'static str, &'static str),
}

const SESSION: &[Step] = &[
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
    Step::Say("The queue shows what can start now; the blocked child waits:"),
    Step::Run(&["ready"]),
    Step::Run(&["ready", "--for", "task.ship-the-parser"]),
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
    Step::Say(
        "A ruling is a Decision; a term is a Note. A second Decision on the same ground is nudged about the first — read it before going on:",
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
    Step::Say("The doubt closes into the record that settled it, and is archived right after:"),
    Step::Run(&[
        "close",
        "question.do-fences-nest",
        "--resolved-by",
        "decision.fences-never-nest",
    ]),
    Step::Run(&["archive", "question.do-fences-nest"]),
    Step::Say(
        "Status is the session's opening — the active Task with its last log line, the rules, the queue, the epics:",
    ),
    Step::Run(&["status", "--budget", "0"]),
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
    Step::Say(
        "Reading back: one record, the whole notebook, a search that reaches the archive, and the gate — clean, because every settled record was archived as it settled:",
    ),
    Step::Run(&["show", "task.ship-the-parser"]),
    Step::Run(&["list"]),
    Step::Run(&["search", "fence"]),
    Step::Run(&["check"]),
];

const REFUSALS: &[Step] = &[
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
    Step::Say("`unknown-id`: no record carries the id."),
    Step::Run(&["start", "task.parser"]),
    Step::Say(
        "`invalid-transition`: the record's state does not allow the move; the valid moves are listed, each with its command.",
    ),
    Step::Run(&["close", "task.ship-the-parser", "--no-proof"]),
    Step::Say(
        "`invalid-argument`: a flag or value is malformed or foreign to the record; the retry shape is given.",
    ),
    Step::Run(&["add", "decision", "Tabs", "--priority", "2"]),
    Step::Run(&["close", "task.ship-the-parser"]),
    Step::Say("`dangling-ref`: an envelope reference names a record that does not exist."),
    Step::Run(&["add", "task", "A child", "--from", "task.ghost"]),
    Step::Say("`would-cycle`: the edge would close a dependency cycle, which is walked in full."),
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
    Step::Say("`duplicate-id`: ids are never reused, the archive included."),
    Step::Run(&[
        "add",
        "task",
        "Ship the parser again",
        "--id",
        "task.ship-the-parser",
    ]),
    Step::Say("`wrong-type`: the id names a type the command does not act on."),
    Step::Run(&["comment", "note.fence", "a line"]),
    Step::Say(
        "`still-referenced`: a delete would leave the notebook pointing at nothing; every holder is named.",
    ),
    Step::Run(&["delete", "task.ship-the-parser"]),
    Step::Say("`cannot-supersede`: the record named by `--supersedes` cannot die by supersession."),
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
    Step::Say(
        "`archived`: an archived record is read, never mutated in place; `restore` brings it back.",
    ),
    Step::Run(&["archive", "decision.fences-never-nest"]),
    Step::Run(&["edit", "decision.fences-never-nest", "--title", "Fences"]),
    Step::Say(
        "`invalid-record`: the file carries error findings, which close it to every verb until `check` is answered.",
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
            assert!(is_generated(text), "{file} lacks the mark");
        }
        assert!(!is_generated(
            &skill.skill.replace("  generated: anb\n", "")
        ));
        assert!(!is_generated("# No frontmatter at all\n"));
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
