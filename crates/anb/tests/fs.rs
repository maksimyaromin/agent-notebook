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

/// The root is the seam's whole universe, so a link out of it is not a
/// record however sound its target: a file a project commits could
/// otherwise decide what a later `view` prints.
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

/// What the shell sees when it stops listening mid-reply.
mod a_reader_that_walks_away {
    use super::*;
    use std::fmt::Write as _;
    use std::io::Read as _;
    use std::process::{Command, Stdio};

    #[test]
    fn a_reader_that_stops_early_ends_the_reply_quietly() {
        let project = TempDir::new().unwrap();
        let notebook = project.path().join("nb");
        fs::create_dir_all(notebook.join("tasks")).unwrap();
        // Past any pipe buffer, so the write blocks and then fails.
        let mut body = String::new();
        for line in 0..20_000 {
            let _ = writeln!(body, "  line {line}");
        }
        fs::write(
            notebook.join("tasks/task.big.md"),
            format!(
                "---\nid: task.big\ntype: task\nstate: open\ntitle: Big\ncreated: 2026-08-24\nupdated: 2026-08-25\n---\n\n{body}"
            ),
        )
        .unwrap();

        let mut view = Command::new(env!("CARGO_BIN_EXE_anb"))
            .args(["--notebook", notebook.to_str().unwrap(), "view", "task.big"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the binary runs");
        // The pipe closes when this temporary drops at the end of the
        // statement — binding it would keep the reader listening.
        let mut first = [0u8; 16];
        view.stdout.take().unwrap().read_exact(&mut first).unwrap();

        let ended = view.wait_with_output().unwrap();
        assert!(
            ended.status.success(),
            "a closed pipe is the reader's choice, not a failure: {:?}, {}",
            ended.status,
            String::from_utf8_lossy(&ended.stderr)
        );
        assert!(
            ended.stderr.is_empty(),
            "nothing is reported about it either: {}",
            String::from_utf8_lossy(&ended.stderr)
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
