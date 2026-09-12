//! `setup`: wire an agent's session start to the notebook, in the project.
//!
//! Instruction markers, hook commands and skill frontmatter identify the
//! files setup may update. The setup receipt remembers instruction files
//! after a project removes their markers, so an upgrade preserves the
//! project's replacement. Project workflow extensions remain user-owned.

use crate::skill;
use anb_core::{NotebookError, Storage, StorageError};
use serde_json::{Map, Value, json};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const BEGIN: &str = "<!-- anb:begin -->";
const END: &str = "<!-- anb:end -->";

/// A marked discovery paragraph for agents that have not loaded the skill.
pub const SNIPPET: &str = "<!-- anb:begin -->Project working memory lives in .agent-notebook/: plain Markdown containing decisions, domain knowledge and unfinished work. Read the files directly when anb is unavailable; no skill is needed for reading. With anb installed, start with `anb recall` for work and context, or `anb --help` for commands. Project workflow extensions belong in .agents/anb.md.<!-- anb:end -->";

/// The command both hosts run at session start; also the marker by which
/// setup recognises its own hook group among others.
pub const HOOK_COMMAND: &str = "anb hook";
const LEGACY_HOOK_COMMAND: &str = "anb status --hook";

/// Host-side ceiling on session-start context collection, including lock waits.
const HOOK_TIMEOUT_SECONDS: u64 = 15;

const AGENTS_FILE: &str = "AGENTS.md";
const CLAUDE_FILE: &str = "CLAUDE.md";
const CLAUDE_SETTINGS: &str = ".claude/settings.json";
const CODEX_HOOKS: &str = ".codex/hooks.json";
const CLAUDE_SKILLS: &str = ".claude/skills";
const SHARED_SKILLS: &str = ".agents/skills";
const RECEIPT_FILE: &str = ".anb-setup.json";

/// The instruction file an agent reads its one line from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instructions {
    /// `CLAUDE.md`, or `AGENTS.md` when `CLAUDE.md` links or imports it.
    Claude,
    /// `AGENTS.md`, the file of the agents.md convention.
    Agents,
}

/// One agent setup can wire: what it reads instructions from, where its
/// host runs a session-start hook, where it looks for skills.
#[derive(Debug)]
pub struct Agent {
    pub name: &'static str,
    instructions: Instructions,
    hook: Option<&'static str>,
    skills: &'static str,
}

/// Every agent setup knows. Codex and the agents.md convention share the
/// instruction file and the skills directory; only Codex runs a project
/// hook, so a tool that follows the convention alone is named apart.
pub const AGENTS: [Agent; 3] = [
    Agent {
        name: "claude-code",
        instructions: Instructions::Claude,
        hook: Some(CLAUDE_SETTINGS),
        skills: CLAUDE_SKILLS,
    },
    Agent {
        name: "codex",
        instructions: Instructions::Agents,
        hook: Some(CODEX_HOOKS),
        skills: SHARED_SKILLS,
    },
    Agent {
        name: "agents-md",
        instructions: Instructions::Agents,
        hook: None,
        skills: SHARED_SKILLS,
    },
];

/// The agent names as `--agent` takes them, in the order setup lists them.
#[must_use]
pub fn agent_names() -> Vec<&'static str> {
    AGENTS.iter().map(|agent| agent.name).collect()
}

/// What Codex asks of a project hook before it runs it, printed so a
/// silent first session is not mistaken for a broken install.
pub const CODEX_NOTICE: &str =
    "Codex runs a project hook after you review it: run /hooks in Codex from this directory";

/// What setup did to one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Written,
    Already,
    Removed,
    Absent,
    /// `CLAUDE.md` already imports `AGENTS.md`, so the snippet there reaches
    /// Claude Code through the import.
    Imports,
    /// `CLAUDE.md` is a link to `AGENTS.md`: one file, one line.
    Links,
    /// The file is a link to somewhere else. Writing through it would edit a
    /// file the project does not own, so setup leaves it as it found it.
    Linked,
    /// A skill or instruction file the project has taken over.
    Yours,
    /// A file or directory an agent not named reads too: removal takes it
    /// out only when every agent that reads it is named.
    Shared,
}

