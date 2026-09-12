use anb::cli::Cli;
use anb::lock;
use anb_core::StorageError;
use clap::Parser;
use std::fs;
use tempfile::TempDir;

#[test]
fn a_read_without_a_lock_file_leaves_the_notebook_untouched() {
    let root = TempDir::new().unwrap();
    let command = Cli::try_parse_from(["anb", "check"]).unwrap().command;
    assert!(lock::taken(root.path(), &command).unwrap().is_none());
    assert!(fs::read_dir(root.path()).unwrap().next().is_none());
}

#[test]
fn an_existing_unusable_lock_is_a_read_error_not_an_unlocked_snapshot() {
    let root = TempDir::new().unwrap();
    fs::create_dir(root.path().join(".lock")).unwrap();
    let command = Cli::try_parse_from(["anb", "check"]).unwrap().command;
    assert!(matches!(
        lock::taken(root.path(), &command),
        Err(StorageError::Io { .. })
    ));
}

#[cfg(unix)]
#[test]
fn a_symlinked_lock_cannot_create_a_file_outside_the_notebook() {
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let target = outside.path().join("untouched");
    std::os::unix::fs::symlink(&target, root.path().join(".lock")).unwrap();
    let command = Cli::try_parse_from(["anb", "add", "note", "Evidence"])
        .unwrap()
        .command;
    assert!(matches!(
        lock::taken(root.path(), &command),
        Err(StorageError::Io { .. })
    ));
    assert!(!target.exists());
}
