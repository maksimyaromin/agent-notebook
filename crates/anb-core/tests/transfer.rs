use anb_core::{MemoryStorage, Notebook, NotebookError, RecordFile, Storage, StorageError};
use std::collections::BTreeMap;

fn note(id: &str, state: &str, extra: &str) -> String {
    format!(
        "---\nid: note.{id}\ntype: note\nstate: {state}\ntitle: Source: {id}\nby: Historical author\ncreated: 2019-02-03\nupdated: 2020-04-05\n{extra}---\nOriginal evidence.\r\nNo final newline"
    )
}

#[test]
fn import_checks_the_prospective_union_without_writing_and_preserves_source_history() {
    let source = MemoryStorage::from_files([
        (
            "notes/note.first.md",
            note(
                "first",
                "active",
                "link: basis note.second\ncustom: Original: value\n",
            ),
        ),
        (
            "archive/notes/note.second.md",
            note("second", "retired", ""),
        ),
        ("config", "scope: mine\n".to_owned()),
    ]);
    let mut target = MemoryStorage::from_files([("config", "scope: team\n")]);
    let preview = Notebook::new(&mut target).import(&source, true).unwrap();
    assert!(preview.check);
    assert_eq!(
        preview.paths,
        ["archive/notes/note.second.md", "notes/note.first.md"]
    );
    assert!(target.list("notes").unwrap().is_empty());
    let imported = Notebook::new(&mut target).import(&source, false).unwrap();
    assert_eq!(imported.paths, preview.paths);
    let first = target.read("notes/note.first.md").unwrap();
    let record = RecordFile::parse(&first);
    assert_eq!(record.field("created"), Some("2019-02-03"));
    assert_eq!(record.field("updated"), Some("2020-04-05"));
    assert_eq!(record.field("by"), Some("Historical author"));
    assert_eq!(record.field("custom"), Some("Original: value"));
    assert_eq!(record.body(), "Original evidence.\r\nNo final newline");
    assert_eq!(target.read("config").unwrap(), "scope: team\n");
    assert!(target.exists("archive/notes/note.second.md").unwrap());
    assert_eq!(
        Notebook::new(&mut target)
            .import(&source, false)
            .unwrap()
            .unchanged,
        2
    );
}

#[test]
fn an_import_resolves_references_against_existing_target_records() {
    let source = MemoryStorage::from_files([(
        "notes/note.first.md",
        note("first", "active", "link: basis note.second\n"),
    )]);
    let mut target =
        MemoryStorage::from_files([("notes/note.second.md", note("second", "active", ""))]);
    assert_eq!(
        Notebook::new(&mut target)
            .import(&source, false)
            .unwrap()
            .paths,
        ["notes/note.first.md"]
    );
}

#[test]
fn import_refuses_an_id_collision_before_creating_any_record() {
    let original = note("second", "active", "custom: Target\n");
    let source = MemoryStorage::from_files([
        ("notes/note.first.md", note("first", "active", "")),
        (
            "archive/notes/note.second.md",
            note("second", "retired", ""),
        ),
    ]);
    let mut target = MemoryStorage::from_files([("notes/note.second.md", original.clone())]);
    assert!(
        matches!(Notebook::new(&mut target).import(&source, false), Err(NotebookError::DuplicateId { id, .. }) if id == "note.second")
    );
    assert!(!target.exists("notes/note.first.md").unwrap());
    assert_eq!(target.read("notes/note.second.md").unwrap(), original);
}

#[test]
fn import_refuses_a_dangling_source_reference_before_creating_any_record() {
    let source = MemoryStorage::from_files([
        ("notes/note.first.md", note("first", "active", "")),
        (
            "notes/note.second.md",
            note("second", "active", "link: basis note.missing\n"),
        ),
    ]);
    let mut target = MemoryStorage::new();
    assert!(matches!(
        Notebook::new(&mut target).import(&source, false),
        Err(NotebookError::InvalidRecord { .. })
    ));
    assert!(target.list("notes").unwrap().is_empty());
}