impl Outcome {
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Outcome::Written => "written",
            Outcome::Already => "already",
            Outcome::Removed => "removed",
            Outcome::Absent => "absent",
            Outcome::Imports => "imports AGENTS.md",
            Outcome::Links => "links AGENTS.md",
            Outcome::Linked => "a link, left alone",
            Outcome::Yours => "yours, left alone",
            Outcome::Shared => "read by an agent not named, kept",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Wired {
    pub path: String,
    pub outcome: Outcome,
}

/// The reply: every file setup looked at, the agents it left alone, and the
/// notice a host's own trust step earns.
#[derive(Debug, PartialEq, Eq)]
pub struct SetUp {
    pub removed: bool,
    pub files: Vec<Wired>,
    /// The agents not named, so the choice is on record in the log.
    pub skipped: Vec<&'static str>,
    pub notice: Option<&'static str>,
}

/// One file's part of a run, decided before any file changes: what setup
/// reports for it, and the write it still owes.
struct Planned {
    wired: Wired,
    pending: Option<Pending>,
}

enum Pending {
    Text(PathBuf, String),
    Settings(PathBuf, Value),
    Delete(PathBuf),
    /// A file under a host's directory: the file goes, and so does every
    /// directory this leaves empty, up to and including that one.
    DeleteUnder(PathBuf, PathBuf),
}

/// Install into, or remove from, the project at `project`, for the agents
/// `named` and no other: a file two of them share is planned once. Every
/// file is read and judged before the first is written, so a refusal
/// leaves the project exactly as it was.
///
/// # Errors
/// [`NotebookError::InvalidArgument`] when no agent is named or a name is
/// not one setup knows, when a settings file is not JSON or an instruction
/// file carries a stray marker — setup rewrites nothing it cannot read
/// back — or a storage failure on any read or write.
pub fn apply(project: &Path, remove: bool, named: &[String]) -> Result<SetUp, NotebookError> {
    let chosen = chosen_agents(named)?;
    let receipt = read_receipt(project)?;
    let mut instructions = receipt.clone().unwrap_or_default();
    let is_chosen = |agent: &Agent| chosen.iter().any(|it| it.name == agent.name);
    let reads_agents = |agent: &Agent| {
        agent.instructions == Instructions::Agents
            || (agent.instructions == Instructions::Claude && reads_agents_file(project))
    };
    // A removal takes a shared file out only when every agent that reads
    // it is named: the file is as much the unnamed agent's.
    let all_readers_named =
        |reads: &dyn Fn(&Agent) -> bool| AGENTS.iter().filter(|agent| reads(agent)).all(is_chosen);
    let was_installed = |file: &str| {
        instructions.iter().any(|name| name == file)
            || chosen.iter().any(|agent| {
                let reads = if file == AGENTS_FILE {
                    reads_agents(agent)
                } else {
                    agent.instructions == Instructions::Claude && !reads_agents(agent)
                };
                reads && project.join(agent.skills).join("anb/SKILL.md").is_file()
            })
    };

    let mut plans = Vec::new();
    if chosen.iter().any(|agent| reads_agents(agent)) {
        if remove && !all_readers_named(&reads_agents) {
            plans.push(left_alone(AGENTS_FILE, Outcome::Shared));
        } else {
            plans.push(plan_snippet(
                project,
                AGENTS_FILE,
                remove,
                was_installed(AGENTS_FILE),
            )?);
        }
    }
    if chosen
        .iter()
        .any(|agent| agent.instructions == Instructions::Claude)
    {
        plans.push(plan_claude_file(
            project,
            remove,
            was_installed(CLAUDE_FILE),
        )?);
    }
    for hook in distinct(chosen.iter().filter_map(|agent| agent.hook)) {
        plans.push(plan_hook(project, hook, remove)?);
    }
    for host in distinct(chosen.iter().map(|agent| agent.skills)) {
        if remove && !all_readers_named(&|agent: &Agent| agent.skills == host) {
            plans.push(left_alone(host, Outcome::Shared));
            continue;
        }
        for skill in skill::installable() {
            let dir = format!("{host}/{}", skill.name);
            for (file, text) in &skill.files {
                plans.push(plan_skill_file(project, &dir, file, text, remove)?);
            }
        }
    }
    let mut files = Vec::new();
    for Planned { wired, pending } in plans {
        if let Some(pending) = pending {
            perform(pending)?;
        }
        // A later file failure must not erase a completed instruction's ownership history.
        if remember_instruction(&wired, remove, &mut instructions) {
            write_receipt(project, &instructions)?;
        }
        files.push(wired);
    }
    if receipt.is_some() || !instructions.is_empty() {
        let outcome = if receipt.as_ref() == Some(&instructions) {
            Outcome::Already
        } else if instructions.is_empty() {
            Outcome::Removed
        } else {
            Outcome::Written
        };
        files.push(Wired {
            path: RECEIPT_FILE.to_owned(),
            outcome,
        });
    }
    let codex_written = files
        .iter()
        .any(|wired| wired.path == CODEX_HOOKS && wired.outcome == Outcome::Written);
    Ok(SetUp {
        removed: remove,
        files,
        skipped: AGENTS
            .iter()
            .filter(|agent| !chosen.iter().any(|it| it.name == agent.name))
            .map(|agent| agent.name)
            .collect(),
        notice: codex_written.then_some(CODEX_NOTICE),
    })
}

fn read_receipt(project: &Path) -> Result<Option<Vec<String>>, NotebookError> {
    let path = project.join(RECEIPT_FILE);
    if is_link(&path) {
        return Err(NotebookError::InvalidArgument {
            reason: format!("setup: {RECEIPT_FILE} is a link. Move it aside before running setup"),
        });
    }
    let Some(text) = read_text(&path)? else {
        return Ok(None);
    };
    let invalid = || NotebookError::InvalidArgument {
        reason: format!(
            "setup: {RECEIPT_FILE} must contain an instructions array naming AGENTS.md or CLAUDE.md. Correct the receipt before running setup"
        ),
    };
    let receipt: Value = serde_json::from_str(&text).map_err(|_| invalid())?;
    let instructions: Vec<String> =
        serde_json::from_value(receipt.get("instructions").cloned().ok_or_else(invalid)?)
            .map_err(|_| invalid())?;
    if instructions
        .iter()
        .any(|file| !matches!(file.as_str(), AGENTS_FILE | CLAUDE_FILE))
    {
        return Err(invalid());
    }
    Ok(Some(instructions))
}

fn remember_instruction(wired: &Wired, remove: bool, instructions: &mut Vec<String>) -> bool {
    if !matches!(wired.path.as_str(), AGENTS_FILE | CLAUDE_FILE) {
        return false;
    }
    if remove && matches!(wired.outcome, Outcome::Removed | Outcome::Absent) {
        let before = instructions.len();
        instructions.retain(|file| file != &wired.path);
        return before != instructions.len();
    }
    if matches!(
        wired.outcome,
        Outcome::Written | Outcome::Already | Outcome::Yours
    ) && !instructions.contains(&wired.path)
    {
        instructions.push(wired.path.clone());
        instructions.sort();
        return true;
    }
    false
}

fn write_receipt(project: &Path, instructions: &[String]) -> Result<(), NotebookError> {
    let path = project.join(RECEIPT_FILE);
    if instructions.is_empty() {
        return fs::remove_file(&path).map_err(|error| io_failure(&path, &error));
    }
    let mut text = serde_json::to_string_pretty(&json!({ "instructions": instructions }))
        .expect("JSON values always serialise");
    text.push('\n');
    crate::fs_storage::FsStorage::new(project.to_owned())
        .write(RECEIPT_FILE, &text)
        .map_err(NotebookError::Storage)
}

/// The agents `named`, each once, in setup's own order. Nothing is written
/// for an agent nobody named: the files of a tool the project does not run
/// read as noise at best and as a commitment at worst.
fn chosen_agents(named: &[String]) -> Result<Vec<&'static Agent>, NotebookError> {
    let known = || agent_names().join(", ");
    if named.is_empty() {
        return Err(NotebookError::InvalidArgument {
            reason: format!(
                "setup: name the agents to wire with --agent, one of {}",
                known()
            ),
        });
    }
    if let Some(unknown) = named
        .iter()
        .find(|name| !AGENTS.iter().any(|agent| agent.name == name.as_str()))
    {
        return Err(NotebookError::InvalidArgument {
            reason: format!("setup: `{unknown}` is not an agent; one of {}", known()),
        });
    }
    Ok(AGENTS
        .iter()
        .filter(|agent| named.iter().any(|name| name == agent.name))
        .collect())
}

