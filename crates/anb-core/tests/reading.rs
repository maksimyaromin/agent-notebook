//! What a verb opens. A notebook's cost is the files it reads, and the
//! archive is the one part of it that only ever grows — so which paths
//! cross the Storage seam is behaviour, watched here at the seam itself.

use anb_core::{
    Budget, CitedProof, Draft, MemoryStorage, Notebook, NotebookError, RecordType, Storage,
    StorageError,
};
use std::cell::RefCell;

const TODAY: &str = "2026-08-27";

/// A Storage that remembers every path read through it.
struct Watched {
    files: MemoryStorage,
    reads: RefCell<Vec<String>>,
    probes: RefCell<Vec<String>>,
}

impl Watched {
    fn reads(&self) -> Vec<String> {
        self.reads.borrow().clone()
    }

    fn probes(&self) -> Vec<String> {
        self.probes.borrow().clone()
    }

    fn archived_reads(&self) -> Vec<String> {
        self.reads
            .borrow()
            .iter()
            .filter(|path| path.starts_with("archive/"))
            .cloned()
            .collect()
    }
}

impl Storage for Watched {
    fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
        self.files.list(dir)
    }

    fn exists(&self, path: &str) -> Result<bool, StorageError> {
        self.probes.borrow_mut().push(path.to_owned());
        self.files.exists(path)
    }

    fn read(&self, path: &str) -> Result<String, StorageError> {
        self.reads.borrow_mut().push(path.to_owned());
        self.files.read(path)
    }

    fn write(&mut self, path: &str, content: &str) -> Result<(), StorageError> {
        self.files.write(path, content)
    }

    fn remove(&mut self, path: &str) -> Result<(), StorageError> {
        self.files.remove(path)
    }
}

fn task(id: &str, state: &str, extra: &str) -> String {
    format!(
        "---\nid: {id}\ntype: task\nstate: {state}\ntitle: Task {id}\n{extra}created: 2026-08-10\nupdated: 2026-08-20\n---\n\nA body.\n"
    )
}

/// A notebook with `live` open Tasks and `archived` closed ones, none of
/// them named by anything live.
fn watched(live: usize, archived: usize) -> Watched {
    let mut written: Vec<(String, String)> = Vec::new();
    for index in 0..live {
        let id = format!("task.live-{index}");
        written.push((format!("tasks/{id}.md"), task(&id, "open", "")));
    }
    for index in 0..archived {
        let id = format!("task.filed-{index}");
        written.push((format!("archive/tasks/{id}.md"), task(&id, "closed", "")));
    }
    Watched {
        files: MemoryStorage::from_files(written),
        reads: RefCell::new(Vec::new()),
        probes: RefCell::new(Vec::new()),
    }
}

fn nothing_lost(_cited: &[CitedProof]) -> Vec<CitedProof> {
    Vec::new()
}

#[test]
fn a_query_about_live_work_opens_nothing_in_the_archive() {
    let mut storage = watched(3, 200);
    let notebook = Notebook::new(&mut storage);
    notebook.ready().unwrap();
    notebook.list().unwrap();
    notebook.view("task.live-0").unwrap();

    assert_eq!(
        storage.archived_reads(),
        Vec::<String>::new(),
        "the queue and the listing are about live work; history is known by name"
    );
}

#[test]
fn the_dashboard_opens_only_the_archived_records_live_ones_name() {
    // Two hundred archived Tasks, one of them a live Task's blocker: an
    // epic's progress is a count of its children, and its children are
    // archived as they settle.
    let mut storage = watched(1, 200);
    storage
        .write(
            "tasks/task.live-0.md",
            &task("task.live-0", "open", "blocked-by: task.filed-7\n"),
        )
        .unwrap();
    for query in ["overview", "status"] {
        storage.reads.borrow_mut().clear();
        let notebook = Notebook::new(&mut storage);
        if query == "overview" {
            notebook.overview().unwrap();
        } else {
            notebook
                .status(TODAY, Budget::Unbounded, nothing_lost, None)
                .unwrap();
        }
        assert_eq!(
            storage.archived_reads(),
            vec!["archive/tasks/task.filed-7.md"],
            "{query} opens the record an edge points at, once — what the \
             archive costs follows the live notebook's edges, not its own size"
        );
    }
}

