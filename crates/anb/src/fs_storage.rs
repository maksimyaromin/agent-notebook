//! The filesystem adapter behind the Storage seam, and where the notebook
//! root lives on disk.
//!
//! Writes are atomic — a temp file in the target directory, then a rename —
//! so a reader never sees a half-written record.
//!
//! A symlink is never a record, wherever it points: the seam cannot vouch
//! for a file it did not write, and following one would let a link
//! committed to a project decide what a later reader's `view` prints.

use anb_core::{ARCHIVE_DIR, RecordType, Storage, StorageError};
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

/// Why `root` cannot hold a notebook, when it cannot. A path that names a
/// file is not a directory to put records in, and the first write's "File
/// exists" would be a true error about the wrong thing.
#[must_use]
pub fn unusable_root(root: &Path) -> Option<String> {
    if root.exists() && !root.is_dir() {
        return Some(format!(
            "notebook: {} is a file, not a notebook directory",
            root.display()
        ));
    }
    record_directories()
        .map(|relative| root.join(relative))
        .find(|directory| is_symlink(directory))
        .map(|directory| {
            format!(
                "notebook: {} is a link, not a notebook directory",
                directory.display()
            )
        })
}

/// Every directory under the root that a record's path passes through.
///
/// The seam refuses a record that is a symlink, but it walks a directory
/// without asking: a `tasks` linked elsewhere and committed to a project
/// would have every listing read files the notebook never wrote, and every
/// write land outside it. The set is the notebook's own and settled, so
/// proving it costs one `lstat` each and no reading at all.
fn record_directories() -> impl Iterator<Item = String> {
    std::iter::once(ARCHIVE_DIR.to_owned()).chain(RecordType::ALL.into_iter().flat_map(|of| {
        let directory = of.directory();
        [directory.to_owned(), format!("{ARCHIVE_DIR}/{directory}")]
    }))
}

/// Name this run's leavings in the notebook's own ignore file: the lock a
/// writer takes and the temp file an interrupted write leaves behind belong
/// to a machine, not to the project's history.
///
/// A notebook that already states its own rules keeps them, and a root that
/// refuses the file works exactly as well — only git sees the difference.
pub fn ignore_leavings(root: &Path) {
    let _ = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(".gitignore"))
        .and_then(|mut file| std::io::Write::write_all(&mut file, ignored_names().as_bytes()));
}

/// The lock file's name in the notebook root. It carries no `.md` suffix
/// and sits outside every type directory, so no listing, query, or check
/// can mistake it for a record.
pub const LOCK_FILE: &str = ".lock";

/// The suffix on the temp file [`FsStorage::write`] renames from.
const TEMP_SUFFIX: &str = "tmp";

/// What a notebook committed with its project must not carry: the two
/// names a run leaves behind, each written by the code above it.
fn ignored_names() -> String {
    format!("# A run's leavings, not the project's history.\n{LOCK_FILE}\n*.{TEMP_SUFFIX}\n")
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

    /// A failure names the file as the filesystem knows it, root included:
    /// a seam path alone leaves the reader guessing which notebook it was.
    fn failed(&self, path: &str, error: &std::io::Error) -> StorageError {
        StorageError::Io {
            path: self.absolute(path).display().to_string(),
            detail: error.to_string(),
        }
    }
}

impl Storage for FsStorage {
    fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
        let entries = match fs::read_dir(self.absolute(dir)) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(self.failed(dir, &error)),
        };
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| self.failed(dir, &error))?;
            if !names_a_file(&entry) {
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
        let absolute = self.absolute(path);
        if is_symlink(&absolute) {
            return Err(StorageError::NotFound {
                path: path.to_owned(),
            });
        }
        let bytes = match fs::read(&absolute) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Err(StorageError::NotFound {
                    path: path.to_owned(),
                });
            }
            Err(error) => return Err(self.failed(path, &error)),
        };
        String::from_utf8(bytes).map_err(|_| StorageError::NotUtf8 {
            path: path.to_owned(),
        })
    }

    fn write(&mut self, path: &str, content: &str) -> Result<(), StorageError> {
        let target = self.absolute(path);
        if is_symlink(&target) {
            return Err(self.failed(path, &symlink_refused()));
        }
        let directory = target.parent().expect("a joined path has a parent");
        fs::create_dir_all(directory).map_err(|error| self.failed(path, &error))?;
        let mut pending = Pending::beside(&target);
        fs::write(&pending.path, content).map_err(|error| self.failed(path, &error))?;
        fs::rename(&pending.path, &target).map_err(|error| self.failed(path, &error))?;
        pending.persisted = true;
        Ok(())
    }

    fn remove(&mut self, path: &str) -> Result<(), StorageError> {
        match fs::remove_file(self.absolute(path)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Err(StorageError::NotFound {
                path: path.to_owned(),
            }),
            Err(error) => Err(self.failed(path, &error)),
        }
    }

    /// The filesystem answers this from the inode, so an id's existence
    /// costs a `stat` rather than the record it names.
    fn exists(&self, path: &str) -> Result<bool, StorageError> {
        match fs::symlink_metadata(self.absolute(path)) {
            Ok(found) => Ok(found.is_file()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(self.failed(path, &error)),
        }
    }
}

/// Whether the entry names a file to read. The kind usually arrives with
/// the listing, so on most platforms this costs nothing to ask.
fn names_a_file(entry: &fs::DirEntry) -> bool {
    entry.file_type().is_ok_and(|kind| kind.is_file())
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|found| found.is_symlink())
}

fn symlink_refused() -> std::io::Error {
    std::io::Error::new(ErrorKind::InvalidInput, "a symlink is not a record")
}

/// The file a write lands in before it is renamed into place. It removes
/// itself unless the rename claimed it, so no failure — a full disk, a
/// quota, a killed process mid-call — leaves a stray behind.
///
/// The bytes go through `fs::write`, which creates with the umask the user
/// set, because a notebook is committed to a project and read by whoever
/// reads the project.
struct Pending {
    path: PathBuf,
    persisted: bool,
}

impl Pending {
    /// Beside the target, so the rename stays within one filesystem and
    /// therefore stays atomic. One writer per process holds one of these at
    /// a time, so the pid is name enough.
    fn beside(target: &Path) -> Pending {
        let directory = target.parent().unwrap_or(Path::new("."));
        let name = target.file_name().unwrap_or_default().to_string_lossy();
        Pending {
            path: directory.join(format!(".{name}.{}.{TEMP_SUFFIX}", std::process::id())),
            persisted: false,
        }
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        if !self.persisted {
            let _ = fs::remove_file(&self.path);
        }
    }
}