/// Each value once, in first-seen order.
fn distinct<'a>(values: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut seen = Vec::new();
    for value in values {
        if !seen.contains(&value) {
            seen.push(value);
        }
    }
    seen
}

/// Whether Claude Code reaches `AGENTS.md` through its own file: a
/// `CLAUDE.md` that is a link to it or imports it.
fn reads_agents_file(project: &Path) -> bool {
    let path = project.join(CLAUDE_FILE);
    if is_link(&path) {
        return links_agents_file(project, &path);
    }
    read_text(&path)
        .ok()
        .flatten()
        .is_some_and(|text| imports_agents_file(&text))
}

/// A skill file: written where absent or stale, left alone once the user
/// has made it theirs by dropping the mark.
fn plan_skill_file(
    project: &Path,
    dir: &str,
    file: &str,
    rendered: &str,
    remove: bool,
) -> Result<Planned, NotebookError> {
    let shown = format!("{dir}/{file}");
    let path = project.join(dir).join(file);
    ensure_local_parent(project, &path)?;
    if is_link(&path) {
        return Ok(left_alone(shown, Outcome::Linked));
    }
    let existing = read_text(&path)?;
    if existing
        .as_deref()
        .is_some_and(|text| !skill::is_managed(text))
    {
        return Ok(left_alone(shown, Outcome::Yours));
    }
    let (outcome, pending) = match (remove, existing) {
        (true, Some(_)) => (
            Outcome::Removed,
            Some(Pending::DeleteUnder(path, host_root(project, dir))),
        ),
        (true, None) => (Outcome::Absent, None),
        (false, Some(text)) if text == rendered => (Outcome::Already, None),
        (false, _) => (
            Outcome::Written,
            Some(Pending::Text(path, rendered.to_owned())),
        ),
    };
    Ok(Planned {
        wired: Wired {
            path: shown,
            outcome,
        },
        pending,
    })
}

