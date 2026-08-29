//! The fs adapter against a real directory: the Storage contract the
//! in-memory twin already specifies, plus what only a filesystem can pose —
//! bytes outside UTF-8, leftover temp files, and where the root resolves.

use anb::fs_storage::{FsStorage, notebook_root, resolve_root};
use anb_core::{Storage, StorageError};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
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

    #[test]
    fn an_empty_flag_still_outranks_the_environment() {
        let dir = TempDir::new().unwrap();
        assert_eq!(
            notebook_root(dir.path(), Some(Path::new("")), Some(OsStr::new("ignored"))),
            dir.path(),
            "a named root is the caller's word, however little of it there is"
        );
    }

    #[test]
    fn a_named_root_outranks_the_environment_and_the_default() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".agent-notebook")).unwrap();
        assert_eq!(
            notebook_root(
                dir.path(),
                Some(Path::new("elsewhere/notes")),
                Some(OsStr::new("ignored")),
            ),
            dir.path().join("elsewhere/notes")
        );
    }

    #[test]
    fn a_relative_environment_root_anchors_on_the_project_not_the_caller() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        assert_eq!(
            notebook_root(
                &dir.path().join("a/b"),
                None,
                Some(OsStr::new(".tmp/private"))
            ),
            dir.path().join(".tmp/private"),
            "one export names one notebook, wherever it is read from"
        );
    }

    #[test]
    fn the_environment_outranks_the_default() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".agent-notebook")).unwrap();
        assert_eq!(
            notebook_root(dir.path(), None, Some(OsStr::new(".tmp/private"))),
            dir.path().join(".tmp/private")
        );
    }

    #[test]
    fn an_absolute_choice_is_taken_whole() {
        let dir = TempDir::new().unwrap();
        let away = TempDir::new().unwrap();
        assert_eq!(
            notebook_root(dir.path(), Some(away.path()), None),
            away.path(),
            "a notebook may live outside the project entirely"
        );
    }

    #[test]
    fn an_empty_choice_is_no_choice() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        assert_eq!(
            notebook_root(dir.path(), None, Some(OsStr::new(""))),
            dir.path().join(".agent-notebook"),
            "an unset variable often arrives as an empty one"
        );
    }
}

/// The relocated notebook, driven end to end through the real binary: the
/// location is configuration, so every verb must reach it wherever it sits.
mod a_notebook_that_moved {
    use super::*;
    use std::process::Command;

    fn anb(project: &TempDir, root: &str, line: &[&str]) -> String {
        anb_in(project.path(), root, line)
    }

    /// One command against a notebook the environment names, run from `cwd`.
    fn anb_in(cwd: &Path, root: &str, line: &[&str]) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_anb"))
            .args(line)
            .current_dir(cwd)
            .env("ANB_NOTEBOOK", root)
            .output()
            .expect("the binary runs");
        assert!(
            output.status.success(),
            "`anb {}` failed: {}",
            line.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("output is UTF-8")
    }

    #[test]
    fn the_flag_outranks_the_environment_all_the_way_through_the_binary() {
        let project = TempDir::new().unwrap();
        fs::create_dir_all(project.path().join(".git")).unwrap();
        anb(
            &project,
            ".tmp/from-the-environment",
            &[
                "--notebook",
                ".tmp/from-the-flag",
                "add",
                "Named on the line",
            ],
        );
        assert!(
            project
                .path()
                .join(".tmp/from-the-flag/tasks/task.named-on-the-line.md")
                .is_file()
        );
        assert!(
            !project.path().join(".tmp/from-the-environment").exists(),
            "the flag is the one that was honored"
        );
    }

    #[test]
    fn one_exported_root_is_one_notebook_from_every_directory() {
        let project = TempDir::new().unwrap();
        fs::create_dir_all(project.path().join(".git")).unwrap();
        let deep = project.path().join("crates/anb/src");
        fs::create_dir_all(&deep).unwrap();

        anb(&project, ".tmp/private", &["add", "Filed from the root"]);
        let listed = anb_in(&deep, ".tmp/private", &["list"]);

        assert!(
            listed.contains("task.filed-from-the-root"),
            "a `cd` must not fork the notebook: {listed}"
        );
        assert!(
            !deep.join(".tmp").exists(),
            "and nothing is written beside the working directory"
        );
    }

    #[test]
    fn a_notebook_outside_any_repository_takes_the_cycle_too() {
        let loose = TempDir::new().unwrap();
        anb(&loose, "notes", &["add", "No repository in sight"]);
        anb(&loose, "notes", &["start", "task.no-repository-in-sight"]);
        anb(
            &loose,
            "notes",
            &["close", "task.no-repository-in-sight", "--no-proof"],
        );
        assert!(
            loose
                .path()
                .join("notes/tasks/task.no-repository-in-sight.md")
                .is_file()
        );
    }

    #[test]
    fn the_task_cycle_reaches_it_and_the_default_stays_empty() {
        let project = TempDir::new().unwrap();
        fs::create_dir_all(project.path().join(".git")).unwrap();
        let elsewhere = ".tmp/private-notebook";

        anb(&project, elsewhere, &["add", "Work kept to myself"]);
        anb(&project, elsewhere, &["start", "task.work-kept-to-myself"]);
        anb(
            &project,
            elsewhere,
            &["comment", "task.work-kept-to-myself", "a line"],
        );
        anb(&project, elsewhere, &["submit", "task.work-kept-to-myself"]);
        anb(
            &project,
            elsewhere,
            &["close", "task.work-kept-to-myself", "--no-proof"],
        );
        anb(
            &project,
            elsewhere,
            &["archive", "task.work-kept-to-myself"],
        );

        anb(&project, elsewhere, &["check"]);
        assert!(
            project
                .path()
                .join(elsewhere)
                .join("archive/tasks/task.work-kept-to-myself.md")
                .is_file()
        );
        assert!(
            !project.path().join(".agent-notebook").exists(),
            "nothing was written where the default would have put it"
        );
    }
}
