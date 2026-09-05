//! The skills the binary installs into a project, one directory each under
//! a host's `skills/`: [`anb`], rendered from the binary so it cannot drift,
//! and [`atlas`], written by hand and carried in the binary. This module owns
//! what the two share: the frontmatter mark `setup` rewrites by, the list of
//! installables, and writing a skill's files into a directory or diffing them
//! against it.

use anb_core::StorageError;
use std::fs;
use std::path::Path;

pub mod anb;
pub mod atlas;

/// The frontmatter line by which `setup` knows a skill file as its own. A
/// user who deletes the line owns the file from then on: setup rewrites only
/// what still carries it.
pub const MANAGED_MARK: &str = "managed-by: anb";

/// A skill `setup` installs: its directory name under a host's `skills/`
/// and its files, paths relative to that directory.
pub struct Installable {
    pub name: &'static str,
    pub files: Vec<(&'static str, String)>,
}

/// Both skills the binary carries: the one rendered from itself and the
/// atlas, written by hand.
#[must_use]
pub fn installable() -> [Installable; 2] {
    let rendered = anb::render();
    [
        Installable {
            name: anb::NAME,
            files: rendered
                .files()
                .iter()
                .map(|(file, text)| (*file, (*text).to_owned()))
                .collect(),
        },
        Installable {
            name: atlas::NAME,
            files: atlas::files()
                .iter()
                .map(|(file, text)| (*file, (*text).to_owned()))
                .collect(),
        },
    ]
}

/// Which committed file differs from the rendering.
#[derive(Debug, PartialEq, Eq)]
pub struct Drift {
    pub file: &'static str,
    pub reason: &'static str,
}

/// Write a skill's files, paths relative to `dir`.
///
/// # Errors
/// The storage failure of the write.
pub fn write_into(dir: &Path, files: &[(&'static str, &str)]) -> Result<(), StorageError> {
    for &(file, text) in files {
        let path = dir.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| io_failure(parent, &error))?;
        }
        fs::write(&path, text).map_err(|error| io_failure(&path, &error))?;
    }
    Ok(())
}

/// Which of a skill's files under `dir` differ from `files`.
///
/// # Errors
/// The storage failure of a read; a missing file is drift, not a failure.
pub fn drift(dir: &Path, files: &[(&'static str, &str)]) -> Result<Vec<Drift>, StorageError> {
    let mut drifted = Vec::new();
    for &(file, expected) in files {
        let path = dir.join(file);
        match fs::read_to_string(&path) {
            Ok(text) if text == expected => {}
            Ok(_) => drifted.push(Drift {
                file,
                reason: "differs from the rendering",
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => drifted.push(Drift {
                file,
                reason: "missing",
            }),
            Err(error) => return Err(io_failure(&path, &error)),
        }
    }
    Ok(drifted)
}

/// Whether a skill file on disk is one setup wrote and may rewrite.
#[must_use]
pub fn is_managed(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("---\n") else {
        return false;
    };
    let Some(end) = rest.find("\n---\n") else {
        return false;
    };
    rest[..end].lines().any(|line| line.trim() == MANAGED_MARK)
}

fn io_failure(path: &Path, error: &std::io::Error) -> StorageError {
    StorageError::Io {
        path: path.display().to_string(),
        detail: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_file_is_known_by_its_frontmatter_mark() {
        let skill = anb::render();
        for (file, text) in skill.files() {
            assert!(is_managed(text), "{file} lacks the mark");
        }
        assert!(!is_managed(&skill.skill.replace("  managed-by: anb\n", "")));
        assert!(!is_managed("# No frontmatter at all\n"));
    }

    #[test]
    fn setup_installs_the_rendered_skill_and_the_atlas() {
        let [anb, atlas] = installable();
        assert_eq!(anb.name, "anb");
        let rendered = self::anb::render();
        assert_eq!(
            anb.files,
            rendered
                .files()
                .iter()
                .map(|(file, text)| (*file, (*text).to_owned()))
                .collect::<Vec<_>>()
        );
        assert_eq!(atlas.name, "anb-atlas");
        assert_eq!(
            atlas.files,
            self::atlas::files()
                .iter()
                .map(|(file, text)| (*file, (*text).to_owned()))
                .collect::<Vec<_>>()
        );
    }
}