fn perform(pending: Pending) -> Result<(), NotebookError> {
    match pending {
        Pending::Text(path, text) => write_text(&path, &text),
        Pending::Settings(path, settings) => write_settings(&path, &settings),
        Pending::Delete(path) => fs::remove_file(&path).map_err(|error| io_failure(&path, &error)),
        Pending::DeleteUnder(path, root) => {
            fs::remove_file(&path).map_err(|error| io_failure(&path, &error))?;
            remove_emptied_dirs(&path, &root);
            Ok(())
        }
    }
}

fn plan_snippet(
    project: &Path,
    file: &'static str,
    remove: bool,
    was_installed: bool,
) -> Result<Planned, NotebookError> {
    let path = project.join(file);
    if is_link(&path) {
        return Ok(left_alone(file, Outcome::Linked));
    }
    let text = read_text(&path)?.unwrap_or_default();
    if bounded_line(&text, file)?.is_none() && was_installed {
        return Ok(left_alone(file, Outcome::Yours));
    }
    let (outcome, pending) = if remove {
        match snippet_removed(&text, file)? {
            Some(rest) if rest.trim().is_empty() => (Outcome::Removed, Some(Pending::Delete(path))),
            Some(rest) => (Outcome::Removed, Some(Pending::Text(path, rest))),
            None => (Outcome::Absent, None),
        }
    } else {
        match snippet_applied(&text, file)? {
            Some(patched) => (Outcome::Written, Some(Pending::Text(path, patched))),
            None => (Outcome::Already, None),
        }
    };
    Ok(Planned {
        wired: Wired {
            path: file.to_owned(),
            outcome,
        },
        pending,
    })
}

/// Claude Code reads `CLAUDE.md`, not `AGENTS.md`. A `CLAUDE.md` that links
/// or imports `AGENTS.md` already carries the snippet through it, and a
/// second copy would reach Claude twice; any other one gets its own line.
fn plan_claude_file(
    project: &Path,
    remove: bool,
    was_installed: bool,
) -> Result<Planned, NotebookError> {
    let path = project.join(CLAUDE_FILE);
    if is_link(&path) {
        let outcome = if links_agents_file(project, &path) {
            Outcome::Links
        } else {
            Outcome::Linked
        };
        return Ok(left_alone(CLAUDE_FILE, outcome));
    }
    if read_text(&path)?.is_some_and(|text| imports_agents_file(&text)) {
        return Ok(left_alone(CLAUDE_FILE, Outcome::Imports));
    }
    plan_snippet(project, CLAUDE_FILE, remove, was_installed)
}

fn left_alone(file: impl Into<String>, outcome: Outcome) -> Planned {
    Planned {
        wired: Wired {
            path: file.into(),
            outcome,
        },
        pending: None,
    }
}

fn is_link(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink())
}

fn ensure_local_parent(project: &Path, path: &Path) -> Result<(), NotebookError> {
    for parent in path
        .ancestors()
        .skip(1)
        .take_while(|parent| *parent != project)
    {
        match fs::symlink_metadata(parent) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(NotebookError::InvalidArgument {
                    reason: format!(
                        "setup: {} is a linked directory; use a project-owned directory before running setup",
                        parent.strip_prefix(project).unwrap_or(parent).display()
                    ),
                });
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(io_failure(parent, &error)),
        }
    }
    Ok(())
}

fn links_agents_file(project: &Path, claude: &Path) -> bool {
    let (Ok(claude), Ok(agents)) = (
        fs::canonicalize(claude),
        fs::canonicalize(project.join(AGENTS_FILE)),
    ) else {
        return false;
    };
    claude == agents
}

/// Whether `CLAUDE.md` pulls `AGENTS.md` in: an `@AGENTS.md` import on a line
/// of its own, the shape Claude Code's own docs give.
fn imports_agents_file(text: &str) -> bool {
    text.lines()
        .any(|line| matches!(line.trim(), "@AGENTS.md" | "@./AGENTS.md"))
}

fn plan_hook(project: &Path, file: &'static str, remove: bool) -> Result<Planned, NotebookError> {
    let path = project.join(file);
    ensure_local_parent(project, &path)?;
    if is_link(&path) {
        return Ok(left_alone(file, Outcome::Linked));
    }
    let settings = read_settings(&path, file)?;
    let (outcome, pending) = if remove {
        match hook_removed(settings) {
            Some(rest) if rest.as_object().is_some_and(Map::is_empty) => (
                Outcome::Removed,
                Some(Pending::DeleteUnder(path, host_root(project, file))),
            ),
            Some(rest) => (Outcome::Removed, Some(Pending::Settings(path, rest))),
            None => (Outcome::Absent, None),
        }
    } else {
        match hook_applied(settings) {
            Some(patched) => (Outcome::Written, Some(Pending::Settings(path, patched))),
            None => (Outcome::Already, None),
        }
    };
    Ok(Planned {
        wired: Wired {
            path: file.to_owned(),
            outcome,
        },
        pending,
    })
}

