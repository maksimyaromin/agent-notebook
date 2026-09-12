//! The refusals: one variant per outcome a caller tells apart, and the
//! words each carries.

use crate::encode;
use crate::finding::Finding;
use crate::record::RecordType;
use crate::reply::{Blocker, carriers_of};
use crate::status::counted;
use crate::storage::StorageError;

/// One variant per outcome a caller tells apart; the CLI turns each into a
/// structured `error[code]` payload with computed `try:` lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotebookError {
    /// No live or archived record carries this id.
    UnknownId {
        id: String,
    },
    /// The record exists only in the archive; archived records are read,
    /// never mutated in place — `restore` moves one back.
    Archived {
        id: String,
    },
    /// The record carries error findings, which exclude it from mutation.
    InvalidRecord {
        path: String,
        findings: Vec<Finding>,
    },
    /// The id names a type this command does not act on.
    WrongType {
        id: String,
        expected: String,
    },
    /// Another identity took the Task: `start` takes work, and a Task
    /// already taken changes hands through `edit --taken-by` first.
    Taken {
        id: String,
        taken_by: String,
    },
    /// Another local session is working on this Task.
    SessionConflict {
        id: String,
        session: String,
    },
    /// Local session state needs recovery before another operation is safe.
    SessionRecovery {
        session: Option<String>,
        path: String,
        reason: String,
    },
    /// The record's state does not allow this move; `valid` names the
    /// moves it does allow, each the word its retry command is keyed by.
    InvalidTransition {
        id: String,
        state: String,
        valid: Vec<&'static str>,
    },
    /// Successful completion still depends on unfinished Tasks.
    UnfinishedDependencies {
        id: String,
        blockers: Vec<String>,
    },
    /// An argument is malformed before any record is touched.
    InvalidArgument {
        reason: String,
    },
    /// The id is taken; ids are never reused, archive included.
    DuplicateId {
        id: String,
        holder: String,
    },
    /// A delete would leave the notebook pointing at nothing. Every
    /// blocker is named, since repairing them is the whole path forward.
    StillReferenced {
        id: String,
        blockers: Vec<Blocker>,
    },
    /// A reference argument names a record that does not exist.
    DanglingRef {
        field: &'static str,
        target: String,
    },
    /// The supersession victim cannot die by supersession.
    CannotSupersede {
        id: String,
        reason: String,
    },
    /// The edge would close a dependency cycle; `chain` walks it,
    /// first and last the same Task.
    WouldCycle {
        chain: Vec<String>,
    },
    Storage(StorageError),
}

impl NotebookError {
    /// The stable kebab-case code a caller keys on.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            NotebookError::UnknownId { .. } => "unknown-id",
            NotebookError::Archived { .. } => "archived",
            NotebookError::InvalidRecord { .. } => "invalid-record",
            NotebookError::WrongType { .. } => "wrong-type",
            NotebookError::Taken { .. } => "taken",
            NotebookError::SessionConflict { .. } => "session-conflict",
            NotebookError::SessionRecovery { .. } => "session-recovery",
            NotebookError::InvalidTransition { .. } => "invalid-transition",
            NotebookError::UnfinishedDependencies { .. } => "unfinished-dependencies",
            NotebookError::InvalidArgument { .. } => "invalid-argument",
            NotebookError::DuplicateId { .. } => "duplicate-id",
            NotebookError::StillReferenced { .. } => "still-referenced",
            NotebookError::DanglingRef { .. } => "dangling-ref",
            NotebookError::CannotSupersede { .. } => "cannot-supersede",
            NotebookError::WouldCycle { .. } => "would-cycle",
            // The command-level code and the Check finding share one
            // vocabulary.
            NotebookError::Storage(StorageError::NotUtf8 { .. }) => "not-utf8",
            NotebookError::Storage(_) => "storage",
        }
    }
}

impl std::fmt::Display for NotebookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotebookError::UnknownId { id } => write!(f, "no record `{id}`"),
            NotebookError::Archived { id } => write!(f, "`{id}` is archived"),
            NotebookError::InvalidRecord { path, findings } => {
                write!(
                    f,
                    "{path} is invalid ({})",
                    counted(findings.len(), "finding")
                )
            }
            NotebookError::WrongType { id, expected } => {
                write!(f, "`{id}` is not {expected}")
            }
            NotebookError::Taken { id, taken_by } => {
                write!(f, "`{id}` is taken by {taken_by}")
            }
            NotebookError::SessionConflict { id, session } => {
                write!(
                    f,
                    "`{id}` is in use by local session `{session}`; use --join to work on it together"
                )
            }
            NotebookError::SessionRecovery {
                session,
                path,
                reason,
            } => {
                if let Some(session) = session {
                    write!(
                        f,
                        "local session `{session}` at `{path}` needs recovery: {reason}"
                    )
                } else {
                    write!(
                        f,
                        "local session state at `{path}` needs recovery: {reason}"
                    )
                }
            }
            NotebookError::InvalidTransition { id, state, valid } => {
                if valid.is_empty() {
                    write!(f, "`{id}` is {state}; no move is valid from `{state}`")
                } else {
                    write!(f, "`{id}` is {state}; valid: {}", valid.join(", "))
                }
            }
            NotebookError::InvalidArgument { reason } => f.write_str(reason),
            NotebookError::UnfinishedDependencies { id, .. } => {
                write!(
                    f,
                    "cannot complete `{id}` while dependencies remain unfinished"
                )
            }
            NotebookError::DuplicateId { id, holder } => {
                write!(f, "`{id}` already exists at {holder}")
            }
            NotebookError::StillReferenced { id, blockers } => {
                let records = counted(carriers_of(blockers).count(), "record");
                write!(f, "`{id}` is still referenced by {records}")
            }
            NotebookError::DanglingRef { field, target } => {
                write!(f, "{field}: `{target}` names no record")
            }
            NotebookError::CannotSupersede { id, reason } => {
                write!(f, "cannot supersede `{id}`: {reason}")
            }
            NotebookError::WouldCycle { chain } => {
                write!(
                    f,
                    "the edge would close a dependency cycle: {}",
                    encode::id_chain(chain)
                )
            }
            NotebookError::Storage(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for NotebookError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NotebookError::Storage(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for NotebookError {
    fn from(error: StorageError) -> Self {
        NotebookError::Storage(error)
    }
}

pub(super) fn settled_question(id: &str, state: &str) -> NotebookError {
    NotebookError::InvalidTransition {
        id: id.to_owned(),
        state: state.to_owned(),
        valid: Vec::new(),
    }
}

pub(super) fn type_list(types: &[RecordType]) -> String {
    let words: Vec<String> = types
        .iter()
        .map(|record_type| format!("a {}", record_type.word()))
        .collect();
    words.join(" or ")
}

/// The commands that carry a still-live record toward its settled state —
/// the recovery payload of an archive refused too early.
pub(super) fn settling_commands(record_type: RecordType, state: &str) -> Vec<&'static str> {
    match (record_type, state) {
        (RecordType::Task, "open") => vec!["start"],
        (RecordType::Task | RecordType::Question, _) => vec!["close"],
        (RecordType::Decision | RecordType::Note, _) => vec!["retire"],
    }
}
