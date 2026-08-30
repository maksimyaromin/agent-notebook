//! What the host asks git, and the deadline it asks under.
//!
//! Both answers are conveniences: an identity to sign a new record with,
//! and whether a cited commit still exists. Neither is worth waiting on
//! without end — a writer asks with the notebook locked, and the session
//! hook asks under a contract to fail soft — so a git that does not answer
//! in time has answered nothing.

use std::sync::mpsc;
use std::time::Duration;

/// Long enough for any answer git reads off a local disk, short enough that
/// a wedged one is a pause and not a hang.
pub(crate) const DEADLINE: Duration = Duration::from_secs(2);

/// The accountable identity, as git knows it; absence is legal.
#[must_use]
pub fn user_name() -> Option<String> {
    answered(DEADLINE, || {
        let output = std::process::Command::new("git")
            .args(["config", "--get", "user.name"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let name = String::from_utf8(output.stdout).ok()?;
        let name = name.trim();
        (!name.is_empty()).then(|| name.to_owned())
    })
    .flatten()
}

/// `work`'s answer, or `None` when `deadline` passes first.
///
/// The thread that outlived the deadline is left to its own end: nothing a
/// returning caller needs is bought by reaching back into it, and the
/// process it holds ends when this one does.
pub(crate) fn answered<T: Send + 'static>(
    deadline: Duration,
    work: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || sender.send(work()));
    receiver.recv_timeout(deadline).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_answer_within_the_deadline_is_the_answer() {
        assert_eq!(answered(Duration::from_secs(30), || "here"), Some("here"));
    }

    #[test]
    fn work_that_outlasts_the_deadline_answers_nothing() {
        assert_eq!(
            answered(Duration::from_millis(10), || {
                std::thread::sleep(Duration::from_secs(30));
                "too late"
            }),
            None
        );
    }
}