#[test]
fn a_verb_that_judges_history_opens_it() {
    let mut storage = watched(1, 2);
    Notebook::new(&mut storage).check().unwrap();

    assert_eq!(
        storage.archived_reads(),
        vec![
            "archive/tasks/task.filed-0.md",
            "archive/tasks/task.filed-1.md"
        ],
        "verification reads every file it verifies"
    );
}

#[test]
fn a_dashboard_opens_each_record_it_reads_once() {
    let mut storage = watched(3, 2);
    // Two live Tasks wait on the same filed one, so the widening that finds
    // an epic's settled children runs beside the live pass — and the file
    // two records name is still one file.
    for id in ["task.live-0", "task.live-1"] {
        storage
            .write(
                &format!("tasks/{id}.md"),
                &task(id, "open", "blocked-by: task.filed-0\n"),
            )
            .unwrap();
    }
    Notebook::new(&mut storage)
        .status(TODAY, Budget::Unbounded, nothing_lost, None)
        .unwrap();

    let mut once = storage.reads();
    let read = once.len();
    once.sort();
    once.dedup();
    assert_eq!(
        read,
        once.len(),
        "a Status costs the notebook one pass, not one per section: {:?}",
        storage.reads()
    );
    assert_eq!(
        storage.archived_reads(),
        vec!["archive/tasks/task.filed-0.md"],
        "the widening opens the filed record a live one names, and no other"
    );
}

#[test]
fn the_widening_opens_each_filed_record_once_when_the_edges_loop() {
    // Two filed Tasks naming each other, hand-edited in: `blocked-by` cycles
    // are refused at write and `from` has no guard at all, so a walk that
    // followed them without remembering would never stop.
    let mut storage = watched(0, 0);
    for (id, other) in [("task.a", "task.b"), ("task.b", "task.a")] {
        storage
            .write(
                &format!("archive/tasks/{id}.md"),
                &task(
                    id,
                    "closed",
                    &format!("from: {other}\nblocked-by: {other}\n"),
                ),
            )
            .unwrap();
    }
    storage
        .write(
            "tasks/task.live.md",
            &task("task.live", "open", "blocked-by: task.a\n"),
        )
        .unwrap();

    Notebook::new(&mut storage).list_for("task.live").unwrap();

    let mut opened = storage.archived_reads();
    opened.sort();
    assert_eq!(
        opened,
        vec!["archive/tasks/task.a.md", "archive/tasks/task.b.md"],
        "each end of the loop is opened once, and the walk ends"
    );
}

/// The archive answers one question for the dashboard: where a record sits
/// inside an epic. Establishing that there is no epic costs the one hop a
/// hub would be named on; the lineage behind it is read only once a hub is
/// there to place a record in.
#[test]
fn a_dashboard_reads_a_lineage_only_once_an_epic_can_hold_it() {
    let mut storage = watched(0, 0);
    for (id, origin) in [
        ("task.child", "from: task.hub\n"),
        ("task.deeper", "from: task.child\n"),
    ] {
        storage
            .write(
                &format!("archive/tasks/{id}.md"),
                &task(id, "closed", origin),
            )
            .unwrap();
    }
    storage
        .write(
            "tasks/task.live.md",
            &task("task.live", "open", "from: task.deeper\n"),
        )
        .unwrap();

    Notebook::new(&mut storage)
        .status(TODAY, Budget::Unbounded, nothing_lost, None)
        .unwrap();
    assert_eq!(
        storage.archived_reads(),
        Vec::<String>::new(),
        "no live record names a child, so not even the one hop is owed"
    );

    storage
        .write(
            "tasks/task.hub.md",
            &task("task.hub", "open", "blocked-by: task.child\n"),
        )
        .unwrap();
    // A second live record naming the same child: the hop is owed once,
    // however many records name it.
    storage
        .write(
            "tasks/task.waiting.md",
            &task("task.waiting", "open", "blocked-by: task.child\n"),
        )
        .unwrap();
    storage.reads.borrow_mut().clear();
    Notebook::new(&mut storage)
        .status(TODAY, Budget::Unbounded, nothing_lost, None)
        .unwrap();
    let mut opened = storage.archived_reads();
    opened.sort();
    assert_eq!(
        opened,
        vec![
            "archive/tasks/task.child.md",
            "archive/tasks/task.deeper.md"
        ],
        "the hub's child, and the lineage that carries the live record to it"
    );
}

