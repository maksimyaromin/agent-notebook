//! The filesystem adapter behind the Storage seam, and where the notebook
//! root lives on disk.
//!
//! Writes sync a new temporary file before replacing the target. Unix
//! hosts also sync directory entries, including newly created parents.
//! A sync error after rename can mean the write is visible but not durable;
//! callers reconcile exact bytes before retrying a multi-file operation.
//! Private `.tmp` directories exclude their contents from Git before a
//! metadata write, independently of the notebook's own ignore rules.
//!
//! A symlink is never a record, wherever it points: the seam cannot vouch
//! for a file it did not write, and following one would let a link
//! committed to a project decide what a later reader's `show` prints.

use anb_core::{ARCHIVE_DIR, RecordType, Storage, StorageError};
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Private practices for a local project. Linked worktrees use their common
/// Git directory as identity; unrelated checkouts and non-Git directories do
/// not share personal preferences just because their names match.
///
/// # Errors
/// A project path that cannot be resolved.
pub fn personal_root(project: &Path, user: &Path) -> Result<PathBuf, StorageError> {
    let anchor = project_anchor(project);
    let identity = crate::git::common_dir(anchor).unwrap_or_else(|| anchor.to_path_buf());
    let identity = identity.canonicalize().map_err(|error| StorageError::Io {
        path: identity.display().to_string(),
        detail: error.to_string(),
    })?;
    let digest = Sha256::digest(identity.as_os_str().as_encoded_bytes());
    let mut key = String::with_capacity(64);
    for byte in digest {
        let _ = write!(key, "{byte:02x}");
    }
    Ok(user.join("projects").join(key))
}

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

/// The notebook root for this call: the one the flags name, else the one
/// the environment names, else [`resolve_root`]'s default. A notebook may
/// sit beside the code and be committed with it, hide in a git-ignored
/// corner, or live outside the repository altogether.
///
/// `--notebook` and `--global` name a root two ways and share the flag
/// rung, so a call using both is refused rather than ranked. `--global`
/// names the user's own notebook, `NOTEBOOK_DIR` in `home`, which is the
/// same directory a call made from the home directory would find.
///
/// A named path anchors differently from `from_env`, because they are typed
/// at different moments: a flag arrives with a known working directory and
/// is read from there, while `ANB_NOTEBOOK` is exported once and outlives
/// every `cd`, so it is read from the project. Anchoring the variable on
/// the working directory would make one export mean a different notebook in
/// every directory. `home` is exported the same way, and is required
/// absolute for the same reason.
///
/// # Errors
/// The reason, when both flags name a root, or when `--global` has no
/// absolute home to name one in.
pub fn notebook_root(
    start: &Path,
    named: Option<&Path>,
    global: bool,
    home: Option<&Path>,
    from_env: Option<&OsStr>,
) -> Result<PathBuf, String> {
    if global {
        if named.is_some() {
            return Err("notebook: pass exactly one of --notebook <path>, --global".to_owned());
        }
        return user_root(home);
    }
    if let Some(named) = named {
        return Ok(start.join(named));
    }
    Ok(match from_env.filter(|chosen| !chosen.is_empty()) {
        Some(chosen) => project_anchor(start).join(chosen),
        None => resolve_root(start),
    })
}

