//! The fs adapter against a real directory: the Storage contract the
//! in-memory twin already specifies, plus what only a filesystem can pose —
//! bytes outside UTF-8, leftover temp files, and where the root resolves.

use anb::fs_storage::{FsStorage, notebook_root, resolve_root};
use anb_core::{Storage, StorageError};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn storage_in(dir: &TempDir) -> FsStorage {
    FsStorage::new(dir.path().to_owned())
}

/// A repository with no notebook yet: what the root rules resolve against,
/// and what the binary is run from.
fn a_project() -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".git")).unwrap();
    project
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

/// The root is the seam's whole universe, so a link out of it is not a
/// record however sound its target: a file a project commits could
/// otherwise decide what a later `show` prints.
#[cfg(unix)]
#[test]
fn a_symlink_is_no_record_of_the_notebook() {
    let dir = TempDir::new().unwrap();
    let mut storage = storage_in(&dir);
    storage.write("tasks/task.here.md", "here").unwrap();
    storage.write("elsewhere/task.there.md", "there").unwrap();
    let tasks = dir.path().join("tasks");
    std::os::unix::fs::symlink(
        dir.path().join("elsewhere/task.there.md"),
        tasks.join("task.linked.md"),
    )
    .unwrap();

    assert_eq!(storage.list("tasks").unwrap(), vec!["tasks/task.here.md"]);
    assert_eq!(
        storage.read("tasks/task.linked.md"),
        Err(StorageError::NotFound {
            path: "tasks/task.linked.md".to_owned()
        })
    );
    assert_eq!(storage.exists("tasks/task.linked.md"), Ok(false));
    assert!(storage.write("tasks/task.linked.md", "written").is_err());
    assert_eq!(
        fs::read_to_string(dir.path().join("elsewhere/task.there.md")).unwrap(),
        "there",
        "a write must not travel down a link to a file outside the root"
    );
}

/// The temp file is the atomic write's own business: when the rename cannot
/// land, the failure is the caller's to see and the leftover is not.
#[test]
fn a_write_that_cannot_land_leaves_no_temp_file_behind() {
    let dir = TempDir::new().unwrap();
    let mut storage = storage_in(&dir);
    // A directory standing where the record's file belongs: the temp file
    // is written, and no rename can replace a directory with it.
    fs::create_dir_all(dir.path().join("tasks/task.demo.md/inside")).unwrap();
    assert!(storage.write("tasks/task.demo.md", "body").is_err());
    assert_eq!(
        storage.list("tasks").unwrap(),
        Vec::<String>::new(),
        "the temp file of the failed write must be gone"
    );
}

/// The leaf guard refuses a record that is a link; a directory is walked
/// through without asking. A `tasks` linked out of the root, committed to
/// a project, would have every listing read files the notebook never wrote
/// and every write land outside it — so the root is refused for what it is.
#[cfg(unix)]
#[test]
fn a_root_whose_directory_is_a_link_is_no_notebook() {
    let archive = anb_core::ARCHIVE_DIR;
    for linked in [
        anb_core::RecordType::Task.directory().to_owned(),
        archive.to_owned(),
        format!("{archive}/{}", anb_core::RecordType::Note.directory()),
    ] {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("notebook");
        let outside = dir.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        fs::create_dir_all(root.join(&linked).parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&outside, root.join(&linked)).unwrap();

        assert_eq!(
            anb::fs_storage::unusable_root(&root),
            Some(format!(
                "notebook: {} is a link, not a notebook directory",
                root.join(&linked).display()
            )),
            "a linked `{linked}` must not pass as the notebook's own"
        );
    }
}

/// The root itself is the caller's to place — a notebook may be a link
/// into a dotfile tree, and the lock the writers take is per-inode, so it
/// serializes through one either way. Only a directory *inside* the root
/// is the notebook's own.
#[cfg(unix)]
#[test]
fn a_root_that_is_itself_a_link_is_a_notebook_like_any_other() {
    let dir = TempDir::new().unwrap();
    let real = dir.path().join("elsewhere");
    fs::create_dir_all(real.join("tasks")).unwrap();
    let root = dir.path().join("notebook");
    std::os::unix::fs::symlink(&real, &root).unwrap();

    assert_eq!(anb::fs_storage::unusable_root(&root), None);
    let mut storage = FsStorage::new(root.clone());
    storage.write("tasks/task.demo.md", "body").unwrap();
    assert_eq!(storage.read("tasks/task.demo.md").unwrap(), "body");
}

/// A notebook path that names a file is refused for what it is, rather
/// than through the first write's "File exists".
#[test]
fn a_root_that_names_a_file_is_no_notebook() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("notes.md");
    std::fs::write(&file, "not a notebook").unwrap();
    assert_eq!(
        anb::fs_storage::unusable_root(&file),
        Some(format!(
            "notebook: {} is a file, not a notebook directory",
            file.display()
        ))
    );
    assert_eq!(anb::fs_storage::unusable_root(dir.path()), None);
    assert_eq!(
        anb::fs_storage::unusable_root(&dir.path().join("not-yet")),
        None,
        "a notebook that does not exist yet is not a file"
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

    /// A git worktree and a submodule carry `.git` as a file naming the
    /// real directory. It anchors a project exactly as the directory does —
    /// walking past it would put the notebook in whatever sits above.
    #[test]
    fn a_git_file_anchors_the_project_like_a_git_directory() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        fs::write(
            dir.path().join("a/.git"),
            "gitdir: /elsewhere/.git/worktrees/a\n",
        )
        .unwrap();
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
            notebook_root(
                dir.path(),
                Some(Path::new("")),
                false,
                None,
                Some(OsStr::new("ignored"))
            )
            .unwrap(),
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
                false,
                None,
                Some(OsStr::new("ignored")),
            )
            .unwrap(),
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
                false,
                None,
                Some(OsStr::new(".tmp/private"))
            )
            .unwrap(),
            dir.path().join(".tmp/private"),
            "one export names one notebook, wherever it is read from"
        );
    }

    #[test]
    fn the_environment_outranks_the_default() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".agent-notebook")).unwrap();
        assert_eq!(
            notebook_root(
                dir.path(),
                None,
                false,
                None,
                Some(OsStr::new(".tmp/private"))
            )
            .unwrap(),
            dir.path().join(".tmp/private")
        );
    }

    #[test]
    fn an_absolute_choice_is_taken_whole() {
        let dir = TempDir::new().unwrap();
        let away = TempDir::new().unwrap();
        assert_eq!(
            notebook_root(dir.path(), Some(away.path()), false, None, None).unwrap(),
            away.path(),
            "a notebook may live outside the project entirely"
        );
    }

    #[test]
    fn an_empty_choice_is_no_choice() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        assert_eq!(
            notebook_root(dir.path(), None, false, None, Some(OsStr::new(""))).unwrap(),
            dir.path().join(".agent-notebook"),
            "an unset variable often arrives as an empty one"
        );
    }
}