#[test]
fn migration_previews_without_writes_then_keeps_originals_outside_record_directories() {
    let original = note("first", "active", "custom: Original: value\n");
    let mut storage = MemoryStorage::from_files([("notes/note.first.md", original.clone())]);
    let preview = Notebook::new(&mut storage).migrate(true).unwrap();
    assert_eq!(preview.paths, ["notes/note.first.md"]);
    assert_eq!(storage.read("notes/note.first.md").unwrap(), original);
    assert!(storage.list(".migrations.tmp").unwrap().is_empty());
    let migrated = Notebook::new(&mut storage).migrate(false).unwrap();
    let journal: BTreeMap<String, String> =
        serde_json::from_str(&storage.read(migrated.backup.as_ref().unwrap()).unwrap()).unwrap();
    assert_eq!(journal["notes/note.first.md"], original);
    let canonical = storage.read("notes/note.first.md").unwrap();
    assert_eq!(
        RecordFile::parse(&canonical).body(),
        RecordFile::parse(&original).body()
    );
    assert_eq!(
        RecordFile::parse(&canonical).field("updated"),
        Some("2020-04-05")
    );
    let replay = Notebook::new(&mut storage).migrate(false).unwrap();
    assert!(replay.paths.is_empty());
    assert!(replay.backup.is_none());
    assert_eq!(replay.unchanged, 1);
}

struct FailingStorage {
    inner: MemoryStorage,
    remaining: usize,
    commit_failure: bool,
}

impl Storage for FailingStorage {
    fn list(&self, path: &str) -> Result<Vec<String>, StorageError> {
        self.inner.list(path)
    }
    fn read(&self, path: &str) -> Result<String, StorageError> {
        self.inner.read(path)
    }
    fn remove(&mut self, path: &str) -> Result<(), StorageError> {
        self.inner.remove(path)
    }
    fn write(&mut self, path: &str, text: &str) -> Result<(), StorageError> {
        if self.remaining == 0 {
            if self.commit_failure {
                self.inner.write(path, text)?;
            }
            return Err(StorageError::Io {
                path: path.to_owned(),
                detail: "injected write failure".to_owned(),
            });
        }
        self.remaining -= 1;
        self.inner.write(path, text)
    }
}

#[test]
fn interrupted_migrations_keep_every_original_and_resume_at_every_write_boundary() {
    let originals = BTreeMap::from([
        (
            "notes/note.first.md".to_owned(),
            note("first", "active", ""),
        ),
        (
            "archive/notes/note.second.md".to_owned(),
            note("second", "retired", ""),
        ),
    ]);
    for (remaining, commit_failure) in
        (0..4).flat_map(|remaining| [false, true].map(|committed| (remaining, committed)))
    {
        let mut storage = FailingStorage {
            inner: MemoryStorage::from_files(originals.clone()),
            remaining,
            commit_failure,
        };
        assert!(Notebook::new(&mut storage).migrate(false).is_err());
        if remaining == 0 {
            for (path, text) in &originals {
                assert_eq!(storage.read(path).unwrap(), *text);
            }
        }
        if remaining > 0 || commit_failure {
            let journal = storage
                .list(".migrations.tmp")
                .unwrap()
                .into_iter()
                .find(|path| {
                    path.rsplit_once('.')
                        .is_some_and(|(_, extension)| extension == "json")
                })
                .unwrap();
            let saved: BTreeMap<String, String> =
                serde_json::from_str(&storage.read(&journal).unwrap()).unwrap();
            assert_eq!(saved, originals);
        }
        storage.remaining = usize::MAX;
        Notebook::new(&mut storage).migrate(false).unwrap();
        for (path, text) in &originals {
            assert_eq!(
                storage.read(path).unwrap(),
                RecordFile::parse(text).normalize()
            );
        }
    }
}

#[test]
fn a_resumed_migration_refuses_a_record_changed_since_its_journal_was_written() {
    let mut storage = FailingStorage {
        inner: MemoryStorage::from_files([("notes/note.first.md", note("first", "active", ""))]),
        remaining: 1,
        commit_failure: false,
    };
    assert!(Notebook::new(&mut storage).migrate(false).is_err());
    let correction = note("first", "active", "custom: A later correction\n");
    storage
        .inner
        .write("notes/note.first.md", &correction)
        .unwrap();
    storage.remaining = usize::MAX;
    assert!(
        matches!(Notebook::new(&mut storage).migrate(false), Err(NotebookError::InvalidArgument { reason }) if reason.contains("changed since migration"))
    );
    assert_eq!(storage.read("notes/note.first.md").unwrap(), correction);
}

#[test]
fn an_interrupted_import_can_be_retried_without_overwriting_existing_records() {
    let source = MemoryStorage::from_files([
        ("notes/note.first.md", note("first", "active", "")),
        ("notes/note.second.md", note("second", "active", "")),
    ]);
    let mut target = FailingStorage {
        inner: MemoryStorage::new(),
        remaining: 1,
        commit_failure: false,
    };
    assert!(Notebook::new(&mut target).import(&source, false).is_err());
    target.remaining = usize::MAX;
    let replay = Notebook::new(&mut target).import(&source, false).unwrap();
    assert_eq!(replay.unchanged, 1);
    assert_eq!(replay.paths, ["notes/note.second.md"]);
}
