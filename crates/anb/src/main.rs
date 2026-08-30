//! The binary: wire the real world — cwd, clock, git identity — to the
//! shell and print one reply.

use anb::cli::{Cli, Command};
use anb::fs_storage::{FsStorage, NOTEBOOK_ENV, notebook_root, unusable_root};
use anb::lock;
use anb::reconcile::lost_proofs;
use anb::reply::{Host, execute};
use anb::scope::refused_globally;
use anb::{json, text};
use anb_core::{NotebookError, StorageError};
use clap::Parser;
use std::io::{self, ErrorKind, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => return parse_refused(&error),
    };
    match run(cli) {
        Ok((output, exit)) => emitted(&mut io::stdout(), &terminated(output), exit),
        Err(payload) => emitted(&mut io::stderr(), &terminated(payload), ExitCode::FAILURE),
    }
}

/// Write one reply out and answer with the exit it earned.
///
/// A reader that stops early — `anb view <id> | head` — closes the pipe
/// mid-write. That is the reader's choice, not a failed command, and the
/// process ends on the verdict it had already reached. Any other write
/// failure is the host's to report.
fn emitted(stream: &mut impl Write, payload: &str, exit: ExitCode) -> ExitCode {
    match stream
        .write_all(payload.as_bytes())
        .and_then(|()| stream.flush())
    {
        Ok(()) => exit,
        Err(error) if error.kind() == ErrorKind::BrokenPipe => exit,
        Err(error) => {
            let _ = writeln!(io::stderr(), "anb: the reply could not be written: {error}");
            ExitCode::FAILURE
        }
    }
}

/// An unknown verb joins the recovery-payload contract; everything else
/// clap refuses (or serves, like `--help`) keeps clap's rendering.
fn parse_refused(error: &clap::Error) -> ExitCode {
    let Some(recovery) = anb::recovery::unknown_command_recovery(error) else {
        error.exit();
    };
    // The command line failed to parse, so the `--json` flag is read raw —
    // over OS strings, since an argument may not be UTF-8 at all.
    let payload = if std::env::args_os().any(|arg| arg == "--json") {
        json::render_recovery(&recovery)
    } else {
        text::render_recovery(&recovery)
    };
    emitted(&mut io::stderr(), &terminated(payload), ExitCode::FAILURE)
}

/// `Ok` is stdout with the reply's exit; `Err` is the rendered recovery
/// payload for stderr.
fn run(cli: Cli) -> Result<(String, ExitCode), String> {
    let subject = anb::recovery::subject(&cli.command);
    let hook = matches!(cli.command, Command::Status { hook: true, .. });
    let render_failure = |error: &NotebookError| {
        if cli.json {
            json::render_error(error, &subject)
        } else {
            text::render_error(error, &subject)
        }
    };

    // A session starts whatever state the notebook is in, so the hook's
    // fail-soft reaches the wiring below as well as the verb: everything
    // between here and `execute` can fail before a reply exists to soften.
    let stopped = |error: &NotebookError| -> Result<(String, ExitCode), String> {
        if hook {
            return Ok((String::new(), ExitCode::SUCCESS));
        }
        Err(render_failure(error))
    };

    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            return stopped(&NotebookError::Storage(StorageError::Io {
                path: ".".to_owned(),
                detail: error.to_string(),
            }));
        }
    };
    let root = match notebook_root(
        &cwd,
        cli.notebook.as_deref(),
        cli.global,
        std::env::home_dir().as_deref(),
        std::env::var_os(NOTEBOOK_ENV).as_deref(),
    ) {
        Ok(root) => root,
        Err(reason) => return stopped(&NotebookError::InvalidArgument { reason }),
    };
    if let Some(refusal) = refused_globally(&cli.command, cli.global) {
        return stopped(&refusal);
    }
    if let Some(reason) = unusable_root(&root) {
        return stopped(&NotebookError::InvalidArgument { reason });
    }
    let mut storage = FsStorage::new(root.clone());
    let today = jiff::Zoned::now().date().to_string();

    // Bound to a name, so the claim lives until the command has answered;
    // `let _ =` would release it before the first read.
    let _lock = match lock::taken(&root, &cli.command) {
        Ok(lock) => lock,
        Err(error) => return stopped(&NotebookError::Storage(error)),
    };

    let lost = |cited: &[anb_core::CitedProof]| lost_proofs(&root, cited);
    let host = Host {
        git_by: anb::git::user_name,
        read_report: &read_report,
        lost_proofs: &lost,
        today: &today,
    };
    match execute(cli.command, &mut storage, host) {
        Ok(reply) => {
            let exit = if reply.failed() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            };
            let output = if cli.json {
                json::render(&reply)
            } else {
                text::render(&reply, &today)
            };
            Ok((output, exit))
        }
        Err(error) => Err(render_failure(&error)),
    }
}

/// The compact-JSON renderings carry no newline of their own; the terminal
/// still gets one.
fn terminated(mut output: String) -> String {
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// A report the caller named by path, read from wherever the work left it:
/// outside the notebook root as often as in it, so this is the shell's
/// read, not Storage's.
fn read_report(path: &str) -> Result<String, StorageError> {
    std::fs::read_to_string(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => StorageError::NotFound {
            path: path.to_owned(),
        },
        std::io::ErrorKind::InvalidData => StorageError::NotUtf8 {
            path: path.to_owned(),
        },
        _ => StorageError::Io {
            path: path.to_owned(),
            detail: error.to_string(),
        },
    })
}
