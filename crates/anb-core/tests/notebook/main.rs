//! The notebook at the Storage seam: strings in, exact strings and
//! returned models out. Write-time invariants and the queries derived over
//! them are specified together, because a write is only ever observable
//! through what a later read answers. Expected file bytes derive from the
//! format's canonical form, never from running the code.

mod archive;
mod budget;
mod check;
mod creation;
mod dependencies;
mod edit;
mod lifecycle;
mod queries;
mod restore;
mod status;

use anb_core::{
    Blocker, Budget, CitedProof, DebtSignal, Draft, Edit, Filter, FindingCode, Link, MemoryStorage,
    Notebook, NotebookError, Proof, RecordType, Repair, Storage, StorageError, Transitioned,
};

const TODAY: &str = "2026-08-27";

/// A Status asked with nothing to settle against the world outside the
/// notebook: the host finds every cited proof still there.
fn no_lost_proofs(_cited: &[CitedProof]) -> Vec<CitedProof> {
    Vec::new()
}

fn task_file(state: &str, extra_lines: &[&str]) -> String {
    record_file("task.demo", "task", state, extra_lines, "")
}

fn record_file(id: &str, type_word: &str, state: &str, extra_lines: &[&str], body: &str) -> String {
    let mut text =
        format!("---\nid: {id}\ntype: {type_word}\nstate: {state}\ntitle: A demo record\n");
    for line in extra_lines {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str("created: 2026-08-24\nupdated: 2026-08-25\n---\n");
    text.push_str(body);
    text
}

/// The listing narrowed to one hub's scope.
fn within(hub: &str) -> Filter {
    Filter {
        hub: Some(hub.to_owned()),
        ..Filter::default()
    }
}

fn storage_with(files: &[(&str, &str)]) -> MemoryStorage {
    MemoryStorage::from_files(files.iter().map(|(path, text)| (*path, *text)))
}

fn moved(id: &str, from: &'static str, to: &'static str) -> Transitioned {
    Transitioned {
        id: id.to_owned(),
        from,
        to,
        already: false,
    }
}

/// [`MemoryStorage`] whose `remove` always fails — the crash between a
/// move's write and its remove.
struct RemoveFails(MemoryStorage);

impl Storage for RemoveFails {
    fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
        self.0.list(dir)
    }
    fn read(&self, path: &str) -> Result<String, StorageError> {
        self.0.read(path)
    }
    fn write(&mut self, path: &str, content: &str) -> Result<(), StorageError> {
        self.0.write(path, content)
    }
    fn remove(&mut self, path: &str) -> Result<(), StorageError> {
        Err(StorageError::Io {
            path: path.to_owned(),
            detail: "refused".to_owned(),
        })
    }
}

/// A root that is there and cannot be served: a medium that names no
/// missing file, only a failure. What a notebook read behind another does
/// with it is what every case posing it asks.
struct UnreadableNotebook;

impl Storage for UnreadableNotebook {
    fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
        Err(Self::failure(dir))
    }

    fn read(&self, path: &str) -> Result<String, StorageError> {
        Err(Self::failure(path))
    }

    fn write(&mut self, path: &str, _content: &str) -> Result<(), StorageError> {
        Err(Self::failure(path))
    }

    fn remove(&mut self, path: &str) -> Result<(), StorageError> {
        Err(Self::failure(path))
    }
}

impl UnreadableNotebook {
    fn failure(path: &str) -> StorageError {
        StorageError::Io {
            path: path.to_owned(),
            detail: "the medium answered nothing".to_owned(),
        }
    }
}

/// [`MemoryStorage`] holds strings, so the adapter's duty is simulated:
/// the marked paths answer reads with [`StorageError::NotUtf8`].
struct BinaryHolding {
    inner: MemoryStorage,
    binary: Vec<String>,
}

impl BinaryHolding {
    fn with_binary_at(path: &str, files: &[(&str, &str)]) -> Self {
        let mut all: Vec<(&str, &str)> = files.to_vec();
        all.push((path, ""));
        BinaryHolding {
            inner: MemoryStorage::from_files(all),
            binary: vec![path.to_owned()],
        }
    }
}

impl Storage for BinaryHolding {
    fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
        self.inner.list(dir)
    }

    fn read(&self, path: &str) -> Result<String, StorageError> {
        if self.binary.iter().any(|held| held == path) {
            return Err(StorageError::NotUtf8 {
                path: path.to_owned(),
            });
        }
        self.inner.read(path)
    }

    fn write(&mut self, path: &str, content: &str) -> Result<(), StorageError> {
        self.inner.write(path, content)
    }

    fn remove(&mut self, path: &str) -> Result<(), StorageError> {
        self.inner.remove(path)
    }
}
