//! The Storage seam: the only path between the Core and data.
//!
//! The Core never touches the filesystem, git, or the network. It is handed an
//! implementation of [`Storage`] and a notebook root, and speaks to it in
//! strings and relative paths only. The Core ships only the in-memory adapter;
//! every host brings its own.

use std::collections::BTreeMap;

/// A path relative to the notebook root, e.g. `tasks/parser-fences.md`.
///
/// Always `/`-separated, never absolute, never containing `.` or `..`
/// components. Adapters are responsible for mapping it to their medium.
pub type RelPath = str;

/// Errors an adapter may report. The Core treats them as opaque outcomes;
/// it never retries and never inspects platform detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    NotFound {
        path: String,
    },
    /// The file exists but its bytes are not UTF-8, so it cannot cross the
    /// seam as a string. Only an adapter sees raw bytes, so only an adapter
    /// can report this.
    NotUtf8 {
        path: String,
    },
    /// `detail` is human-readable and adapter-specific.
    Io {
        path: String,
        detail: String,
    },
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::NotFound { path } => write!(f, "not found: {path}"),
            StorageError::NotUtf8 { path } => write!(f, "not UTF-8: {path}"),
            StorageError::Io { path, detail } => write!(f, "storage error on {path}: {detail}"),
        }
    }
}

impl std::error::Error for StorageError {}

/// The object handed to the Core, and the only path to data.
///
/// No command may bypass it: tests feed strings in and assert strings and
/// returned models out at exactly this seam.
pub trait Storage {
    /// List the paths directly under `dir` (non-recursive), sorted, each
    /// keeping its `dir/` prefix. A missing directory is an empty listing,
    /// not an error.
    ///
    /// # Errors
    /// [`StorageError::Io`] when the adapter cannot enumerate `dir`.
    fn list(&self, dir: &RelPath) -> Result<Vec<String>, StorageError>;

    /// Read the full contents of the file at `path`.
    ///
    /// # Errors
    /// [`StorageError::NotFound`] when `path` does not exist,
    /// [`StorageError::Io`] on any other adapter failure.
    fn read(&self, path: &RelPath) -> Result<String, StorageError>;

    /// Write `content` to `path`, creating parents as needed and replacing
    /// any existing contents atomically from the Core's point of view.
    ///
    /// # Errors
    /// [`StorageError::Io`] when the adapter cannot persist the write.
    fn write(&mut self, path: &RelPath, content: &str) -> Result<(), StorageError>;

    /// Remove the file at `path`. Removing a missing path is an error:
    /// the Core moves records (archive), it never blind-deletes.
    ///
    /// # Errors
    /// [`StorageError::NotFound`] when `path` does not exist,
    /// [`StorageError::Io`] on any other adapter failure.
    fn remove(&mut self, path: &RelPath) -> Result<(), StorageError>;
}

/// In-memory adapter. It lives in the library, not behind `cfg(test)`:
/// a host wanting an ephemeral notebook uses it as-is.
#[derive(Debug, Default, Clone)]
pub struct MemoryStorage {
    files: BTreeMap<String, String>,
}

impl MemoryStorage {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn from_files<I, P, C>(files: I) -> Self
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<String>,
        C: Into<String>,
    {
        Self {
            files: files
                .into_iter()
                .map(|(p, c)| (p.into(), c.into()))
                .collect(),
        }
    }
}

impl Storage for MemoryStorage {
    fn list(&self, dir: &RelPath) -> Result<Vec<String>, StorageError> {
        let prefix = if dir.is_empty() {
            String::new()
        } else {
            format!("{}/", dir.trim_end_matches('/'))
        };
        Ok(self
            .files
            .keys()
            .filter(|path| {
                path.strip_prefix(&prefix)
                    .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
            })
            .cloned()
            .collect())
    }

    fn read(&self, path: &RelPath) -> Result<String, StorageError> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| StorageError::NotFound {
                path: path.to_owned(),
            })
    }

    fn write(&mut self, path: &RelPath, content: &str) -> Result<(), StorageError> {
        self.files.insert(path.to_owned(), content.to_owned());
        Ok(())
    }

    fn remove(&mut self, path: &RelPath) -> Result<(), StorageError> {
        if self.files.remove(path).is_some() {
            Ok(())
        } else {
            Err(StorageError::NotFound {
                path: path.to_owned(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_is_non_recursive_and_sorted() {
        let storage = MemoryStorage::from_files([
            ("tasks/b.md", "b"),
            ("tasks/a.md", "a"),
            ("tasks/archive/z.md", "z"),
            ("notes/n.md", "n"),
        ]);
        assert_eq!(
            storage.list("tasks").unwrap(),
            vec!["tasks/a.md", "tasks/b.md"]
        );
    }

    #[test]
    fn list_of_missing_dir_is_empty() {
        let storage = MemoryStorage::new();
        assert_eq!(storage.list("tasks").unwrap(), Vec::<String>::new());
    }

    #[test]
    fn read_missing_is_not_found() {
        let storage = MemoryStorage::new();
        assert_eq!(
            storage.read("tasks/absent.md"),
            Err(StorageError::NotFound {
                path: "tasks/absent.md".into()
            })
        );
    }

    #[test]
    fn remove_missing_is_not_found() {
        let mut storage = MemoryStorage::new();
        assert!(matches!(
            storage.remove("tasks/absent.md"),
            Err(StorageError::NotFound { .. })
        ));
    }
}