/// A listing is a snapshot: between it and the read, another process may
/// have archived or expunged the record. Reporting that as the reader's
/// own storage failure turns someone else's completed work into an error
/// on every read verb.
#[test]
fn a_record_filed_between_the_listing_and_the_read_is_skipped() {
    struct Vanishing {
        files: MemoryStorage,
        gone: String,
    }

    impl Storage for Vanishing {
        fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
            self.files.list(dir)
        }

        fn read(&self, path: &str) -> Result<String, StorageError> {
            if path == self.gone {
                return Err(StorageError::NotFound {
                    path: path.to_owned(),
                });
            }
            self.files.read(path)
        }

        fn write(&mut self, path: &str, content: &str) -> Result<(), StorageError> {
            self.files.write(path, content)
        }

        fn remove(&mut self, path: &str) -> Result<(), StorageError> {
            self.files.remove(path)
        }
    }

    let mut storage = Vanishing {
        files: watched(3, 0).files,
        gone: "tasks/task.live-1.md".to_owned(),
    };
    let rows = Notebook::new(&mut storage).list().unwrap();

    let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["task.live-0", "task.live-2"],
        "the rest of the listing still answers"
    );
}

#[test]
fn minting_an_id_opens_nothing_in_the_archive() {
    let mut storage = watched(3, 200);
    Notebook::new(&mut storage)
        .create(&Draft::new(RecordType::Task, "A fresh task"), TODAY)
        .unwrap();

    assert_eq!(
        storage.archived_reads(),
        Vec::<String>::new(),
        "a filed record claims the name on its file, and the listing carries it"
    );
}

/// An id a body cites is a name, and a name is answered by its place.
#[test]
fn an_id_a_body_cites_is_probed_not_opened() {
    let mut storage = watched(3, 200);

    Notebook::new(&mut storage)
        .comment(
            "task.live-0",
            None,
            "Waiting on task.live-1, filed under task.filed-0, unlike task.ghost.",
            TODAY,
        )
        .unwrap();

    assert_eq!(
        storage.reads(),
        vec!["tasks/task.live-0.md"],
        "the record being written is the only one whose bytes matter"
    );
    assert_eq!(
        storage.probes(),
        vec![
            "tasks/task.live-1.md",
            "tasks/task.filed-0.md",
            "archive/tasks/task.filed-0.md",
            "tasks/task.ghost.md",
            "archive/tasks/task.ghost.md",
        ],
        "each cited id is probed where it could sit, the live home first"
    );
}

#[test]
fn a_cycle_check_opens_the_chain_it_walks() {
    // The refused edge closes a two-Task loop; the rest of the live Tasks
    // and the whole archive have nothing to do with it.
    let mut storage = watched(20, 200);
    storage
        .write(
            "tasks/task.live-0.md",
            &task("task.live-0", "open", "blocked-by: task.live-1\n"),
        )
        .unwrap();

    let error = Notebook::new(&mut storage)
        .block("task.live-1", "task.live-0", TODAY)
        .unwrap_err();

    assert!(
        matches!(error, NotebookError::WouldCycle { .. }),
        "{error:?}"
    );
    assert_eq!(
        storage.reads(),
        vec!["tasks/task.live-1.md", "tasks/task.live-0.md"],
        "the record being edged and one step of the walk out of it \u{2014} the \
         notebook is never opened"
    );
}
