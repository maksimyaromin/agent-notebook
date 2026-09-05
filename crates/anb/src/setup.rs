//! `setup`: wire an agent's session start to the notebook, in the project.
//!
//! Three mechanisms, each bounded so a re-run patches in place and
//! `--remove` takes out only what setup put in: one descriptive line in the
//! instruction files every agent reads — `AGENTS.md`, and `CLAUDE.md`, which
//! Claude Code reads instead — between markers; one `SessionStart` hook
//! group in the settings Claude Code and Codex run hooks from; and the anb
//! skill files where each agent looks for skills, known as setup's own by
//! the generated mark in their frontmatter. Other tools' lines and hook
//! groups in the same files are never touched; a skill file the user made
//! theirs is theirs.

use crate::skill;
use anb_core::{NotebookError, StorageError};
use serde_json::{Map, Value, json};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const BEGIN: &str = "<!-- anb:begin -->";
const END: &str = "<!-- anb:end -->";

/// The one line an agent reads before it knows the tool: descriptive, so it
/// informs without instructing, and marker-bounded, so it is found again.
pub const SNIPPET: &str = "<!-- anb:begin -->Project working memory: .agent-notebook/ — `anb status` shows the current state, `anb --help` the commands.<!-- anb:end -->";

/// The command both hosts run at session start; also the marker by which
/// setup recognises its own hook group among others.
pub const HOOK_COMMAND: &str = "anb status --hook";

/// The hook's own ceiling. Status is bounded and fails soft, so it never
/// needs the hosts' ten-minute default; a stuck lock must not hold a
/// session's start for longer than a reader would wait.
const HOOK_TIMEOUT_SECONDS: u64 = 15;

const AGENTS_FILE: &str = "AGENTS.md";
const CLAUDE_FILE: &str = "CLAUDE.md";
const CLAUDE_SETTINGS: &str = ".claude/settings.json";
const CODEX_HOOKS: &str = ".codex/hooks.json";
/// Where each host looks for skills: Claude Code in its own directory,
/// Codex and Pi in the shared one.
const SKILL_DIRS: [&str; 2] = [".claude/skills/anb", ".agents/skills/anb"];

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
    /// A skill file whose frontmatter no longer says setup generated it: the
    /// user made it theirs, and setup neither rewrites nor removes it.
    Yours,
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
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Wired {
    pub path: String,
    pub outcome: Outcome,
}

/// The reply: every file setup looked at, and the notice a host's own trust
/// step earns.
#[derive(Debug, PartialEq, Eq)]
pub struct SetUp {
    pub removed: bool,
    pub files: Vec<Wired>,
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
    /// A file under a directory setup owns whole: the file goes, and so does
    /// every directory this leaves empty, up to and including that one.
    DeleteUnder(PathBuf, PathBuf),
}

/// Install into, or remove from, the project at `project`. Every file is
/// read and judged before the first is written, so a refusal leaves the
/// project exactly as it was.
///
/// # Errors
/// [`NotebookError::InvalidArgument`] when a settings file is not JSON or an
/// instruction file carries a stray marker — setup rewrites nothing it
/// cannot read back — or a storage failure on any read or write.
pub fn apply(project: &Path, remove: bool) -> Result<SetUp, NotebookError> {
    let mut plans = vec![
        plan_snippet(project, AGENTS_FILE, remove)?,
        plan_claude_file(project, remove)?,
        plan_hook(project, CLAUDE_SETTINGS, remove)?,
        plan_hook(project, CODEX_HOOKS, remove)?,
    ];
    let rendered = skill::render();
    for dir in SKILL_DIRS {
        for (file, text) in rendered.files() {
            plans.push(plan_skill_file(project, dir, file, text, remove)?);
        }
    }
    let mut files = Vec::new();
    for Planned { wired, pending } in plans {
        if let Some(pending) = pending {
            perform(pending)?;
        }
        files.push(wired);
    }
    let codex_written = files
        .iter()
        .any(|wired| wired.path == CODEX_HOOKS && wired.outcome == Outcome::Written);
    Ok(SetUp {
        removed: remove,
        files,
        notice: codex_written.then_some(CODEX_NOTICE),
    })
}

/// A generated skill file: written where absent or stale, left alone once
/// the user has made it theirs by dropping the generated mark.
fn plan_skill_file(
    project: &Path,
    dir: &str,
    file: &str,
    rendered: &str,
    remove: bool,
) -> Result<Planned, NotebookError> {
    let shown = format!("{dir}/{file}");
    let path = project.join(dir).join(file);
    if is_link(&path) {
        return Ok(left_alone(shown, Outcome::Linked));
    }
    let existing = read_text(&path)?;
    if existing
        .as_deref()
        .is_some_and(|text| !skill::is_generated(text))
    {
        return Ok(left_alone(shown, Outcome::Yours));
    }
    let (outcome, pending) = match (remove, existing) {
        (true, Some(_)) => (
            Outcome::Removed,
            Some(Pending::DeleteUnder(path, project.join(dir))),
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
) -> Result<Planned, NotebookError> {
    let path = project.join(file);
    if is_link(&path) {
        return Ok(left_alone(file, Outcome::Linked));
    }
    let text = read_text(&path)?.unwrap_or_default();
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
fn plan_claude_file(project: &Path, remove: bool) -> Result<Planned, NotebookError> {
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
    plan_snippet(project, CLAUDE_FILE, remove)
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
    if is_link(&path) {
        return Ok(left_alone(file, Outcome::Linked));
    }
    let settings = read_settings(&path, file)?;
    let (outcome, pending) = if remove {
        match hook_removed(settings) {
            Some(rest) if rest.as_object().is_some_and(Map::is_empty) => {
                (Outcome::Removed, Some(Pending::Delete(path)))
            }
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
    if !patched.is_empty() && !patched.ends_with('\n') {
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
    Ok(Some(format!("{}{}", &text[..start], &text[end..])))
}

/// The byte range of the marker-bounded segment, begin marker to the end of
/// the end marker. A marker standing without its pair is refused: patching
/// around it would leave two openings, and guessing which line the writer
/// meant is not setup's to do.
fn bounded_line(text: &str, file: &str) -> Result<Option<(usize, usize)>, NotebookError> {
    let stray = || NotebookError::InvalidArgument {
        reason: format!(
            "setup: {file} carries an anb marker without its pair — fix or remove that line"
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

/// `settings` with setup's `SessionStart` group added; `None` when a group
/// already runs the hook command.
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
    if groups.iter().any(runs_the_hook) {
        return None;
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

/// Setup's hook is recognised by its command alone, so a timeout the user
/// tuned by hand keeps the group ours.
fn is_the_hook(hook: &Value) -> bool {
    hook.get("command")
        .and_then(Value::as_str)
        .is_some_and(|command| command.trim_start().starts_with(HOOK_COMMAND))
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
    serde_json::from_str(&text).map_err(|error| NotebookError::InvalidArgument {
        reason: format!("setup: {file} is not JSON ({error}) — fix it or move it aside"),
    })
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
            format!("# Guide without a final newline\n{SNIPPET}\n")
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
        assert_eq!(hook_removed(json!({"model": "opus"})), None);
    }

    #[test]
    fn a_hand_tuned_timeout_keeps_the_group_ours() {
        let tuned = json!({"hooks": {"SessionStart": [{"hooks": [
            {"type": "command", "command": "anb status --hook --budget 800", "timeout": 5}
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
