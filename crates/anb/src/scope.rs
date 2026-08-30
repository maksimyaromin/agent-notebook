//! Which notebook a call acts on, and what may be done there.
//!
//! The user's notebook holds knowledge that outlives a repository:
//! decisions and notes, and nothing to work on. So a verb that can only
//! create or move a task or a question is refused the global scope — work
//! is always project work. Every other verb reads or writes whatever the
//! notebook it is pointed at happens to hold, and behaves the same in
//! either scope. Where the two roots sit on disk is `fs_storage`'s.

use crate::cli::Command;
use crate::recovery::subject;
use anb_core::NotebookError;

/// Why `command` cannot act on the user's notebook, when `global` names it
/// and the verb has nothing to reach there.
#[must_use]
pub fn refused_globally(command: &Command, global: bool) -> Option<NotebookError> {
    if !global || !writes_work(command) {
        return None;
    }
    let verb = subject(command).verb;
    Some(NotebookError::InvalidArgument {
        reason: format!(
            "{verb}: the user's notebook holds decisions and notes — work stays in the project"
        ),
    })
}

/// Whether the verb can only create or move a task or a question, and so
/// can act on nothing the user's notebook holds. Matched whole, so a verb
/// added later is placed rather than assumed.
fn writes_work(command: &Command) -> bool {
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
        | Command::Ask(_)
        | Command::Answer { .. } => true,
        Command::Decide(_)
        | Command::Note(_)
        | Command::Retire { .. }
        | Command::View { .. }
        | Command::List { .. }
        | Command::Search { .. }
        | Command::Ready { .. }
        | Command::Check { .. }
        | Command::Archive { .. }
        | Command::Expunge { .. }
        | Command::Edit(_)
        | Command::Overview { .. }
        | Command::Status { .. } => false,
    }
}
