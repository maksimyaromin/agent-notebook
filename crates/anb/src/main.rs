//! The binary: wire the real world — cwd, clock, identity — to the shell
//! and print one reply.

mod actions;
mod ids;
mod locations;

use anb::cli::{Cli, Command};
use anb::fs_storage::FsStorage;
use anb::lock;
use anb::reconcile::lost_proofs;
use anb::reply::{Host, execute};
use anb::scope::refused_privately;
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
/// A reader that stops early — `anb show <id> | head` — closes the pipe
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

/// Parser failures use the same recovery payload as command failures.
/// Explicit help and version requests keep clap's rendering.
fn parse_refused(error: &clap::Error) -> ExitCode {
    let Some(recovery) = anb::recovery::parse_recovery(error) else {
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
    let native = matches!(cli.command, Command::Hook);
    let json_output = cli.json;
    let actions = actions::Actions::new(&cli);
    let private_refusal = refused_privately(&cli.command, cli.global || cli.personal);
    let result = match private_refusal {
        Some(ref error) => Err(error.clone()),
        None => invoked(cli, &actions),
    };
    match result {
        Ok(answer) => Ok(answer),
        Err(error) => {
            let mut document =
                json::recovery_value(&anb::recovery::Recovery::new(&error, &subject));
            if private_refusal.is_none() {
                actions.qualify(&mut document);
            }
            if native {
                return Ok((anb::hook::payload(&document), ExitCode::SUCCESS));
            }
            Err(if json_output {
                document.to_string()
            } else {
                json::toon(&document)
            })
        }
    }
}

fn invoked(mut cli: Cli, actions: &actions::Actions) -> Result<(String, ExitCode), NotebookError> {
    let native = matches!(cli.command, Command::Hook);
    let session = if native {
        anb::hook::session(cli.session.clone())?
    } else {
        anb::session::resolve(cli.session.clone())?
    };
    let locations = locations::Locations::resolve(&cli)?;
    let mut storage = FsStorage::new(locations.root.clone());
    let user = locations
        .global
        .as_ref()
        .map(|path| FsStorage::new(path.clone()));
    let personal = locations
        .personal
        .as_ref()
        .map(|path| FsStorage::new(path.clone()));
    // Named guards keep every participating notebook locked through rendering.
    let _lock = lock::taken(&locations.root, &cli.command)?;
    let _recall_locks = locations
        .global
        .iter()
        .chain(locations.personal.iter())
        .map(|path| lock::shared(path))
        .collect::<Result<Vec<_>, _>>()?;
    let today = jiff::Zoned::now().date().to_string();
    let lost = |cited: &[anb_core::CitedProof]| lost_proofs(&locations.root, cited);
    let host = Host {
        session: session.as_deref(),
        identity: anb::identity::name,
        read_file: &read_file,
        lost_proofs: &lost,
        user_notebook: user.as_ref().map(|source| source as &dyn anb_core::Storage),
        personal_notebook: personal
            .as_ref()
            .map(|source| source as &dyn anb_core::Storage),
        audience: locations.audience,
        project_dir: &locations.cwd,
        today: &today,
    };
    ids::assign(&mut cli.command)?;
    let reply = execute(cli.command, &mut storage, host)?;
    let exit = if reply.failed() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    };
    if !cli.json
        && let anb::reply::Reply::Skill(anb::reply::SkillReply::Printed(skill)) = &reply
    {
        return Ok((skill.clone(), exit));
    }
    let document = json::value_with(&reply, |document| {
        actions.qualify(document);
        if native {
            document["session"] = serde_json::json!(session);
        }
    });
    let output = if native {
        anb::hook::payload(&document)
    } else if cli.json {
        document.to_string()
    } else {
        json::toon(&document)
    };
    Ok((output, exit))
}

/// The compact-JSON renderings carry no newline of their own; the terminal
/// still gets one.
fn terminated(mut output: String) -> String {
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// A file the caller named by path, read from wherever the work left it:
/// outside the notebook root as often as in it, so this is the shell's
/// read, not Storage's. `-` reads standard input instead, so a body or a
/// report can arrive from a pipe.
fn read_file(path: &str) -> Result<String, StorageError> {
    let read = if path == "-" {
        io::read_to_string(io::stdin())
    } else {
        std::fs::read_to_string(path)
    };
    read.map_err(|error| match error.kind() {
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
