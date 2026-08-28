//! The binary: wire the real world — cwd, clock, git identity — to the
//! shell and print one reply.

use anb::cli::{Cli, Command};
use anb::fs_storage::{FsStorage, resolve_root};
use anb::reply::execute;
use anb::{json, text};
use anb_core::NotebookError;
use anb_core::storage::StorageError;
use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(output) => {
            print!("{}", terminated(output));
            ExitCode::SUCCESS
        }
        Err(payload) => {
            eprint!("{}", terminated(payload));
            ExitCode::FAILURE
        }
    }
}

/// `Ok` is stdout; `Err` is the rendered recovery payload for stderr.
fn run(cli: Cli) -> Result<String, String> {
    let subject = anb::cli::subject(&cli.command);
    let hook = matches!(cli.command, Command::Status { hook: true, .. });
    let render_failure = |error: &NotebookError| {
        if cli.json {
            json::render_error(error, &subject)
        } else {
            text::render_error(error, &subject)
        }
    };

    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        // The hook's fail-soft covers the whole invocation, this wiring
        // included.
        Err(_) if hook => return Ok(String::new()),
        Err(error) => {
            return Err(render_failure(&NotebookError::Storage(StorageError::Io {
                path: ".".to_owned(),
                detail: error.to_string(),
            })));
        }
    };
    let mut storage = FsStorage::new(resolve_root(&cwd));
    let today = jiff::Zoned::now().date().to_string();

    match execute(cli.command, &mut storage, git_user_name, &today) {
        Ok(reply) => Ok(if cli.json {
            json::render(&reply)
        } else {
            text::render(&reply, &today)
        }),
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

/// The accountable identity, as git knows it; absence is legal.
fn git_user_name() -> Option<String> {
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
}
