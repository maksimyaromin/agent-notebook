//! The fs adapter against a real directory: the Storage contract the
//! in-memory twin already specifies, plus what only a filesystem can pose —
//! bytes outside UTF-8, leftover temp files, and where the root resolves.

use anb::fs_storage::{FsStorage, resolve_root};
use anb_core::{Storage, StorageError};
use std::fs;
use tempfile::TempDir;

fn storage_in(dir: &TempDir) -> FsStorage {
    FsStorage::new(dir.path().to_owned())
}

#[test]
fn write_then_read_round_trips_bytes() {
    let dir = TempDir::new().unwrap();
    let mut storage = storage_in(&dir);
    let content = "---\nid: task.demo\n---\nbody\n";
    storage.write("tasks/task.demo.md", content).unwrap();
    assert_eq!(storage.read("tasks/task.demo.md").unwrap(), content);
}

#[test]
fn a_write_replaces_and_leaves_no_temp_file_behind() {
    let dir = TempDir::new().unwrap();
    let mut storage = storage_in(&dir);
    storage.write("tasks/task.demo.md", "first").unwrap();
    storage.write("tasks/task.demo.md", "second").unwrap();
    assert_eq!(storage.read("tasks/task.demo.md").unwrap(), "second");
    assert_eq!(
        storage.list("tasks").unwrap(),
        vec!["tasks/task.demo.md"],
        "the temp file of the atomic write must be gone"
    );
}

#[test]
fn list_is_non_recursive_sorted_and_prefixed() {
    let dir = TempDir::new().unwrap();
    let mut storage = storage_in(&dir);
    storage.write("tasks/task.b.md", "b").unwrap();
    storage.write("tasks/task.a.md", "a").unwrap();
    storage.write("tasks/nested/task.c.md", "c").unwrap();
    assert_eq!(
        storage.list("tasks").unwrap(),
        vec!["tasks/task.a.md", "tasks/task.b.md"],
        "directories are not entries"
    );
}

#[test]
fn a_missing_directory_lists_empty() {
    let dir = TempDir::new().unwrap();
    assert_eq!(
        storage_in(&dir).list("tasks").unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn reading_a_missing_path_is_not_found() {
    let dir = TempDir::new().unwrap();
    assert_eq!(
        storage_in(&dir).read("tasks/task.absent.md"),
        Err(StorageError::NotFound {
            path: "tasks/task.absent.md".to_owned()
        })
    );
}

#[test]
fn removing_a_missing_path_is_not_found() {
    let dir = TempDir::new().unwrap();
    assert!(matches!(
        storage_in(&dir).remove("tasks/task.absent.md"),
        Err(StorageError::NotFound { .. })
    ));
}

#[test]
fn bytes_outside_utf8_read_as_the_not_utf8_error() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("tasks")).unwrap();
    fs::write(dir.path().join("tasks/task.demo.md"), [0xFF, 0xFE, 0x00]).unwrap();
    assert_eq!(
        storage_in(&dir).read("tasks/task.demo.md"),
        Err(StorageError::NotUtf8 {
            path: "tasks/task.demo.md".to_owned(),
        })
    );
}

mod root_resolution {
    use super::*;

    #[test]
    fn a_notebook_ancestor_inside_the_repository_wins() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::create_dir_all(dir.path().join("a/.agent-notebook")).unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        assert_eq!(
            resolve_root(&dir.path().join("a/b")),
            dir.path().join("a/.agent-notebook")
        );
    }

    #[test]
    fn without_a_notebook_the_repository_root_hosts_it() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        assert_eq!(
            resolve_root(&dir.path().join("a/b")),
            dir.path().join(".agent-notebook")
        );
    }

    #[test]
    fn the_walk_never_crosses_the_repository_boundary() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".agent-notebook")).unwrap();
        fs::create_dir_all(dir.path().join("a/.git")).unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        assert_eq!(
            resolve_root(&dir.path().join("a/b")),
            dir.path().join("a/.agent-notebook"),
            "a notebook above the repository is another project's"
        );
    }

    #[test]
    fn without_a_repository_the_start_directory_hosts_it() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("a")).unwrap();
        assert_eq!(
            resolve_root(&dir.path().join("a")),
            dir.path().join("a/.agent-notebook")
        );
    }
}
