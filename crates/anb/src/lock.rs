//! Readers share one notebook lock; writers hold it exclusively across
//! their complete read-modify-write operation. The operating system
//! releases the lock when its handle closes or its process exits.
//!
//! A read never creates a lock file. An absent lock allows a read-only
//! notebook to be inspected; a lock failure is a storage error.

use crate::cli::Command;
use crate::fs_storage::{LOCK_FILE, ignore_leavings};
use anb_core::StorageError;
use std::fs::{self, File, OpenOptions};
use std::io::ErrorKind;
use std::path::Path;

/// A lock on one notebook, released when this value drops or the process exits.
pub struct Lock {
    _file: File,
}

/// Acquire the lock required by `command`. Creation commands may create
/// the notebook root; other writes against a missing root leave it absent.
/// Reads take a shared lock only when its file already exists.
///
/// # Errors
/// [`StorageError::Io`] when a lock file cannot be opened or locked.
/// Filesystems that do not honor operating-system locks cannot provide
/// cross-process serialization, even when their lock calls succeed.
pub fn taken(root: &Path, command: &Command) -> Result<Option<Lock>, StorageError> {
    if !writes(command) {
        return shared(root);
    }
    if !root.is_dir() && !creates(command) {
        return Ok(None);
    }
    crate::fs_storage::create_directory_tree(root).map_err(|error| io_error(root, &error))?;
    let path = root.join(LOCK_FILE);
    let file = opened(&path).map_err(|error| io_error(&path, &error))?;
    file.lock().map_err(|error| io_error(&path, &error))?;
    ignore_leavings(root);
    Ok(Some(Lock { _file: file }))
}

/// Acquire a shared lock without creating files. Only an absent lock returns `None`.
///
/// # Errors
/// [`StorageError::Io`] when an existing lock is not a regular file,
/// cannot be read, or cannot be locked.
pub fn shared(root: &Path) -> Result<Option<Lock>, StorageError> {
    let path = root.join(LOCK_FILE);
    guard_lock_file(&path).map_err(|error| io_error(&path, &error))?;
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(&path, &error)),
    };
    file.lock_shared()
        .map_err(|error| io_error(&path, &error))?;
    Ok(Some(Lock { _file: file }))
}

fn opened(path: &Path) -> std::io::Result<File> {
    guard_lock_file(path)?;
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
}

fn guard_lock_file(path: &Path) -> std::io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "the notebook lock is not a regular file",
        )),
        Err(error) => Err(error),
    }
}

/// Commands that can create the first records may create the notebook root.
fn creates(command: &Command) -> bool {
    matches!(
        command,
        Command::Add(_) | Command::Import { check: false, .. }
    )
}

/// Writers hold the lock across validation and mutation. Readers share it.
///
/// Classification precedes validation, so even a refused write is locked.
fn writes(command: &Command) -> bool {
    match command {
        Command::Import { check, .. } | Command::Migrate { check } => !check,
        Command::Add(_)
        | Command::Start { .. }
        | Command::Submit { .. }
        | Command::Close(_)
        | Command::Reopen { .. }
        | Command::Hold { .. }
        | Command::Unhold { .. }
        | Command::Block { .. }
        | Command::Unblock { .. }
        | Command::Comment { .. }
        | Command::Retire { .. }
        | Command::Archive { .. }
        | Command::Restore { .. }
        | Command::Delete { .. }
        | Command::Edit(_) => true,
        Command::Ready { .. }
        | Command::Recall { .. }
        | Command::Hook
        | Command::List { .. }
        | Command::Show { .. }
        | Command::Status { .. }
        | Command::Graph(_)
        | Command::Check { .. }
        | Command::Debt { .. }
        | Command::Setup { .. }
        | Command::Skill { .. } => false,
    }
}

fn io_error(path: &Path, error: &std::io::Error) -> StorageError {
    StorageError::Io {
        path: path.display().to_string(),
        detail: error.to_string(),
    }
}