/// `text` with the snippet in place: the bounded line replaced where one
/// stands, appended where none does. `None` when the text already carries
/// this very line.
///
/// # Errors
/// [`NotebookError::InvalidArgument`] on a stray marker in `file`.
pub fn snippet_applied(text: &str, file: &str) -> Result<Option<String>, NotebookError> {
    if let Some((start, end)) = bounded_line(text, file)? {
        if &text[start..end] == SNIPPET {
            return Ok(None);
        }
        return Ok(Some(format!("{}{SNIPPET}{}", &text[..start], &text[end..])));
    }
    let mut patched = text.to_owned();
    // CommonMark reads a non-blank line right after a list item or a
    // paragraph as a continuation of it, so the snippet stands behind a
    // blank line of its own.
    while !patched.is_empty() && !patched.ends_with("\n\n") {
        patched.push('\n');
    }
    patched.push_str(SNIPPET);
    patched.push('\n');
    Ok(Some(patched))
}

/// `text` without the bounded line, its own newline included; `None` when
/// there is none to remove.
///
/// # Errors
/// [`NotebookError::InvalidArgument`] on a stray marker in `file`.
pub fn snippet_removed(text: &str, file: &str) -> Result<Option<String>, NotebookError> {
    let Some((start, end)) = bounded_line(text, file)? else {
        return Ok(None);
    };
    let end = if text[end..].starts_with('\n') {
        end + 1
    } else {
        end
    };
    let (before, after) = (&text[..start], &text[end..]);
    let before = if after.is_empty() || after.starts_with('\n') {
        without_paragraph_break(before)
    } else {
        before
    };
    Ok(Some(format!("{before}{after}")))
}

/// `before` without the blank line that ends it, when one does. Setup put
/// that break ahead of an appended snippet where the guide ended on one
/// newline, and removal cannot tell that guide from one that ended on a
/// blank line of its own: the first comes back as it was, the second
/// ending on one newline.
fn without_paragraph_break(before: &str) -> &str {
    before
        .strip_suffix('\n')
        .filter(|kept| kept.ends_with('\n'))
        .unwrap_or(before)
}

/// The byte range of the marker-bounded segment, begin marker to the end of
/// the end marker. A marker standing without its pair is refused: patching
/// around it would leave two openings, and guessing which line the writer
/// meant is not setup's to do.
fn bounded_line(text: &str, file: &str) -> Result<Option<(usize, usize)>, NotebookError> {
    let stray = || NotebookError::InvalidArgument {
        reason: format!(
            "setup: {file} has an anb marker without its pair. Restore the missing marker or remove that line"
        ),
    };
    let Some(start) = text.find(BEGIN) else {
        return if text.contains(END) {
            Err(stray())
        } else {
            Ok(None)
        };
    };
    let end = text[start..].find(END).ok_or_else(stray)?;
    Ok(Some((start, start + end + END.len())))
}

/// Install the native hook or replace an exact legacy command in place.
#[must_use]
pub fn hook_applied(settings: Value) -> Option<Value> {
    let mut root = into_object(settings);
    let hooks = root
        .entry("hooks")
        .or_insert_with(|| Value::Object(Map::new()));
    let hooks = object_mut(hooks);
    let groups = hooks
        .entry("SessionStart")
        .or_insert_with(|| Value::Array(Vec::new()));
    let groups = array_mut(groups);
    let mut upgraded = false;
    for group in groups.iter_mut() {
        if let Some(hooks) = group.get_mut("hooks").and_then(Value::as_array_mut) {
            for hook in hooks {
                if hook
                    .get("command")
                    .and_then(Value::as_str)
                    .is_some_and(|command| command.trim() == LEGACY_HOOK_COMMAND)
                {
                    hook["command"] = json!(HOOK_COMMAND);
                    upgraded = true;
                }
            }
        }
    }
    if groups.iter().any(runs_the_hook) {
        return upgraded.then_some(Value::Object(root));
    }
    groups.push(json!({
        "hooks": [{"type": "command", "command": HOOK_COMMAND, "timeout": HOOK_TIMEOUT_SECONDS}]
    }));
    Some(Value::Object(root))
}