/// The user's own notebook root inside `home`: the notebook `--global`
/// names, and the one Recall may include as an explicit personal audience.
///
/// # Errors
/// The reason, when there is no absolute home to name one in.
pub fn user_root(home: Option<&Path>) -> Result<PathBuf, String> {
    let Some(home) = home else {
        return Err(
            "notebook: --global names a notebook in the home directory, and this run knows no home"
                .to_owned(),
        );
    };
    if !home.is_absolute() {
        return Err(format!(
            "notebook: the home directory is not an absolute path: {}",
            home.display()
        ));
    }
    Ok(home.join(NOTEBOOK_DIR))
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
        .chain(std::iter::once(".migrations.tmp".to_owned()))
        .chain(std::iter::once(crate::session::DIRECTORY.to_owned()))
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

    fn ignore_private_directory(&self, path: &str) -> Result<(), StorageError> {
        let Some(directory) = private_directory(path) else {
            return Ok(());
        };
        let absolute = self.absolute(directory);
        if is_symlink(&absolute) {
            return Err(self.failed(directory, &symlink_refused()));
        }
        let ignore = format!("{directory}/.gitignore");
        match self.read(&ignore) {
            Ok(content) if matches!(content.as_str(), "*" | "*\n") => Ok(()),
            Ok(_) => Err(self.failed(&ignore, &std::io::Error::new(
                ErrorKind::InvalidInput,
                "private metadata needs an internal .gitignore containing only '*'; preserve and inspect the existing file",
            ))),
            Err(StorageError::NotFound { .. }) => self.write_atomically(&ignore, "*\n"),
            Err(error) => Err(error),
        }
    }

    fn write_atomically(&self, path: &str, content: &str) -> Result<(), StorageError> {
        let target = self.absolute(path);
        if is_symlink(&target) {
            return Err(self.failed(path, &symlink_refused()));
        }
        let directory = target.parent().expect("a joined path has a parent");
        create_directory_tree(directory).map_err(|error| self.failed(path, &error))?;
        let permissions = match fs::metadata(&target) {
            Ok(metadata) => Some(metadata.permissions()),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => return Err(self.failed(path, &error)),
        };
        let (mut pending, mut file) =
            Pending::create(&target).map_err(|error| self.failed(path, &error))?;
        if let Some(permissions) = permissions {
            file.set_permissions(permissions)
                .map_err(|error| self.failed(path, &error))?;
        }
        file.write_all(content.as_bytes())
            .map_err(|error| self.failed(path, &error))?;
        file.sync_all().map_err(|error| self.failed(path, &error))?;
        fs::rename(&pending.path, &target).map_err(|error| self.failed(path, &error))?;
        pending.persisted = true;
        sync_directory(directory).map_err(|error| self.failed(path, &error))
    }
}

impl Storage for FsStorage {
    fn list(&self, dir: &str) -> Result<Vec<String>, StorageError> {
        let entries = match fs::read_dir(self.absolute(dir)) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(self.failed(dir, &error)),
        };
        let private = private_directory(&format!("{dir}/")).is_some();
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| self.failed(dir, &error))?;
            if private && is_symlink(&entry.path()) {
                return Err(StorageError::Io {
                    path: entry.path().display().to_string(),
                    detail: "a symlink is not private metadata".to_owned(),
                });
            }
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
            if private_directory(path).is_some() {
                return Err(self.failed(path, &symlink_refused()));
            }
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
        self.ignore_private_directory(path)?;
        self.write_atomically(path, content)
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

fn private_directory(path: &str) -> Option<&str> {
    path.split_once('/')
        .map(|(directory, _)| directory)
        .filter(|directory| {
            directory.eq_ignore_ascii_case(".tmp")
                || Path::new(directory)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("tmp"))
        })
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|found| found.is_symlink())
}

fn symlink_refused() -> std::io::Error {
    std::io::Error::new(ErrorKind::InvalidInput, "a symlink is not a record")
}

/// A temporary file owned by one write. Error unwinding removes it unless
/// the rename committed; abrupt process termination may leave an ignored sibling.
///
/// A new file uses the process umask; a replacement receives its target's
/// permissions before it becomes visible.
struct Pending {
    path: PathBuf,
    persisted: bool,
}

impl Pending {
    fn create(target: &Path) -> std::io::Result<(Pending, fs::File)> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory = target.parent().unwrap_or(Path::new("."));
        let name = target.file_name().unwrap_or_default().to_string_lossy();
        loop {
            let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = directory.join(format!(
                ".{name}.{}.{sequence}.{TEMP_SUFFIX}",
                std::process::id()
            ));
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(file) => {
                    return Ok((
                        Pending {
                            path,
                            persisted: false,
                        },
                        file,
                    ));
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
    }
}

/// Sync each new directory's entry before a later record write depends on it.
pub(crate) fn create_directory_tree(directory: &Path) -> std::io::Result<()> {
    let missing: Vec<&Path> = directory
        .ancestors()
        .filter(|path| !path.as_os_str().is_empty())
        .take_while(|path| !path.exists())
        .collect();
    fs::create_dir_all(directory)?;
    for created in missing.iter().rev() {
        sync_directory(
            created
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or(Path::new(".")),
        )?;
    }
    Ok(())
}

#[cfg(unix)]
fn sync_directory(directory: &Path) -> std::io::Result<()> {
    fs::File::open(directory)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_directory: &Path) -> std::io::Result<()> {
    Ok(())
}

impl Drop for Pending {
    fn drop(&mut self) {
        if !self.persisted {
            let _ = fs::remove_file(&self.path);
        }
    }
}
