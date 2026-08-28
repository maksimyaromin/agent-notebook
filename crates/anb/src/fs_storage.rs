//! The filesystem adapter behind the Storage seam, and where the notebook
//! root lives on disk.
//!
//! Writes are atomic — a temp file in the target directory, then a rename —
//! so a reader never sees a half-written record.

use anb_core::{Storage, StorageError};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Where the notebook lives: the nearest ancestor of `start` already
/// carrying one, the search bounded by the repository — the notebook is
/// committed with its code, so a directory above `.git` is never read.
/// At the repository root the notebook appears on first write; outside any
/// repository, `start` itself hosts it.
#[must_use]
pub fn resolve_root(start: &Path) -> PathBuf {
    for dir in start.ancestors() {
        let root = dir.join(NOTEBOOK_DIR);
        if root.is_dir() || dir.join(".git").exists() {
            return root;
        }
    }
    start.join(NOTEBOOK_DIR)
}

const NOTEBOOK_DIR: &str = ".agent-notebook";

pub struct FsStorage {
    root: PathBuf,
}

impl FsStorage {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        FsStorage { root }
    }

    fn absolute(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }
}

impl Storage for FsStorage {
    fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
        let entries = match fs::read_dir(self.absolute(dir)) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(io_error(dir, &error)),
        };
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| io_error(dir, &error))?;
            if !entry.path().is_file() {
                continue;
            }
            // A name outside UTF-8 cannot be addressed through the seam's
            // string paths, so it cannot be a record's file.
            if let Some(name) = entry.file_name().to_str() {
                paths.push(format!("{dir}/{name}"));
            }
        }
        paths.sort();
        Ok(paths)
    }

    fn read(&self, path: &str) -> Result<String, StorageError> {
        let bytes = match fs::read(self.absolute(path)) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Err(StorageError::NotFound {
                    path: path.to_owned(),
                });
            }
            Err(error) => return Err(io_error(path, &error)),
        };
        String::from_utf8(bytes).map_err(|_| StorageError::Io {
            path: path.to_owned(),
            detail: "not valid UTF-8".to_owned(),
        })
    }

    fn write(&mut self, path: &str, content: &str) -> Result<(), StorageError> {
        let target = self.absolute(path);
        let directory = target.parent().expect("a joined path has a parent");
        fs::create_dir_all(directory).map_err(|error| io_error(path, &error))?;
        let temp = directory.join(format!(
            ".{}.{}.tmp",
            target
                .file_name()
                .expect("a record path names a file")
                .to_string_lossy(),
            std::process::id()
        ));
        fs::write(&temp, content).map_err(|error| io_error(path, &error))?;
        fs::rename(&temp, &target).map_err(|error| {
            let _ = fs::remove_file(&temp);
            io_error(path, &error)
        })
    }

    fn remove(&mut self, path: &str) -> Result<(), StorageError> {
        match fs::remove_file(self.absolute(path)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Err(StorageError::NotFound {
                path: path.to_owned(),
            }),
            Err(error) => Err(io_error(path, &error)),
        }
    }
}

fn io_error(path: &str, error: &std::io::Error) -> StorageError {
    StorageError::Io {
        path: path.to_owned(),
        detail: error.to_string(),
    }
}