/// `settings` with setup's hook taken out of every `SessionStart` group and
/// the containers it leaves empty removed; `None` when no group runs it.
#[must_use]
pub fn hook_removed(settings: Value) -> Option<Value> {
    let mut root = into_object(settings);
    let groups = root
        .get_mut("hooks")
        .and_then(Value::as_object_mut)
        .and_then(|hooks| hooks.get_mut("SessionStart"))
        .and_then(Value::as_array_mut)?;
    if !groups.iter().any(runs_the_hook) {
        return None;
    }
    for group in groups.iter_mut() {
        if let Some(hooks) = group.get_mut("hooks").and_then(Value::as_array_mut) {
            hooks.retain(|hook| !is_the_hook(hook));
        }
    }
    groups.retain(|group| {
        group
            .get("hooks")
            .and_then(Value::as_array)
            .is_none_or(|hooks| !hooks.is_empty())
    });
    prune_empty(&mut root);
    Some(Value::Object(root))
}

fn runs_the_hook(group: &Value) -> bool {
    group
        .get("hooks")
        .and_then(Value::as_array)
        .is_some_and(|hooks| hooks.iter().any(is_the_hook))
}

/// Arguments or shell operators make a command user-owned. A separate
/// timeout setting does not change command ownership.
fn is_the_hook(hook: &Value) -> bool {
    hook.get("command")
        .and_then(Value::as_str)
        .is_some_and(|command| matches!(command.trim(), HOOK_COMMAND | LEGACY_HOOK_COMMAND))
}

/// Drop `hooks.SessionStart` when it emptied, and `hooks` after it: a key
/// setup added and left empty would be setup's leftover.
fn prune_empty(root: &mut Map<String, Value>) {
    let hooks_empty = root
        .get_mut("hooks")
        .and_then(Value::as_object_mut)
        .map(|hooks| {
            if hooks
                .get("SessionStart")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
            {
                hooks.remove("SessionStart");
            }
            hooks.is_empty()
        });
    if hooks_empty == Some(true) {
        root.remove("hooks");
    }
}

fn into_object(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}

fn object_mut(value: &mut Value) -> &mut Map<String, Value> {
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
    value.as_object_mut().expect("just made an object")
}

fn array_mut(value: &mut Value) -> &mut Vec<Value> {
    if !value.is_array() {
        *value = Value::Array(Vec::new());
    }
    value.as_array_mut().expect("just made an array")
}

fn read_text(path: &Path) -> Result<Option<String>, NotebookError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_failure(path, &error)),
    }
}

/// A settings file as JSON, an empty object when there is no file. A file
/// that is not JSON is refused: setup patches a document it can read back,
/// never one it would have to guess at.
fn read_settings(path: &Path, file: &str) -> Result<Value, NotebookError> {
    let Some(text) = read_text(path)? else {
        return Ok(Value::Object(Map::new()));
    };
    if text.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let settings: Value =
        serde_json::from_str(&text).map_err(|error| NotebookError::InvalidArgument {
            reason: format!(
                "setup: {file} is not JSON ({error}). Correct the file or move it aside"
            ),
        })?;
    let containers_valid = settings.is_object()
        && settings.get("hooks").is_none_or(|hooks| {
            hooks.is_object() && hooks.get("SessionStart").is_none_or(Value::is_array)
        });
    if !containers_valid {
        return Err(NotebookError::InvalidArgument {
            reason: format!(
                "setup: {file} must be an object, with hooks as an object and hooks.SessionStart as an array when present; correct those settings before running setup"
            ),
        });
    }
    Ok(settings)
}

/// The directory a host's files live under, `.codex` for `.codex/hooks.json`:
/// what removal may take out once it stands empty, since a host directory
/// holding nothing else was setup's alone.
fn host_root(project: &Path, file: &str) -> PathBuf {
    project.join(file.split_once('/').map_or(file, |(top, _)| top))
}

/// A directory setup emptied is setup's leftover; one holding anything else
/// stays, and nothing above `root` is ever touched.
fn remove_emptied_dirs(file: &Path, root: &Path) {
    let mut dir = file.parent();
    while let Some(emptied) = dir.filter(|dir| dir.starts_with(root)) {
        if fs::remove_dir(emptied).is_err() || emptied == root {
            return;
        }
        dir = emptied.parent();
    }
}

fn write_text(path: &Path, text: &str) -> Result<(), NotebookError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| io_failure(parent, &error))?;
    }
    fs::write(path, text).map_err(|error| io_failure(path, &error))
}

fn write_settings(path: &Path, settings: &Value) -> Result<(), NotebookError> {
    let mut text = serde_json::to_string_pretty(settings).expect("JSON values always serialise");
    text.push('\n');
    write_text(path, &text)
}

