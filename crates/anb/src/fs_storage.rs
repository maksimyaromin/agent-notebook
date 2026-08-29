//! The filesystem adapter behind the Storage seam, and where the notebook
//! root lives on disk.
//!
//! Writes are atomic — a temp file in the target directory, then a rename —
//! so a reader never sees a half-written record.

use anb_core::{Storage, StorageError};
use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// The project `start` belongs to: the nearest ancestor already holding a
/// notebook or a repository, else `start` itself. A directory above `.git`
/// belongs to another project. It is what a relative path in the notebook
/// is read from — the working directory of the moment is not stable enough
/// to mean anything to a later reader.
#[must_use]
pub fn project_anchor(start: &Path) -> &Path {
    for dir in start.ancestors() {
        if dir.join(NOTEBOOK_DIR).is_dir() || dir.join(".git").exists() {
            return dir;
        }
    }
    start
}

/// Where the notebook lives when nobody names one: `.agent-notebook` in the
/// project `start` belongs to. At the repository root it appears on first
/// write; outside any repository, `start` itself hosts it.
#[must_use]
pub fn resolve_root(start: &Path) -> PathBuf {
    project_anchor(start).join(NOTEBOOK_DIR)
}

/// The notebook root for this call: the path the caller named, else the one
/// the environment names, else [`resolve_root`]'s default. A notebook may
/// sit beside the code and be committed with it, hide in a git-ignored
/// corner, or live outside the repository altogether.
///
/// A relative path anchors differently in the two, because they are typed
/// at different moments: a flag arrives with a known working directory and
/// is read from there, while `ANB_NOTEBOOK` is exported once and outlives
/// every `cd`, so it is read from the project. Anchoring the variable on
/// the working directory would make one export mean a different notebook in
/// every directory.
#[must_use]
pub fn notebook_root(start: &Path, named: Option<&Path>, from_env: Option<&OsStr>) -> PathBuf {
    if let Some(named) = named {
        return start.join(named);
    }
    match from_env.filter(|chosen| !chosen.is_empty()) {
        Some(chosen) => project_anchor(start).join(chosen),
        None => resolve_root(start),
    }
}

pub const NOTEBOOK_ENV: &str = "ANB_NOTEBOOK";

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
        String::from_utf8(bytes).map_err(|_| StorageError::NotUtf8 {
            path: path.to_owned(),
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
