//! The mutation lock: the window in which one process owns the notebook.
//!
//! Every verb that writes reads the records it needs, splices them, and
//! writes them back, and a settling verb moves several files in turn. Two
//! runs that overlap on that window read the same bytes and the second
//! write erases the first, or a reader lands between two files of one move
//! and reports the half-finished state as corruption. Both ran to a
//! successful exit.
//!
//! So a writer takes the notebook exclusively and a reader shares it. The
//! wait is bounded by the writers ahead of you, each of them one mutation
//! long, and nothing but `anb` ever takes this lock.

use crate::cli::Command;
use crate::fs_storage::{LOCK_FILE, ignore_leavings};
use anb_core::StorageError;
use std::fs::{self, File, OpenOptions};
use std::path::Path;

/// A claim on one notebook, held for as long as this value lives. Dropping
/// it releases the notebook; so does the process ending, however it ends,
/// because the kernel — not a file the next run must clean up — holds the
/// claim.
pub struct Lock {
    _file: File,
}

/// The claim `command` needs on the notebook at `root`, or `None` when it
/// needs none.
///
/// A writer's claim is exclusive. The notebook root appears here when the
/// mutation is one that could create it, so two first-ever `add`s still
/// meet on a lock file; a verb that needs a record the notebook does not
/// hold cannot write whatever its arguments say, so it takes no claim and
/// leaves no directory behind when it is refused.
///
/// A reader's claim is shared, and it is taken only if the lock file is
/// already there. Nothing on a read path creates state, so a notebook
/// mounted read-only stays readable.
///
/// # Errors
/// [`StorageError::Io`] when a writer cannot take its claim. A medium that
/// reports no error and still fails to serialize — some network
/// filesystems — is beyond what any caller can detect.
pub fn taken(root: &Path, command: &Command) -> Result<Option<Lock>, StorageError> {
    if !writes(command) {
        return Ok(shared(root));
    }
    if !root.is_dir() && !creates(command) {
        return Ok(None);
    }
    fs::create_dir_all(root).map_err(|error| io_error(&root.display().to_string(), &error))?;
    let file = opened(root).map_err(|error| io_error(LOCK_FILE, &error))?;
    file.lock().map_err(|error| io_error(LOCK_FILE, &error))?;
    ignore_leavings(root);
    Ok(Some(Lock { _file: file }))
}

/// A reader's turn, when there is a lock file to take one on. A reader that
/// cannot take it reads anyway: nothing it does can lose a write, and
/// refusing to read a notebook is worse than reading it a moment early.
fn shared(root: &Path) -> Option<Lock> {
    let file = File::open(root.join(LOCK_FILE)).ok()?;
    file.lock_shared().ok()?;
    Some(Lock { _file: file })
}

fn opened(root: &Path) -> std::io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join(LOCK_FILE))
}

/// Whether the command could write a record the notebook does not hold —
/// the only way a notebook comes into being. Every other verb acts on a
/// record that must already be there, so against a notebook that does not
/// exist it can only be refused.
fn creates(command: &Command) -> bool {
    matches!(command, Command::Add(_))
}

/// Whether the command writes. A writing verb reads the records it needs,
/// splices them, and writes them back, so it holds the notebook alone for
/// that whole window; a reading verb only shares it.
///
/// A refused write counts: the verb is classified before its arguments are
/// judged, and the judgement itself reads the notebook.
fn writes(command: &Command) -> bool {
    match command {
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

fn io_error(path: &str, error: &std::io::Error) -> StorageError {
    StorageError::Io {
        path: path.to_owned(),
        detail: error.to_string(),
    }
}
