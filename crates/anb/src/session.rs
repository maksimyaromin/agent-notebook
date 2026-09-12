//! Local session focus and recoverable Task starts under the notebook lock.

use anb_core::{Filter, Notebook, NotebookError, Record, Storage, StorageError, Transitioned};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

pub const DIRECTORY: &str = ".sessions.tmp";

#[derive(Debug)]
pub struct Started {
    pub transition: Transitioned,
    pub session: Option<String>,
    pub joined: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    version: u8,
    session: String,
    identity: Option<String>,
    focus: Option<String>,
    pending: Option<Pending>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    task: String,
    today: String,
    before: String,
    after: String,
}

/// Resolve the explicit session, then `ANB_SESSION`, then `CODEX_THREAD_ID`.
///
/// # Errors
/// A provided session name is empty, contains control characters or is not UTF-8.
pub fn resolve(explicit: Option<String>) -> Result<Option<String>, NotebookError> {
    if let Some(session) = explicit {
        validate_name(&session)?;
        return Ok(Some(session));
    }
    for variable in ["ANB_SESSION", "CODEX_THREAD_ID"] {
        match std::env::var(variable) {
            Ok(session) => {
                validate_name(&session)?;
                return Ok(Some(session));
            }
            Err(std::env::VarError::NotPresent) => {}
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(invalid(format!("{variable}: the session id must be UTF-8")));
            }
        }
    }
    Ok(None)
}

/// Read remembered focus without changing local state or choosing a Task.
///
/// # Errors
/// Unreadable, malformed or interrupted session state is reported as
/// `session-recovery`, not as an empty session.
pub fn focus(
    storage: &dyn Storage,
    session: Option<&str>,
    identity: Option<&str>,
) -> Result<Option<String>, NotebookError> {
    let Some(session) = session else {
        return Ok(None);
    };
    validate_name(session)?;
    let Some(state) = read(storage, session)? else {
        return Ok(None);
    };
    guard_identity(&state, identity)?;
    if state.pending.is_some() {
        return Err(recovery(
            session,
            "a Task start was interrupted; retry start with this session",
        ));
    }
    Ok(state.focus)
}

/// Start or resume a Task and remember its focus for this local session.
/// The caller must hold the notebook's exclusive lock through this call.
///
/// # Errors
/// Task validation and ownership refusals, a conflicting session, or an
/// interrupted local write. A durable intent makes partial progress
/// recoverable without overwriting intervening record edits.
pub fn start(
    storage: &mut dyn Storage,
    session: Option<&str>,
    identity: Option<&str>,
    request: &Start,
    today: &str,
) -> Result<Started, NotebookError> {
    if request.join && session.is_none() {
        return Err(invalid(
            "start: --join requires --session or an agent session id",
        ));
    }
    if request.next && request.id.is_some() {
        return Err(invalid("start: choose a Task id or --next, not both"));
    }
    if request.hub.is_some() && !request.next {
        return Err(invalid("start: --for narrows --next"));
    }
    let mut resumed = None;
    let mut state = session
        .map(|session| {
            validate_name(session)?;
            let mut state = read(storage, session)?.unwrap_or_else(|| State {
                version: 1,
                session: session.to_owned(),
                identity: identity.map(str::to_owned),
                focus: None,
                pending: None,
            });
            guard_identity(&state, identity)?;
            resumed = state.pending.as_ref().map(|pending| pending.task.clone());
            reconcile(storage, &mut state)?;
            Ok::<_, NotebookError>(state)
        })
        .transpose()?;
    let claims = claims(storage, session)?;
    let excluded = claims
        .iter()
        .filter(|claim| !request.join || claim.pending)
        .map(|claim| claim.task.clone())
        .collect();
    let mut staged = Staged::new(storage);
    let mut notebook = Notebook::new(&mut staged).with_identity(identity);
    if request.next && resumed.is_none() {
        resumed = active_focus(&notebook, state.as_ref(), request.hub.as_deref())?;
    }
    let transition = if request.next && resumed.is_none() {
        notebook
            .start_next(
                &Filter {
                    hub: request.hub.clone(),
                    ..Filter::default()
                },
                &excluded,
                today,
            )?
            .ok_or_else(|| {
                invalid("start: no eligible Task is ready; inspect anb ready or anb status")
            })?
    } else {
        let id = request
            .id
            .as_deref()
            .or(resumed.as_deref())
            .or_else(|| state.as_ref().and_then(|state| state.focus.as_deref()))
            .ok_or_else(|| {
                invalid(
                    "start: name a Task, pass --next, or resume a session with remembered focus",
                )
            })?;
        notebook.start(id, today)?
    };
    let joined = joining(
        &claims,
        &transition.id,
        state.as_ref().and_then(|state| state.focus.as_deref()),
        request.join,
    )?;
    let path =
        task_path(&transition.id).ok_or_else(|| invalid("start: the result is not a Task id"))?;
    let before = storage.read(&path)?;
    let after = staged
        .write
        .map_or_else(|| before.clone(), |(_, content)| content);
    if let Some(state) = &mut state {
        if state.focus.as_deref() != Some(&transition.id) || before != after {
            state.pending = Some(Pending {
                task: transition.id.clone(),
                today: today.to_owned(),
                before,
                after,
            });
            persist(storage, state)?;
            reconcile(storage, state)?;
        }
    } else if before != after {
        storage.write(&path, &after)?;
    }
    Ok(Started {
        transition,
        session: session.map(str::to_owned),
        joined,
    })
}

