//! Resolve notebook audiences once, before acquiring their read or write locks.

use anb::cli::{Cli, Command};
use anb::fs_storage::{NOTEBOOK_ENV, notebook_root, personal_root, unusable_root, user_root};
use anb::recall::Audience;
use anb_core::{NotebookError, StorageError};
use std::path::PathBuf;

pub struct Locations {
    pub cwd: PathBuf,
    pub root: PathBuf,
    pub personal: Option<PathBuf>,
    pub global: Option<PathBuf>,
    pub audience: Audience,
}

impl Locations {
    pub fn resolve(cli: &Cli) -> Result<Self, NotebookError> {
        let cwd = std::env::current_dir().map_err(|error| StorageError::Io {
            path: ".".to_owned(),
            detail: error.to_string(),
        })?;
        let home = std::env::home_dir();
        let recalling = matches!(cli.command, Command::Recall { .. } | Command::Hook);
        let global = user_root(home.as_deref());
        let personal = if cli.personal || (recalling && !cli.global) {
            Some(personal_root(
                &cwd,
                global.as_ref().map_err(|reason| invalid(reason.clone()))?,
            )?)
        } else {
            None
        };
        let root = if let Some(path) = personal.as_ref().filter(|_| cli.personal) {
            path.clone()
        } else {
            notebook_root(
                &cwd,
                cli.notebook.as_deref(),
                cli.global,
                home.as_deref(),
                std::env::var_os(NOTEBOOK_ENV).as_deref(),
            )
            .map_err(invalid)?
        };
        let global = global.ok().filter(|path| recalling && path != &root);
        let personal = personal.filter(|path| recalling && path != &root);
        for path in std::iter::once(&root)
            .chain(global.iter())
            .chain(personal.iter())
        {
            if let Some(reason) = unusable_root(path) {
                return Err(invalid(reason));
            }
        }
        let audience = if cli.global {
            Audience::Global
        } else if cli.personal {
            Audience::Personal
        } else {
            Audience::Project
        };
        Ok(Self {
            cwd,
            root,
            personal,
            global,
            audience,
        })
    }
}

fn invalid(reason: String) -> NotebookError {
    NotebookError::InvalidArgument { reason }
}
