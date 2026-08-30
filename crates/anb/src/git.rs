//! What the host asks git, and the deadline it asks under.
//!
//! Both answers are conveniences: an identity to sign a new record with,
//! and whether a cited commit still exists. Neither is worth waiting on
//! without end — a writer asks with the notebook locked, and the session
//! hook asks under a contract to fail soft — so a git that does not answer
//! in time is killed and has answered nothing.

use std::io::{Read as _, Write as _};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Long enough for any answer git reads off a local disk, short enough that
/// a wedged one is a pause and not a hang.
pub(crate) const DEADLINE: Duration = Duration::from_secs(2);

/// How often the deadline is checked. Short enough to cost a fast answer
/// almost nothing, long enough not to spin.
const POLL: Duration = Duration::from_millis(2);

/// The accountable identity, as git knows it; absence is legal.
#[must_use]
pub fn user_name() -> Option<String> {
    let bytes = answered(
        Command::new("git").args(["config", "--get", "user.name"]),
        String::new(),
    )?;
    let name = String::from_utf8(bytes).ok()?;
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// What `command` writes to stdout when it is given `query` and finishes
/// successfully inside [`DEADLINE`]; `None` for every other outcome,
/// including a child that ran out of time and was killed for it.
///
/// The query and the answer travel on their own threads because git stops
/// reading once its own output backs up: sharing a thread with either one
/// is how the two block forever.
pub(crate) fn answered(command: &mut Command, query: String) -> Option<Vec<u8>> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut asking = child.stdin.take()?;
    std::thread::spawn(move || asking.write_all(query.as_bytes()));
    let mut answering = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = answering.read_to_end(&mut bytes);
        bytes
    });

    let expires = Instant::now() + DEADLINE;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return status.success().then(|| reader.join().unwrap_or_default());
            }
            Ok(None) if Instant::now() < expires => std::thread::sleep(POLL),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_that_finishes_in_time_answers_what_it_wrote() {
        let bytes = answered(&mut Command::new("cat"), "the query\n".to_owned()).unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), "the query\n");
    }

    #[test]
    fn a_command_that_fails_answers_nothing() {
        assert_eq!(answered(&mut Command::new("false"), String::new()), None);
    }

    #[test]
    fn a_command_that_is_not_there_answers_nothing() {
        assert_eq!(
            answered(&mut Command::new("anb-no-such-program"), String::new()),
            None
        );
    }

    /// A wedged git must not outlive the call that asked it: an agent
    /// session runs hundreds of these.
    #[test]
    fn a_command_that_runs_out_of_time_is_killed() {
        let asked = Instant::now();
        assert_eq!(
            answered(Command::new("sh").args(["-c", "sleep 30"]), String::new()),
            None
        );
        assert!(asked.elapsed() < DEADLINE * 2, "{:?}", asked.elapsed());
    }
}