/// One start intention. `next` selects from the ready queue; otherwise
/// `id` names the Task or the current session supplies it.
#[derive(Debug, Default)]
pub struct Start {
    pub id: Option<String>,
    pub next: bool,
    pub hub: Option<String>,
    pub join: bool,
}

fn active_focus(
    notebook: &Notebook<'_>,
    state: Option<&State>,
    hub: Option<&str>,
) -> Result<Option<String>, NotebookError> {
    let Some(id) = state.and_then(|state| state.focus.as_deref()) else {
        return Ok(None);
    };
    let record = match notebook.record(id) {
        Ok(record) => record,
        Err(NotebookError::UnknownId { .. }) => return Ok(None),
        Err(error) => return Err(error),
    };
    if record.state() != Some("active") || record.hold().is_some() {
        return Ok(None);
    }
    if let Some(hub) = hub {
        let rows = notebook.list(&Filter {
            hub: Some(hub.to_owned()),
            ..Filter::default()
        })?;
        if !rows.iter().any(|row| row.id == id) {
            return Ok(None);
        }
    }
    Ok(Some(id.to_owned()))
}

fn reconcile(storage: &mut dyn Storage, state: &mut State) -> Result<(), NotebookError> {
    let Some(pending) = state.pending.as_ref() else {
        return Ok(());
    };
    let path = task_path(&pending.task)
        .ok_or_else(|| recovery(&state.session, "the pending Task id is invalid"))?;
    let current = storage
        .read(&path)
        .map_err(|error| recovery(&state.session, &error.to_string()))?;
    if current != pending.after {
        if current != pending.before {
            return Err(recovery(
                &state.session,
                "the Task changed after the interrupted start; preserve the local session file and inspect the Task before recovering",
            ));
        }
        let mut staged = Staged::new(storage);
        Notebook::new(&mut staged)
            .with_identity(state.identity.as_deref())
            .start(&pending.task, &pending.today)
            .map_err(|error| {
                recovery(
                    &state.session,
                    &format!("the pending start is no longer valid: {error}"),
                )
            })?;
        let after = staged
            .write
            .map_or_else(|| current.clone(), |(_, content)| content);
        if after != pending.after {
            return Err(recovery(
                &state.session,
                "the pending start does not match the record change; inspect the local session file",
            ));
        }
        storage
            .write(&path, &pending.after)
            .map_err(|error| recovery(&state.session, &error.to_string()))?;
    }
    let focus = pending.task.clone();
    state.pending = None;
    state.focus = Some(focus);
    persist(storage, state)
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Claim {
    task: String,
    session: String,
    pending: bool,
}

fn joining(
    claims: &[Claim],
    task: &str,
    own_focus: Option<&str>,
    explicit: bool,
) -> Result<bool, NotebookError> {
    if let Some(claim) = claims
        .iter()
        .find(|claim| claim.task == task && claim.pending)
    {
        return Err(recovery(
            &claim.session,
            "finish the interrupted start in this session before joining its Task",
        ));
    }
    let other = claims.iter().find(|claim| claim.task == task);
    if let Some(claim) = other
        && !explicit
        && own_focus != Some(task)
    {
        return Err(NotebookError::SessionConflict {
            id: claim.task.clone(),
            session: claim.session.clone(),
        });
    }
    Ok(other.is_some())
}

fn claims(storage: &dyn Storage, own: Option<&str>) -> Result<Vec<Claim>, NotebookError> {
    let mut claims = Vec::new();
    for path in storage
        .list(DIRECTORY)
        .map_err(|error| unknown_session(DIRECTORY, &error.to_string()))?
    {
        if std::path::Path::new(&path)
            .extension()
            .is_none_or(|extension| extension != "json")
        {
            continue;
        }
        let text = storage
            .read(&path)
            .map_err(|error| unknown_session(&path, &error.to_string()))?;
        let state = decode(&path, &text)?;
        if Some(state.session.as_str()) == own {
            continue;
        }
        if let Some(task) = state.focus {
            claims.push(Claim {
                task,
                session: state.session.clone(),
                pending: false,
            });
        }
        if let Some(pending) = state.pending {
            claims.push(Claim {
                task: pending.task,
                session: state.session,
                pending: true,
            });
        }
    }
    claims.sort();
    claims.dedup();
    Ok(claims)
}

fn read(storage: &dyn Storage, session: &str) -> Result<Option<State>, NotebookError> {
    let path = session_path(session);
    match storage.read(&path) {
        Ok(text) => decode(&path, &text).map(Some).map_err(|error| match error {
            NotebookError::SessionRecovery { reason, .. } => recovery(session, &reason),
            error => error,
        }),
        Err(StorageError::NotFound { .. }) => Ok(None),
        Err(error) => Err(recovery(session, &error.to_string())),
    }
}

fn decode(path: &str, text: &str) -> Result<State, NotebookError> {
    let state: State = serde_json::from_str(text).map_err(|error| {
        unknown_session(
            path,
            &format!("invalid session JSON: {error}; preserve and inspect this local file"),
        )
    })?;
    if session_path(&state.session) != path {
        return Err(unknown_session(
            path,
            "the session filename does not match its contents; preserve and inspect this local file",
        ));
    }
    if state.version != 1 {
        return Err(recovery(
            &state.session,
            "the session format is not supported",
        ));
    }
    validate_name(&state.session).map_err(|error| recovery(&state.session, &error.to_string()))?;
    if state
        .focus
        .as_deref()
        .is_some_and(|id| task_path(id).is_none())
    {
        return Err(recovery(
            &state.session,
            "the remembered Task id is invalid",
        ));
    }
    if let Some(pending) = &state.pending {
        let path = task_path(&pending.task)
            .ok_or_else(|| recovery(&state.session, "the pending Task id is invalid"))?;
        for text in [&pending.before, &pending.after] {
            let record = Record::parse(&path, text);
            if !record.error_findings().is_empty() {
                return Err(recovery(
                    &state.session,
                    "the pending start contains an invalid Task record",
                ));
            }
        }
    }
    Ok(state)
}

fn persist(storage: &mut dyn Storage, state: &State) -> Result<(), NotebookError> {
    let text = serde_json::to_string(state)
        .map_err(|error| recovery(&state.session, &error.to_string()))?;
    storage
        .write(&session_path(&state.session), &text)
        .map_err(|error| recovery(&state.session, &error.to_string()))
}

fn guard_identity(state: &State, identity: Option<&str>) -> Result<(), NotebookError> {
    if state.identity.as_deref() != identity {
        return Err(recovery(
            &state.session,
            "this session belongs to a different identity; select another session id",
        ));
    }
    Ok(())
}

fn session_path(session: &str) -> String {
    let mut hash = String::with_capacity(64);
    for byte in Sha256::digest(session.as_bytes()) {
        let _ = write!(hash, "{byte:02x}");
    }
    format!("{DIRECTORY}/{hash}.json")
}

fn task_path(id: &str) -> Option<String> {
    let slug = id.strip_prefix("task.")?;
    (!slug.is_empty()
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'))
    .then(|| format!("tasks/{id}.md"))
}

fn validate_name(session: &str) -> Result<(), NotebookError> {
    if session.trim().is_empty() || session.chars().any(char::is_control) {
        return Err(invalid(
            "session: use a nonempty id without control characters",
        ));
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> NotebookError {
    NotebookError::InvalidArgument {
        reason: reason.into(),
    }
}
fn recovery(session: &str, reason: &str) -> NotebookError {
    NotebookError::SessionRecovery {
        session: Some(session.to_owned()),
        path: session_path(session),
        reason: reason.to_owned(),
    }
}

fn unknown_session(path: &str, reason: &str) -> NotebookError {
    NotebookError::SessionRecovery {
        session: None,
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}

/// Core validates and prepares the Task change before the session intent
/// is persisted. Starting a Task writes at most that one existing file.
struct Staged<'a> {
    storage: &'a dyn Storage,
    write: Option<(String, String)>,
}

impl<'a> Staged<'a> {
    fn new(storage: &'a dyn Storage) -> Self {
        Self {
            storage,
            write: None,
        }
    }
}

impl Storage for Staged<'_> {
    fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
        self.storage.list(dir)
    }
    fn read(&self, path: &str) -> Result<String, StorageError> {
        if let Some((written, content)) = &self.write
            && written == path
        {
            return Ok(content.clone());
        }
        self.storage.read(path)
    }
    fn write(&mut self, path: &str, content: &str) -> Result<(), StorageError> {
        if self.write.is_some() {
            return Err(StorageError::Io {
                path: path.to_owned(),
                detail: "a Task start attempted more than one record write".to_owned(),
            });
        }
        self.write = Some((path.to_owned(), content.to_owned()));
        Ok(())
    }
    fn remove(&mut self, path: &str) -> Result<(), StorageError> {
        Err(StorageError::Io {
            path: path.to_owned(),
            detail: "a Task start attempted to remove a record".to_owned(),
        })
    }
}
