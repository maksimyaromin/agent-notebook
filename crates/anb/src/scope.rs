//! Which notebook a call acts on, and what may be done there.
//!
//! Personal and global notebooks hold private Decisions and Notes.
//! Tasks and Questions belong to the shared project notebook, so commands
//! that only create or move work are refused in either private scope.
//! Knowledge commands use the selected notebook without changing their
//! behavior. `fs_storage` resolves the three audience roots.

use crate::cli::Command;
use crate::recovery::subject;
use anb_core::{NotebookError, RecordType};

/// Why `command` cannot act on a personal or global notebook when
/// `private` selects either audience.
#[must_use]
pub fn refused_privately(command: &Command, private: bool) -> Option<NotebookError> {
    if !private {
        return None;
    }
    if matches!(command, Command::Setup { .. }) {
        return Some(NotebookError::InvalidArgument {
            reason: "setup: installs into the project; personal and global notebooks need no separate hook".to_owned(),
        });
    }
    if !writes_work(command) {
        return None;
    }
    let verb = subject(command).verb;
    Some(NotebookError::InvalidArgument {
        reason: format!(
            "{verb}: personal and global notebooks hold Decisions and Notes; work stays in the project"
        ),
    })
}

/// Whether the call can only create or move a task or a question, and so
/// can act on nothing a private notebook holds. Matched whole, so a verb
/// added later is placed rather than assumed; `add` is judged by the type
/// it creates.
fn writes_work(command: &Command) -> bool {
    match command {
        Command::Add(args) => matches!(args.record_type, RecordType::Task | RecordType::Question),
        Command::Start { .. }
        | Command::Submit { .. }
        | Command::Close(_)
        | Command::Reopen { .. }
        | Command::Hold { .. }
        | Command::Unhold { .. }
        | Command::Block { .. }
        | Command::Unblock { .. } => true,
        Command::Comment { .. }
        | Command::Import { .. }
        | Command::Migrate { .. }
        | Command::Recall { .. }
        | Command::Hook
        | Command::Retire { .. }
        | Command::Show { .. }
        | Command::List { .. }
        | Command::Ready { .. }
        | Command::Check { .. }
        | Command::Debt { .. }
        | Command::Archive { .. }
        | Command::Restore { .. }
        | Command::Delete { .. }
        | Command::Edit(_)
        | Command::Graph(_)
        | Command::Status { .. }
        | Command::Setup { .. }
        | Command::Skill { .. } => false,
    }
}
