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
use anb_core::storage::StorageError;
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
/// A writer's claim is exclusive, and the root appears here when the
/// mutation is the call that creates the notebook — so a refused first
/// command leaves an empty notebook where a successful one would have left
/// a full one, and later commands typed below that directory resolve to it.
///
/// A reader's claim is shared, and it is taken only if the lock file is
/// already there. Nothing creates it: a notebook no writer has ever touched
/// has no one to wait for, and a notebook mounted read-only stays readable.
///
/// # Errors
/// [`StorageError::Io`] when a writer cannot take its claim. A medium that
/// reports no error and still fails to serialize — some network
/// filesystems — is beyond what any caller can detect.
pub fn taken(root: &Path, command: &Command) -> Result<Option<Lock>, StorageError> {
    if !writes(command) {
        return Ok(shared(root));
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
        | Command::Return { .. }
        | Command::Reopen { .. }
        | Command::Hold { .. }
        | Command::Unhold { .. }
        | Command::Block { .. }
        | Command::Unblock { .. }
        | Command::Comment { .. }
        | Command::Decide(_)
        | Command::Note(_)
        | Command::Ask(_)
        | Command::Answer { .. }
        | Command::Retire { .. }
        | Command::Archive { .. }
        | Command::Expunge { .. }
        | Command::Edit(_) => true,
        Command::Ready { .. }
        | Command::List { .. }
        | Command::View { .. }
        | Command::Status { .. }
        | Command::Check { .. }
        | Command::Search { .. }
        | Command::Overview { .. } => false,
    }
}

fn io_error(path: &str, error: &std::io::Error) -> StorageError {
    StorageError::Io {
        path: path.to_owned(),
        detail: error.to_string(),
    }
}
