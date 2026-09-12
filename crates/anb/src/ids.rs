//! Allocate identifiers independently of a checkout's clock and contents.

use anb::cli::Command;
use anb_core::{NotebookError, StorageError};
use std::fmt::Write as _;

/// Explicit ids are caller-owned. New ids use 128 bits of OS randomness;
/// no local counter can coordinate independent offline clones.
pub fn assign(command: &mut Command) -> Result<(), NotebookError> {
    let Command::Add(args) = command else {
        return Ok(());
    };
    if args.id.is_some() {
        return Ok(());
    }
    let mut random = [0_u8; 16];
    getrandom::fill(&mut random).map_err(|error| StorageError::Io {
        path: "system random source".to_owned(),
        detail: format!("could not allocate a record id: {error}"),
    })?;
    let mut id = format!("{}.", args.record_type.word());
    for byte in random {
        let _ = write!(id, "{byte:02x}");
    }
    args.id = Some(id);
    Ok(())
}