fn io_failure(path: &Path, error: &std::io::Error) -> NotebookError {
    NotebookError::Storage(StorageError::Io {
        path: PathBuf::from(path).display().to_string(),
        detail: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_exact_legacy_hook_upgrades_in_place_and_keeps_other_entries() {
        let before = json!({"permissions":{"allow":["Bash(ls)"]},"hooks":{"SessionStart":[{"matcher":"startup","hooks":[{"type":"command","command":"anb status --hook","timeout":5},{"type":"command","command":"other-tool prime"}]}]}});
        let mut expected = before.clone();
        expected["hooks"]["SessionStart"][0]["hooks"][0]["command"] = json!("anb hook");
        assert_eq!(hook_applied(before), Some(expected.clone()));
        assert_eq!(hook_applied(expected), None);
    }

    #[test]
    fn custom_hook_commands_are_neither_rewritten_nor_removed() {
        for command in [
            "anb status --hook --budget 800",
            "anb hook --custom",
            "anb hooker",
            "anb status --hook && other-tool prime",
        ] {
            let before = json!({"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":command}]}]}});
            assert_eq!(hook_removed(before.clone()), None, "{command}");
            let installed = hook_applied(before.clone()).unwrap();
            assert_eq!(
                installed["hooks"]["SessionStart"][0],
                before["hooks"]["SessionStart"][0]
            );
            assert_eq!(
                installed["hooks"]["SessionStart"].as_array().unwrap().len(),
                2
            );
            assert_eq!(hook_removed(installed), Some(before));
        }
    }

    #[test]
    fn setup_refuses_wrong_json_container_types_without_replacing_them() {
        for settings in [
            r#"["keep"]"#,
            r#"{"hooks":["keep"]}"#,
            r#"{"hooks":{"SessionStart":"keep"}}"#,
        ] {
            let project = tempfile::TempDir::new().unwrap();
            fs::create_dir(project.path().join(".claude")).unwrap();
            fs::write(project.path().join(CLAUDE_SETTINGS), settings).unwrap();
            assert!(
                apply(project.path(), false, &["claude-code".to_owned()]).is_err(),
                "{settings}"
            );
            assert_eq!(
                fs::read_to_string(project.path().join(CLAUDE_SETTINGS)).unwrap(),
                settings
            );
            assert!(!project.path().join(CLAUDE_FILE).exists());
            assert!(!project.path().join(RECEIPT_FILE).exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn setup_refuses_linked_parent_directories_before_any_write_or_removal() {
        for (agent, directory) in [
            ("claude-code", ".claude"),
            ("agents-md", ".agents/skills"),
            ("agents-md", ".agents/skills/anb/references"),
        ] {
            for remove in [false, true] {
                let project = tempfile::TempDir::new().unwrap();
                let outside = tempfile::TempDir::new().unwrap();
                let settings = r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"anb status --hook"}]}]}}"#;
                fs::write(outside.path().join("settings.json"), settings).unwrap();
                let link = project.path().join(directory);
                fs::create_dir_all(link.parent().unwrap()).unwrap();
                std::os::unix::fs::symlink(outside.path(), &link).unwrap();
                let agents = if agent == "agents-md" {
                    vec![agent.to_owned(), "codex".to_owned()]
                } else {
                    vec![agent.to_owned()]
                };
                assert!(
                    apply(project.path(), remove, &agents).is_err(),
                    "{directory}, remove={remove}"
                );
                assert_eq!(
                    fs::read_to_string(outside.path().join("settings.json")).unwrap(),
                    settings
                );
                assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 1);
                assert!(!project.path().join("AGENTS.md").exists());
                assert!(!project.path().join("CLAUDE.md").exists());
                assert!(!project.path().join(RECEIPT_FILE).exists());
            }
        }
    }

    #[test]
    fn the_snippet_is_appended_once_and_patched_in_place() {
        let fresh = snippet_applied("", "AGENTS.md").unwrap().unwrap();
        assert_eq!(fresh, format!("{SNIPPET}\n"));
        assert_eq!(
            snippet_applied(&fresh, "AGENTS.md").unwrap(),
            None,
            "a second run changes nothing"
        );

        let stale = "# Guide\n<!-- anb:begin -->an older wording<!-- anb:end -->\nmore\n";
        assert_eq!(
            snippet_applied(stale, "AGENTS.md").unwrap().unwrap(),
            format!("# Guide\n{SNIPPET}\nmore\n"),
            "an older line between the markers is replaced where it stands"
        );
        assert_eq!(
            snippet_applied("# Guide without a final newline", "AGENTS.md")
                .unwrap()
                .unwrap(),
            format!("# Guide without a final newline\n\n{SNIPPET}\n")
        );
    }

    #[test]
    fn a_snippet_appended_after_a_list_item_is_a_paragraph_of_its_own() {
        let ends_with_a_list = "# Guide\n- one paragraph or list item is one physical line\n";
        let patched = snippet_applied(ends_with_a_list, "AGENTS.md")
            .unwrap()
            .unwrap();
        assert_eq!(patched, format!("{ends_with_a_list}\n{SNIPPET}\n"));
        assert_eq!(snippet_applied(&patched, "AGENTS.md").unwrap(), None);
    }

    #[test]
    fn a_guide_already_ending_on_a_blank_line_gets_no_second_one() {
        let ends_blank = "# Guide\n\nRead the docs.\n\n";
        assert_eq!(
            snippet_applied(ends_blank, "AGENTS.md").unwrap().unwrap(),
            format!("{ends_blank}{SNIPPET}\n")
        );
    }

    #[test]
    fn a_guide_ending_on_a_newline_reads_as_it_did_after_a_round_trip() {
        let guide = "# Guide\n- one paragraph or list item is one physical line\n";
        let patched = snippet_applied(guide, "AGENTS.md").unwrap().unwrap();
        assert_eq!(
            snippet_removed(&patched, "AGENTS.md").unwrap().unwrap(),
            guide
        );
    }

    #[test]
    fn a_guide_without_a_final_newline_gains_one_over_a_round_trip() {
        let patched = snippet_applied("# Guide", "AGENTS.md").unwrap().unwrap();
        assert_eq!(
            snippet_removed(&patched, "AGENTS.md").unwrap().unwrap(),
            "# Guide\n"
        );
    }

    #[test]
    fn a_guide_ending_on_blank_lines_comes_back_one_blank_line_shorter() {
        for (guide, after) in [
            ("# Guide\n\n", "# Guide\n"),
            ("# Guide\n\n\n", "# Guide\n\n"),
        ] {
            let patched = snippet_applied(guide, "AGENTS.md").unwrap().unwrap();
            assert_eq!(
                snippet_removed(&patched, "AGENTS.md").unwrap().unwrap(),
                after,
                "{guide:?}"
            );
        }
    }

    #[test]
    fn a_snippet_placed_between_two_paragraphs_leaves_one_break_behind() {
        let mid_file = format!("# Guide\n\n{SNIPPET}\n\nmore\n");
        assert_eq!(
            snippet_removed(&mid_file, "AGENTS.md").unwrap().unwrap(),
            "# Guide\n\nmore\n"
        );
        let followed_by_text = format!("# Guide\n\n{SNIPPET}\nmore\n");
        assert_eq!(
            snippet_removed(&followed_by_text, "AGENTS.md")
                .unwrap()
                .unwrap(),
            "# Guide\n\nmore\n",
            "the break ahead of the snippet was the guide's own"
        );
    }

    #[test]
    fn a_marker_without_its_pair_is_refused_not_patched_around() {
        for stray in [
            "# Guide\n<!-- anb:begin -->half a line\n",
            "# Guide\nsome text<!-- anb:end -->\n",
        ] {
            let error = snippet_applied(stray, "AGENTS.md").unwrap_err();
            assert!(
                matches!(error, NotebookError::InvalidArgument { .. }),
                "{stray}"
            );
            assert!(snippet_removed(stray, "AGENTS.md").is_err(), "{stray}");
        }
    }

    #[test]
    fn removing_the_snippet_takes_its_line_and_nothing_else() {
        let text = format!("# Guide\n{SNIPPET}\nmore\n");
        assert_eq!(
            snippet_removed(&text, "AGENTS.md").unwrap().unwrap(),
            "# Guide\nmore\n"
        );
        assert_eq!(snippet_removed("# Guide\n", "AGENTS.md").unwrap(), None);
    }

    #[test]
    fn the_hook_joins_other_tools_groups_and_leaves_when_removed() {
        let theirs = json!({
            "permissions": {"allow": ["Bash(ls)"]},
            "hooks": {"SessionStart": [{"hooks": [{"type": "command", "command": "other-tool prime"}]}]}
        });
        let with_ours = hook_applied(theirs.clone()).unwrap();
        let groups = with_ours["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(groups.len(), 2, "our group is added beside theirs");
        assert_eq!(groups[1]["hooks"][0]["command"], json!(HOOK_COMMAND));
        assert_eq!(
            hook_applied(with_ours.clone()),
            None,
            "a second run changes nothing"
        );

        assert_eq!(
            hook_removed(with_ours).unwrap(),
            theirs,
            "removal leaves the other tool's group and the rest of the file as they were"
        );
    }

    #[test]
    fn removing_the_only_hook_leaves_no_empty_container_behind() {
        let ours = hook_applied(json!({})).unwrap();
        assert_eq!(hook_removed(ours).unwrap(), json!({}));
        assert_eq!(hook_removed(json!({"model": "default"})), None);
    }

    #[test]
    fn a_hand_tuned_timeout_keeps_the_group_ours() {
        let tuned = json!({"hooks": {"SessionStart": [{"hooks": [
            {"type": "command", "command": HOOK_COMMAND, "timeout": 5}
        ]}]}});
        assert_eq!(hook_applied(tuned.clone()), None);
        assert_eq!(hook_removed(tuned).unwrap(), json!({}));
    }

    #[test]
    fn a_claude_file_that_imports_the_agents_file_is_recognised() {
        assert!(imports_agents_file(
            "@AGENTS.md\n\n## Claude Code\nUse plan mode.\n"
        ));
        assert!(!imports_agents_file("See `@AGENTS.md` for the guide.\n"));
    }
}