/// The user's notebook: a second root in the home directory, holding the
/// knowledge that outlives any one repository.
///
/// `HOME` is what `--global` resolves against on this platform, so every
/// case here poses it directly.
#[cfg(unix)]
mod the_users_notebook {
    use super::*;
    use std::process::Command;

    /// One command run from `project`, against `home` as the user's.
    /// `ANB_NOTEBOOK` is passed explicitly, so no case here can be decided
    /// by whatever the developer running the suite exported.
    fn anb_at_home(
        home: &TempDir,
        project: &Path,
        exported: Option<&str>,
        line: &[&str],
    ) -> std::process::Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_anb"));
        command
            .args(line)
            .current_dir(project)
            .env("HOME", home.path());
        match exported {
            Some(root) => command.env("ANB_NOTEBOOK", root),
            None => command.env_remove("ANB_NOTEBOOK"),
        };
        command.output().expect("the binary runs")
    }

    /// [`anb_at_home`] for a case the command must succeed in; the reply is
    /// what the case reads.
    fn served(home: &TempDir, project: &Path, exported: Option<&str>, line: &[&str]) -> String {
        let output = anb_at_home(home, project, exported, line);
        assert!(
            output.status.success(),
            "`anb {}` failed: {}",
            line.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("output is UTF-8")
    }

    #[test]
    fn the_global_scope_names_the_notebook_in_the_users_home() {
        let home = Path::new("/home/reader");
        assert_eq!(
            notebook_root(Path::new("/work/project"), None, true, Some(home), None),
            Ok(home.join(".agent-notebook"))
        );
    }

    /// The flag rung outranks the environment, and `--global` shares it.
    #[test]
    fn the_global_scope_outranks_an_exported_root() {
        let home = Path::new("/home/reader");
        assert_eq!(
            notebook_root(
                Path::new("/work/project"),
                None,
                true,
                Some(home),
                Some(OsStr::new(".tmp/private")),
            ),
            Ok(home.join(".agent-notebook"))
        );
    }

    #[test]
    fn without_the_flag_the_home_decides_nothing() {
        let project = Path::new("/work/project");
        assert_eq!(
            notebook_root(project, None, false, Some(Path::new("/home/reader")), None),
            Ok(project.join(".agent-notebook")),
            "a home is where --global points, not a rung of its own"
        );
    }

    /// Two ways to name one root are one choice made twice, and a caller
    /// who made both is told which two collided rather than given a silent
    /// winner.
    #[test]
    fn naming_a_path_and_the_global_scope_at_once_is_refused() {
        let refused = notebook_root(
            Path::new("/work/project"),
            Some(Path::new("elsewhere")),
            true,
            Some(Path::new("/home/reader")),
            None,
        )
        .expect_err("a contradiction is refused, not ranked");
        assert!(
            refused.contains("--notebook") && refused.contains("--global"),
            "the refusal names both flags: {refused}"
        );
    }

    #[test]
    fn the_global_scope_needs_a_home_to_resolve_against() {
        assert!(
            notebook_root(Path::new("/work/project"), None, true, None, None).is_err(),
            "with no home there is no user's notebook to name"
        );
    }

    /// A home is exported once and outlives every `cd`, exactly like
    /// `ANB_NOTEBOOK`. A relative one read from the working directory would
    /// make `--global` a different notebook in every directory — and would
    /// put the user's private records inside whatever repository they
    /// happened to stand in.
    #[test]
    fn a_relative_home_names_no_users_notebook() {
        assert!(
            notebook_root(
                Path::new("/work/project"),
                None,
                true,
                Some(Path::new("myhome")),
                None,
            )
            .is_err(),
            "one export names one notebook, wherever it is read from"
        );
    }

    /// The whole point of the second root: knowledge recorded from one
    /// repository is read from another, with no path pasted between them.
    #[test]
    fn knowledge_recorded_globally_in_one_project_is_read_from_another() {
        let home = TempDir::new().unwrap();
        let first = a_project();
        let second = a_project();

        served(
            &home,
            first.path(),
            None,
            &[
                "add",
                "decision",
                "Rust for command-line tools",
                "--id",
                "decision.rust-for-clis",
                "--kind",
                "rule",
                "--global",
            ],
        );

        let rows = served(&home, second.path(), None, &["list", "--global"]);
        assert!(
            rows.contains("decision.rust-for-clis"),
            "the other project reads it: {rows}"
        );
        assert!(
            home.path()
                .join(".agent-notebook/decisions/decision.rust-for-clis.md")
                .is_file(),
            "a record is global by residence"
        );
        assert!(
            !first.path().join(".agent-notebook").exists(),
            "and the project it was typed in kept no copy"
        );
    }

    /// The scopes differ in where a record lives and in nothing else: one
    /// grammar, one renderer, one set of rules.
    #[test]
    fn the_same_note_reads_the_same_in_either_scope() {
        let home = TempDir::new().unwrap();
        let project = a_project();
        let filed = ["add", "note", "A shared practice", "--id", "note.practice"];

        served(
            &home,
            project.path(),
            None,
            &[&filed[..], &["--global"]].concat(),
        );
        served(&home, project.path(), None, &filed);

        assert_eq!(
            served(
                &home,
                project.path(),
                None,
                &["show", "note.practice", "--global"]
            ),
            served(&home, project.path(), None, &["show", "note.practice"]),
            "the output contract does not know which scope it is reading"
        );
        assert_eq!(
            fs::read_to_string(home.path().join(".agent-notebook/notes/note.practice.md")).unwrap(),
            fs::read_to_string(
                project
                    .path()
                    .join(".agent-notebook/notes/note.practice.md")
            )
            .unwrap(),
            "and neither does the file"
        );
    }

    /// Work is always project work: a task filed globally would be work no
    /// repository owns.
    #[test]
    fn the_users_notebook_refuses_a_task() {
        let home = TempDir::new().unwrap();
        let project = a_project();

        let refused = anb_at_home(
            &home,
            project.path(),
            None,
            &["--json", "add", "task", "Work with no project", "--global"],
        );

        assert!(!refused.status.success());
        let payload = String::from_utf8(refused.stderr).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(payload.trim()).unwrap_or_else(|_| panic!("not JSON: {payload}"));
        assert_eq!(parsed["error"], "invalid-argument");
        assert!(
            !home.path().join(".agent-notebook").exists(),
            "a refused write leaves no notebook behind"
        );
        assert!(
            !project.path().join(".agent-notebook").exists(),
            "and it is refused, never quietly filed in the project instead"
        );
    }

    /// The pair the second root exists to make visible: a project rule
    /// standing against one of the user's own, counted on the project's
    /// own Status and named in its Debt with both sides and their authors.
    #[test]
    fn a_project_rule_standing_against_the_users_own_reaches_the_projects_debt() {
        let home = TempDir::new().unwrap();
        let project = a_project();
        a_pair_across_the_scopes(&home, &project);

        let status = served(&home, project.path(), None, &["status"]);
        assert!(
            status.contains("debt: 1 — anb debt\n"),
            "the session opens on the count: {status}"
        );
        let debt = served(&home, project.path(), None, &["debt"]);
        let shadow = debt
            .lines()
            .map(str::trim)
            .find(|line| line.starts_with("shadow:"))
            .unwrap_or_else(|| panic!("no shadow line in: {debt}"));
        assert_eq!(
            shadow, "shadow: decision.spaces (Teammate) <-> global decision.tabs (Reader)",
            "the project rule leads: it is the one this repository follows"
        );
        assert!(
            !debt.contains("dangling-mention"),
            "and the citation names something, so nothing calls it missing: {debt}"
        );
    }

    /// An agent reads the Debt as data, so the pair reaches it in
    /// whichever shape it asked for.
    #[test]
    fn the_pair_reaches_the_json_debt_under_its_own_code() {
        let home = TempDir::new().unwrap();
        let project = a_project();
        a_pair_across_the_scopes(&home, &project);

        let payload = served(&home, project.path(), None, &["--json", "debt"]);
        let parsed: serde_json::Value = serde_json::from_str(payload.trim()).unwrap();
        let rows = parsed["debt"].as_array().expect("debt rows");
        assert!(
            rows.iter().any(|row| {
                row["code"] == "shadow"
                    && row["line"]
                        == "shadow: decision.spaces (Teammate) <-> global decision.tabs (Reader)"
            }),
            "no shadow row in: {payload}"
        );
    }

    /// The second root is read and never written: a repository does not
    /// mutate the user's home.
    #[test]
    fn reading_the_users_notebook_for_a_status_leaves_it_byte_for_byte() {
        let home = TempDir::new().unwrap();
        let project = a_project();
        a_pair_across_the_scopes(&home, &project);
        let before = tree_of(&home.path().join(".agent-notebook"));

        served(&home, project.path(), None, &["status"]);

        assert_eq!(tree_of(&home.path().join(".agent-notebook")), before);
    }

    /// A session opens with a Status, so a project's own dashboard cannot
    /// be stopped by the state of a notebook that project does not own.
    #[test]
    fn a_users_notebook_that_is_a_file_leaves_the_projects_status_standing() {
        let home = TempDir::new().unwrap();
        let project = a_project();
        served(
            &home,
            project.path(),
            None,
            &["add", "task", "Work with a project", "--id", "task.work"],
        );
        fs::write(home.path().join(".agent-notebook"), "not a notebook").unwrap();

        let status = served(&home, project.path(), None, &["status"]);
        assert!(status.contains("task.work"), "{status}");
    }

    /// The user's notebook is a notebook root like any other, and the seam
    /// will not walk a directory it did not write.
    #[cfg(unix)]
    #[test]
    fn a_users_notebook_behind_a_linked_directory_is_no_second_scope() {
        let home = TempDir::new().unwrap();
        let project = a_project();
        a_pair_across_the_scopes(&home, &project);
        let decisions = home.path().join(".agent-notebook/decisions");
        let elsewhere = home.path().join("elsewhere");
        fs::rename(&decisions, &elsewhere).unwrap();
        std::os::unix::fs::symlink(&elsewhere, &decisions).unwrap();

        let status = served(&home, project.path(), None, &["status"]);
        assert!(
            !status.contains("shadow:"),
            "a linked directory is not the user's notebook: {status}"
        );
    }

    /// A rule in each scope, the project's citing the user's.
    fn a_pair_across_the_scopes(home: &TempDir, project: &TempDir) {
        served(
            home,
            project.path(),
            None,
            &[
                "add",
                "decision",
                "Indent with tabs",
                "--id",
                "decision.tabs",
                "--kind",
                "rule",
                "--by",
                "Reader",
                "--global",
            ],
        );
        served(
            home,
            project.path(),
            None,
            &[
                "add",
                "decision",
                "Indent with spaces here",
                "--id",
                "decision.spaces",
                "--kind",
                "rule",
                "--by",
                "Teammate",
                "--body",
                "This repository indents with spaces, against decision.tabs.",
            ],
        );
    }

    /// Every file under `root` with its bytes, so a case about what a read
    /// left behind sees an added or removed file as well as a changed one.
    fn tree_of(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut found = Vec::new();
        let mut pending = vec![root.to_owned()];
        while let Some(directory) = pending.pop() {
            let Ok(entries) = fs::read_dir(&directory) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                } else {
                    let bytes = fs::read(&path).unwrap_or_default();
                    found.push((path, bytes));
                }
            }
        }
        found.sort();
        found
    }

    /// The same verb without the flag is the project's to serve, so the
    /// refusal is the scope's and not the verb's.
    #[test]
    fn the_project_still_takes_the_work_the_global_scope_refused() {
        let home = TempDir::new().unwrap();
        let project = a_project();

        served(
            &home,
            project.path(),
            None,
            &["add", "task", "Work with a project", "--id", "task.work"],
        );
        assert!(
            project
                .path()
                .join(".agent-notebook/tasks/task.work.md")
                .is_file()
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
    /// `HOME` is pointed at the project: a Status reads the user's notebook
    /// behind the project's, and the developer's own must not decide what a
    /// case proves.
    fn anb_in(cwd: &Path, root: &str, line: &[&str]) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_anb"))
            .args(line)
            .current_dir(cwd)
            .env("ANB_NOTEBOOK", root)
            .env("HOME", cwd)
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
        let project = a_project();
        anb(
            &project,
            ".tmp/from-the-environment",
            &[
                "--notebook",
                ".tmp/from-the-flag",
                "add",
                "task",
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
        let project = a_project();
        let deep = project.path().join("crates/anb/src");
        fs::create_dir_all(&deep).unwrap();

        anb(
            &project,
            ".tmp/private",
            &["add", "task", "Filed from the root"],
        );
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
        anb(&loose, "notes", &["add", "task", "No repository in sight"]);
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
        let project = a_project();
        let elsewhere = ".tmp/private-notebook";

        anb(&project, elsewhere, &["add", "task", "Work kept to myself"]);
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

/// Settling proofs against a real repository and a real working tree.
mod reconciliation {
    use super::*;
    use anb::reconcile::lost_proofs;
    use anb_core::CitedProof;
    use std::process::Command;

    fn cited(kind: &str, target: &str) -> CitedProof {
        CitedProof {
            record: "task.shipped".to_owned(),
            kind: kind.to_owned(),
            target: target.to_owned(),
        }
    }

    fn git(dir: &TempDir, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("git runs");
        String::from_utf8(output.stdout).expect("git speaks UTF-8")
    }

    /// A repository with one commit, and the sha it made. `history` goes
    /// into the commit, so two repositories never coincide on a sha.
    fn a_repository_of(history: &str) -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        git(&dir, &["init", "-q", "."]);
        git(&dir, &["config", "user.email", "t@e.st"]);
        git(&dir, &["config", "user.name", "T"]);
        fs::write(dir.path().join("f.txt"), history).unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-qm", history]);
        let sha = git(&dir, &["rev-parse", "HEAD"]).trim().to_owned();
        (dir, sha)
    }

    fn a_repository() -> (TempDir, String) {
        a_repository_of("one")
    }

    #[test]
    fn a_commit_the_repository_has_is_not_lost() {
        let (dir, sha) = a_repository();
        assert_eq!(lost_proofs(dir.path(), &[cited("sha", &sha)]), vec![]);
    }

    #[test]
    fn a_commit_the_repository_never_had_is_lost() {
        let (dir, _) = a_repository();
        let gone = cited("sha", "f00dfeedf00dfeedf00dfeedf00dfeedf00dfeed");
        assert_eq!(
            lost_proofs(dir.path(), std::slice::from_ref(&gone)),
            vec![gone]
        );
    }

    #[test]
    fn each_answer_stays_with_the_proof_that_asked_it() {
        let (dir, sha) = a_repository();
        let asked = [
            cited("sha", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            cited("sha", &sha),
            cited("sha", "not-a-sha-at-all"),
        ];
        assert_eq!(
            lost_proofs(dir.path(), &asked),
            vec![asked[0].clone(), asked[2].clone()],
            "the present one in the middle must not shift the answers around it"
        );
    }

    #[test]
    fn more_proofs_than_a_pipe_holds_still_answer() {
        let (dir, sha) = a_repository();
        // Well past a pipe buffer: the query cannot be written and the
        // answers read on one thread without both ends blocking.
        let mut asked: Vec<CitedProof> = (0..4000)
            .map(|n| cited("sha", &format!("{n:040}")))
            .collect();
        asked.push(cited("sha", &sha));
        assert_eq!(
            lost_proofs(dir.path(), &asked).len(),
            4000,
            "every absent one is named, and the present one is not"
        );
    }

    #[test]
    fn a_report_is_settled_by_the_working_tree() {
        let (dir, _) = a_repository();
        fs::create_dir_all(dir.path().join("notes")).unwrap();
        fs::write(dir.path().join("notes/there.md"), "the report").unwrap();
        let gone = cited("report", "notes/gone.md");
        assert_eq!(
            lost_proofs(
                &dir.path().join(".agent-notebook"),
                &[cited("report", "notes/there.md"), gone.clone()]
            ),
            vec![gone],
            "a file left where it lies is a claim a stat settles"
        );
    }

    #[test]
    fn a_report_path_is_read_from_the_project_not_the_notebook_directory() {
        let (dir, _) = a_repository();
        fs::create_dir_all(dir.path().join("reports")).unwrap();
        fs::write(dir.path().join("reports/r.md"), "the report").unwrap();
        assert_eq!(
            lost_proofs(
                &dir.path().join("elsewhere/notebook"),
                &[cited("report", "reports/r.md")]
            ),
            vec![],
            "a path in a record outlives the shell that typed it; the project is the base"
        );
    }

    #[test]
    fn a_proof_nothing_here_can_settle_is_left_alone() {
        let (dir, _) = a_repository();
        assert_eq!(
            lost_proofs(dir.path(), &[cited("pr", "https://example.com/pull/7")]),
            vec![],
            "silence means not known to be lost, never verified"
        );
    }

    #[test]
    fn a_notebook_outside_any_repository_diverges_from_nothing() {
        let loose = TempDir::new().unwrap();
        assert_eq!(
            lost_proofs(loose.path(), &[cited("sha", "f00dfeed")]),
            vec![],
            "with no repository to ask, an accusation would be invented"
        );
    }

    #[test]
    fn the_repository_asked_is_the_notebook_s_own() {
        let (home, sha) = a_repository_of("the notebook's own history");
        let (elsewhere, _) = a_repository_of("a different project entirely");
        assert_eq!(
            lost_proofs(home.path(), &[cited("sha", &sha)]),
            vec![],
            "the commit is in the notebook's repository, wherever the caller stands"
        );
        assert_eq!(
            lost_proofs(elsewhere.path(), &[cited("sha", &sha)]).len(),
            1,
            "and another repository genuinely does not have it"
        );
    }
}

/// `setup` through the real binary: the files an agent reads at session
/// start, written into the project, patched in place, and taken out again
/// leaving no trace of their own.
mod setup {
    use super::*;
    use std::process::{Command, Output};

    fn anb(project: &Path, line: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_anb"))
            .args(line)
            .current_dir(project)
            .env("HOME", project)
            .env_remove("ANB_NOTEBOOK")
            .output()
            .expect("the binary runs")
    }

    fn ok(project: &Path, line: &[&str]) -> String {
        let output = anb(project, line);
        assert!(
            output.status.success(),
            "`anb {}` failed: {}",
            line.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("output is UTF-8")
    }

    fn read(project: &Path, file: &str) -> String {
        fs::read_to_string(project.join(file)).unwrap_or_else(|_| panic!("{file} is there"))
    }

    #[test]
    fn setup_writes_the_snippet_and_both_hooks_and_a_second_run_changes_nothing() {
        let project = TempDir::new().unwrap();
        let first = ok(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        );
        assert_eq!(
            first,
            "ok: setup — 18 files\n  AGENTS.md: written\n  CLAUDE.md: written\n  .claude/settings.json: written\n  .codex/hooks.json: written\n  .claude/skills/anb/SKILL.md: written\n  .claude/skills/anb/references/commands.md: written\n  .claude/skills/anb/references/session.md: written\n  .claude/skills/anb/references/refusals.md: written\n  .claude/skills/anb-atlas/SKILL.md: written\n  .claude/skills/anb-atlas/references/drawing.md: written\n  .claude/skills/anb-atlas/references/intent-loop.md: written\n  .agents/skills/anb/SKILL.md: written\n  .agents/skills/anb/references/commands.md: written\n  .agents/skills/anb/references/session.md: written\n  .agents/skills/anb/references/refusals.md: written\n  .agents/skills/anb-atlas/SKILL.md: written\n  .agents/skills/anb-atlas/references/drawing.md: written\n  .agents/skills/anb-atlas/references/intent-loop.md: written\nskipped: agents-md\nnotice: Codex runs a project hook after you review it: run /hooks in Codex from this directory\n"
        );
        assert!(read(project.path(), "AGENTS.md").contains("<!-- anb:begin -->"));
        assert!(read(project.path(), "CLAUDE.md").contains("<!-- anb:begin -->"));
        let settings: serde_json::Value =
            serde_json::from_str(&read(project.path(), ".claude/settings.json")).unwrap();
        assert_eq!(
            settings["hooks"]["SessionStart"][0]["hooks"][0]["command"],
            serde_json::json!("anb status --hook")
        );
        let codex: serde_json::Value =
            serde_json::from_str(&read(project.path(), ".codex/hooks.json")).unwrap();
        assert_eq!(
            codex["hooks"]["SessionStart"][0]["hooks"][0]["command"],
            serde_json::json!("anb status --hook")
        );

        let agents_before = read(project.path(), "AGENTS.md");
        let second = ok(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        );
        assert_eq!(
            second,
            "ok: setup — 18 files\n  AGENTS.md: already\n  CLAUDE.md: already\n  .claude/settings.json: already\n  .codex/hooks.json: already\n  .claude/skills/anb/SKILL.md: already\n  .claude/skills/anb/references/commands.md: already\n  .claude/skills/anb/references/session.md: already\n  .claude/skills/anb/references/refusals.md: already\n  .claude/skills/anb-atlas/SKILL.md: already\n  .claude/skills/anb-atlas/references/drawing.md: already\n  .claude/skills/anb-atlas/references/intent-loop.md: already\n  .agents/skills/anb/SKILL.md: already\n  .agents/skills/anb/references/commands.md: already\n  .agents/skills/anb/references/session.md: already\n  .agents/skills/anb/references/refusals.md: already\n  .agents/skills/anb-atlas/SKILL.md: already\n  .agents/skills/anb-atlas/references/drawing.md: already\n  .agents/skills/anb-atlas/references/intent-loop.md: already\nskipped: agents-md\n",
            "a re-run finds its own lines and adds nothing"
        );
        assert_eq!(read(project.path(), "AGENTS.md"), agents_before);
    }

    #[test]
    fn setup_keeps_what_other_tools_wrote_and_remove_takes_out_only_its_own() {
        let project = TempDir::new().unwrap();
        fs::write(
            project.path().join("AGENTS.md"),
            "# Our guide\n\nRead the docs.\n",
        )
        .unwrap();
        fs::create_dir_all(project.path().join(".claude")).unwrap();
        fs::write(
            project.path().join(".claude/settings.json"),
            "{\n  \"permissions\": {\"allow\": [\"Bash(ls)\"]},\n  \"hooks\": {\"SessionStart\": [{\"hooks\": [{\"type\": \"command\", \"command\": \"other-tool prime\"}]}]}\n}\n",
        )
        .unwrap();
        ok(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        );
        let agents = read(project.path(), "AGENTS.md");
        assert!(
            agents.starts_with("# Our guide\n\nRead the docs.\n"),
            "{agents}"
        );
        let settings: serde_json::Value =
            serde_json::from_str(&read(project.path(), ".claude/settings.json")).unwrap();
        assert_eq!(
            settings["permissions"]["allow"][0],
            serde_json::json!("Bash(ls)")
        );
        assert_eq!(
            settings["hooks"]["SessionStart"].as_array().unwrap().len(),
            2
        );

        let removed = ok(
            project.path(),
            &[
                "setup",
                "--agent",
                "claude-code",
                "--agent",
                "codex",
                "--agent",
                "agents-md",
                "--remove",
            ],
        );
        assert_eq!(
            removed,
            "ok: setup --remove — 18 files\n  AGENTS.md: removed\n  CLAUDE.md: removed\n  .claude/settings.json: removed\n  .codex/hooks.json: removed\n  .claude/skills/anb/SKILL.md: removed\n  .claude/skills/anb/references/commands.md: removed\n  .claude/skills/anb/references/session.md: removed\n  .claude/skills/anb/references/refusals.md: removed\n  .claude/skills/anb-atlas/SKILL.md: removed\n  .claude/skills/anb-atlas/references/drawing.md: removed\n  .claude/skills/anb-atlas/references/intent-loop.md: removed\n  .agents/skills/anb/SKILL.md: removed\n  .agents/skills/anb/references/commands.md: removed\n  .agents/skills/anb/references/session.md: removed\n  .agents/skills/anb/references/refusals.md: removed\n  .agents/skills/anb-atlas/SKILL.md: removed\n  .agents/skills/anb-atlas/references/drawing.md: removed\n  .agents/skills/anb-atlas/references/intent-loop.md: removed\n"
        );
        assert_eq!(
            read(project.path(), "AGENTS.md"),
            "# Our guide\n\nRead the docs.\n"
        );
        assert!(
            !project.path().join("CLAUDE.md").exists(),
            "a file setup alone filled is gone"
        );
        let settings: serde_json::Value =
            serde_json::from_str(&read(project.path(), ".claude/settings.json")).unwrap();
        assert_eq!(
            settings["hooks"]["SessionStart"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            settings["permissions"]["allow"][0],
            serde_json::json!("Bash(ls)")
        );
        assert!(!project.path().join(".codex/hooks.json").exists());
        assert!(
            !project.path().join(".agents/skills/anb").exists(),
            "a skill directory setup emptied is gone with its files"
        );

        let again = ok(
            project.path(),
            &[
                "setup",
                "--agent",
                "claude-code",
                "--agent",
                "codex",
                "--agent",
                "agents-md",
                "--remove",
            ],
        );
        assert!(again.contains("AGENTS.md: absent"), "{again}");
    }

    #[test]
    fn setup_writes_only_the_named_agents_files() {
        let project = TempDir::new().unwrap();
        let reply = ok(project.path(), &["setup", "--agent", "claude-code"]);
        assert_eq!(
            reply,
            "ok: setup — 9 files\n  CLAUDE.md: written\n  .claude/settings.json: written\n  .claude/skills/anb/SKILL.md: written\n  .claude/skills/anb/references/commands.md: written\n  .claude/skills/anb/references/session.md: written\n  .claude/skills/anb/references/refusals.md: written\n  .claude/skills/anb-atlas/SKILL.md: written\n  .claude/skills/anb-atlas/references/drawing.md: written\n  .claude/skills/anb-atlas/references/intent-loop.md: written\nskipped: codex, agents-md\n"
        );
        for absent in ["AGENTS.md", ".codex", ".agents"] {
            assert!(
                !project.path().join(absent).exists(),
                "{absent} belongs to an agent nobody named"
            );
        }
    }

    #[test]
    fn a_claude_file_importing_the_agents_file_puts_the_line_there_for_claude_alone() {
        let project = TempDir::new().unwrap();
        fs::write(project.path().join("CLAUDE.md"), "@AGENTS.md\n").unwrap();
        let reply = ok(project.path(), &["setup", "--agent", "claude-code"]);
        assert!(reply.contains("AGENTS.md: written"), "{reply}");
        assert!(reply.contains("CLAUDE.md: imports AGENTS.md"), "{reply}");
        assert!(read(project.path(), "AGENTS.md").contains("<!-- anb:begin -->"));
    }

    #[test]
    fn setup_without_an_agent_refuses_and_writes_nothing() {
        let project = TempDir::new().unwrap();
        let refused = anb(project.path(), &["setup"]);
        assert!(!refused.status.success());
        assert_eq!(
            String::from_utf8(refused.stderr).unwrap(),
            "error[invalid-argument]: setup: name the agents to wire with --agent, one of claude-code, codex, agents-md\ntry: anb setup --agent claude-code\ntry: anb setup --agent codex\ntry: anb setup --agent agents-md\n"
        );
        assert_eq!(fs::read_dir(project.path()).unwrap().count(), 0);
    }

    #[test]
    fn an_agent_setup_does_not_know_is_refused() {
        let project = TempDir::new().unwrap();
        let refused = anb(project.path(), &["setup", "--agent", "pi"]);
        assert!(!refused.status.success());
        assert!(
            String::from_utf8(refused.stderr)
                .unwrap()
                .starts_with("error[invalid-argument]: setup: `pi` is not an agent; one of claude-code, codex, agents-md\n")
        );
        assert_eq!(fs::read_dir(project.path()).unwrap().count(), 0);
    }

    #[test]
    fn remove_takes_out_the_named_agents_own_files_and_keeps_what_another_reads() {
        let project = TempDir::new().unwrap();
        ok(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        );
        let removed = ok(project.path(), &["setup", "--agent", "codex", "--remove"]);
        assert_eq!(
            removed,
            "ok: setup --remove — 3 files\n  AGENTS.md: read by an agent not named, kept\n  .codex/hooks.json: removed\n  .agents/skills: read by an agent not named, kept\nskipped: claude-code, agents-md\n"
        );
        assert!(
            !project.path().join(".codex").exists(),
            "the hook's directory was setup's alone"
        );
        assert!(read(project.path(), "AGENTS.md").contains("<!-- anb:begin -->"));
        assert!(project.path().join(".agents/skills/anb/SKILL.md").exists());
        assert!(read(project.path(), "CLAUDE.md").contains("<!-- anb:begin -->"));

        let rest = ok(
            project.path(),
            &[
                "setup",
                "--agent",
                "codex",
                "--agent",
                "agents-md",
                "--remove",
            ],
        );
        assert!(rest.contains("AGENTS.md: removed"), "{rest}");
        assert!(!project.path().join(".agents").exists());
        assert!(read(project.path(), "CLAUDE.md").contains("<!-- anb:begin -->"));
        assert!(project.path().join(".claude/skills/anb/SKILL.md").exists());
    }

    #[test]
    fn a_claude_file_importing_the_agents_file_keeps_the_line_until_claude_is_named_too() {
        let project = TempDir::new().unwrap();
        fs::write(project.path().join("CLAUDE.md"), "@AGENTS.md\n").unwrap();
        ok(
            project.path(),
            &[
                "setup",
                "--agent",
                "claude-code",
                "--agent",
                "codex",
                "--agent",
                "agents-md",
            ],
        );
        let removed = ok(
            project.path(),
            &[
                "setup",
                "--agent",
                "codex",
                "--agent",
                "agents-md",
                "--remove",
            ],
        );
        assert!(
            removed.contains("AGENTS.md: read by an agent not named, kept"),
            "{removed}"
        );
        assert!(read(project.path(), "AGENTS.md").contains("<!-- anb:begin -->"));
    }

    #[test]
    fn a_claude_file_linked_to_the_agents_file_gets_one_line_not_two() {
        let project = TempDir::new().unwrap();
        fs::write(project.path().join("AGENTS.md"), "# Guide\n").unwrap();
        std::os::unix::fs::symlink("AGENTS.md", project.path().join("CLAUDE.md")).unwrap();
        let reply = ok(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        );
        assert!(reply.contains("CLAUDE.md: links AGENTS.md"), "{reply}");
        assert_eq!(
            read(project.path(), "AGENTS.md")
                .matches("<!-- anb:begin -->")
                .count(),
            1
        );
    }

    #[test]
    fn a_claude_file_importing_the_agents_file_is_left_alone() {
        let project = TempDir::new().unwrap();
        fs::write(
            project.path().join("CLAUDE.md"),
            "@AGENTS.md\n\n## Claude Code\n",
        )
        .unwrap();
        let reply = ok(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        );
        assert!(reply.contains("CLAUDE.md: imports AGENTS.md"), "{reply}");
        assert_eq!(
            read(project.path(), "CLAUDE.md"),
            "@AGENTS.md\n\n## Claude Code\n"
        );
    }

    #[test]
    fn a_settings_file_that_is_not_json_refuses_the_whole_setup_and_moves_nothing() {
        let project = TempDir::new().unwrap();
        fs::create_dir_all(project.path().join(".claude")).unwrap();
        fs::write(project.path().join(".claude/settings.json"), "{ not json").unwrap();
        let refused = anb(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        );
        assert!(!refused.status.success());
        let payload = String::from_utf8(refused.stderr).unwrap();
        assert!(
            payload
                .starts_with("error[invalid-argument]: setup: .claude/settings.json is not JSON"),
            "{payload}"
        );
        assert_eq!(read(project.path(), ".claude/settings.json"), "{ not json");
    }

    #[test]
    fn setup_refuses_the_users_notebook() {
        let project = TempDir::new().unwrap();
        let refused = anb(
            project.path(),
            &["setup", "--agent", "claude-code", "--global"],
        );
        assert!(!refused.status.success());
        assert!(
            String::from_utf8(refused.stderr)
                .unwrap()
                .contains("setup: installs into the project"),
        );
        assert!(!project.path().join("AGENTS.md").exists());
    }
}

/// The skill through the real binary: rendered into a directory, checked
/// against it, and installed by setup where each agent looks.
mod skill {
    use super::*;
    use std::process::{Command, Output};

    fn anb(project: &Path, line: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_anb"))
            .args(line)
            .current_dir(project)
            .env("HOME", project)
            .env_remove("ANB_NOTEBOOK")
            .output()
            .expect("the binary runs")
    }

    fn stdout(output: &Output) -> String {
        String::from_utf8(output.stdout.clone()).expect("output is UTF-8")
    }

    #[test]
    fn the_skill_is_written_checked_and_found_drifted_when_edited() {
        let project = TempDir::new().unwrap();
        let written = anb(project.path(), &["skill", "skills/anb"]);
        assert!(written.status.success());
        assert_eq!(
            stdout(&written),
            "ok: skill — 4 files written into skills/anb\n"
        );
        assert!(
            fs::read_to_string(project.path().join("skills/anb/SKILL.md"))
                .unwrap()
                .starts_with("---\nname: anb\n")
        );

        let checked = anb(project.path(), &["skill", "skills/anb", "--check"]);
        assert!(checked.status.success());
        assert_eq!(
            stdout(&checked),
            "ok: skill — skills/anb matches the rendering\n"
        );

        let path = project.path().join("skills/anb/references/refusals.md");
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, text + "\nan edit by hand\n").unwrap();
        fs::remove_file(project.path().join("skills/anb/SKILL.md")).unwrap();
        let drifted = anb(project.path(), &["skill", "skills/anb", "--check"]);
        assert!(
            !drifted.status.success(),
            "drift is a failing exit, so CI stops on it"
        );
        assert_eq!(
            stdout(&drifted),
            "skill: skills/anb has drifted from the rendering\n  SKILL.md: missing\n  references/refusals.md: differs from the rendering\ntry: anb skill skills/anb\n"
        );

        let as_json = anb(
            project.path(),
            &["--json", "skill", "skills/anb", "--check"],
        );
        assert!(!as_json.status.success());
        assert_eq!(
            stdout(&as_json),
            r#"{"ok":"skill","dir":"skills/anb","drift":[{"file":"SKILL.md","reason":"missing"},{"file":"references/refusals.md","reason":"differs from the rendering"}]}
"#
        );
    }

    #[test]
    fn a_skill_file_the_user_made_theirs_is_left_alone_by_setup() {
        let project = TempDir::new().unwrap();
        anb(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        );
        let path = project.path().join(".claude/skills/anb/SKILL.md");
        let text = fs::read_to_string(&path).unwrap();
        let theirs = text.replace("  managed-by: anb\n", "") + "\nOur team's own rule.\n";
        fs::write(&path, &theirs).unwrap();

        let rerun = stdout(&anb(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        ));
        assert!(
            rerun.contains(".claude/skills/anb/SKILL.md: yours, left alone"),
            "{rerun}"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), theirs);

        let removed = stdout(&anb(
            project.path(),
            &[
                "setup",
                "--agent",
                "claude-code",
                "--agent",
                "codex",
                "--remove",
            ],
        ));
        assert!(
            removed.contains(".claude/skills/anb/SKILL.md: yours, left alone"),
            "{removed}"
        );
        assert!(path.exists(), "a file the user owns is never deleted");
    }

    #[test]
    fn removal_takes_only_the_directories_it_emptied() {
        let project = TempDir::new().unwrap();
        anb(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        );
        let in_references = project.path().join(".claude/skills/anb/references/mine.md");
        let in_skill = project.path().join(".agents/skills/anb/mine.md");
        fs::write(&in_references, "mine\n").unwrap();
        fs::write(&in_skill, "mine\n").unwrap();

        let removed = anb(
            project.path(),
            &[
                "setup",
                "--agent",
                "claude-code",
                "--agent",
                "codex",
                "--agent",
                "agents-md",
                "--remove",
            ],
        );
        assert!(removed.status.success());
        assert!(in_references.exists(), "a stray file keeps its directory");
        assert!(in_skill.exists(), "a stray file keeps the skill directory");
        assert!(
            !project
                .path()
                .join(".agents/skills/anb/references")
                .exists()
        );
        assert!(
            !project
                .path()
                .join(".claude/skills/anb/references/commands.md")
                .exists(),
            "the generated files are gone around the stray one"
        );
    }

    #[test]
    fn removal_takes_out_the_host_directories_it_emptied_and_no_other() {
        let project = TempDir::new().unwrap();
        fs::create_dir_all(project.path().join(".claude/skills/theirs")).unwrap();
        fs::write(
            project.path().join(".claude/skills/theirs/SKILL.md"),
            "# Theirs\n",
        )
        .unwrap();
        anb(
            project.path(),
            &["setup", "--agent", "claude-code", "--agent", "codex"],
        );
        anb(
            project.path(),
            &[
                "setup",
                "--agent",
                "claude-code",
                "--agent",
                "codex",
                "--agent",
                "agents-md",
                "--remove",
            ],
        );
        assert!(
            project
                .path()
                .join(".claude/skills/theirs/SKILL.md")
                .exists(),
            "a directory holding anything else stays"
        );
        assert!(!project.path().join(".claude/skills/anb").exists());
        for gone in [".agents", ".codex"] {
            assert!(
                !project.path().join(gone).exists(),
                "{gone} held nothing but setup's files"
            );
        }
    }

    #[test]
    fn a_reference_the_user_made_theirs_is_left_alone_too() {
        for file in [
            ".agents/skills/anb/references/commands.md",
            ".claude/skills/anb-atlas/references/drawing.md",
        ] {
            let project = TempDir::new().unwrap();
            anb(
                project.path(),
                &["setup", "--agent", "claude-code", "--agent", "codex"],
            );
            let path = project.path().join(file);
            let theirs = fs::read_to_string(&path)
                .unwrap()
                .replace("  managed-by: anb\n", "");
            fs::write(&path, &theirs).unwrap();

            let rerun = stdout(&anb(
                project.path(),
                &["setup", "--agent", "claude-code", "--agent", "codex"],
            ));
            assert!(
                rerun.contains(&format!("{file}: yours, left alone")),
                "{rerun}"
            );
            let removed = stdout(&anb(
                project.path(),
                &[
                    "setup",
                    "--agent",
                    "claude-code",
                    "--agent",
                    "codex",
                    "--agent",
                    "agents-md",
                    "--remove",
                ],
            ));
            assert!(
                removed.contains(&format!("{file}: yours, left alone")),
                "{removed}"
            );
            assert_eq!(fs::read_to_string(&path).unwrap(), theirs);
        }
    }
}
