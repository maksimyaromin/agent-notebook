//! Import and migration share a read-only planning boundary before any write.

use super::{Notebook, NotebookError};
use crate::resolve::{archive_of, canonical_paths, is_record_file, resolvable_id};
use crate::{FileBatch, MemoryStorage, RecordFile, RecordType, Storage};
use std::collections::BTreeMap;

const JOURNALS: &str = ".migrations.tmp";
type Files = BTreeMap<String, String>;

impl Notebook<'_> {
    /// Import typed record directories and their archive, preserving source history.
    ///
    /// # Errors
    /// Invalid source records, colliding ids, an invalid prospective notebook,
    /// or a storage failure. Identical records are skipped on a retry.
    pub fn import(
        &mut self,
        source: &dyn Storage,
        check: bool,
    ) -> Result<FileBatch, NotebookError> {
        let incoming = snapshot(source)?;
        if incoming.is_empty() {
            return Err(invalid(
                "source contains no Markdown records in typed directories",
            ));
        }
        let mut prospective = snapshot(self.storage)?;
        let mut writes = Files::new();
        let mut unchanged = 0;
        for (path, original) in incoming {
            let canonical = RecordFile::parse(&original).normalize();
            if let Some(id) = resolvable_id(&path) {
                for holder in canonical_paths(id) {
                    if let Some(existing) = prospective.get(&holder)
                        && (holder != path || RecordFile::parse(existing).normalize() != canonical)
                    {
                        return Err(NotebookError::DuplicateId {
                            id: id.to_owned(),
                            holder,
                        });
                    }
                }
            }
            if prospective.contains_key(&path) {
                unchanged += 1;
            } else {
                prospective.insert(path.clone(), canonical.clone());
                writes.insert(path, canonical);
            }
        }
        validate(&prospective)?;
        if !check {
            for (path, content) in &writes {
                self.storage.write(path, content)?;
            }
        }
        Ok(FileBatch {
            paths: writes.into_keys().collect(),
            unchanged,
            check,
            backup: None,
        })
    }

    /// Normalize record envelopes without changing dates, ids or bodies.
    ///
    /// # Errors
    /// Invalid records, a conflicting interrupted migration, or a storage failure.
    /// Original bytes are journaled before the first record is changed.
    pub fn migrate(&mut self, check: bool) -> Result<FileBatch, NotebookError> {
        let current = snapshot(self.storage)?;
        let pending = pending_journal(self.storage)?;
        let (journal, originals) = match pending {
            Some(path) => {
                let originals: Files =
                    serde_json::from_str(&self.storage.read(&path)?).map_err(|error| {
                        invalid(format!("cannot read migration journal {path}: {error}"))
                    })?;
                if originals.is_empty() {
                    return Err(invalid(format!(
                        "migration journal {path} contains no originals; inspect it before retrying"
                    )));
                }
                for (record_path, original) in &originals {
                    if resolvable_id(record_path).is_none() {
                        return Err(invalid(format!(
                            "migration journal {path} contains an invalid record path"
                        )));
                    }
                    let canonical = RecordFile::parse(original).normalize();
                    if !current
                        .get(record_path)
                        .is_some_and(|text| text == original || text == &canonical)
                    {
                        return Err(invalid(format!(
                            "{record_path} changed since migration {path}; recover its original or finish that migration before retrying"
                        )));
                    }
                }
                (Some(path), originals)
            }
            None => (
                None,
                current
                    .iter()
                    .filter(|(_, original)| RecordFile::parse(original).normalize() != **original)
                    .map(|(path, original)| (path.clone(), original.clone()))
                    .collect(),
            ),
        };
        let writes: Files = originals
            .iter()
            .map(|(path, original)| (path.clone(), RecordFile::parse(original).normalize()))
            .collect();
        let mut prospective = current.clone();
        prospective.extend(writes.clone());
        validate(&prospective)?;
        let paths: Vec<String> = writes.keys().cloned().collect();
        let unchanged = current.len().saturating_sub(writes.len());
        if check || writes.is_empty() {
            return Ok(FileBatch {
                paths,
                unchanged,
                check,
                backup: journal,
            });
        }
        let journal = if let Some(path) = journal {
            path
        } else {
            let path = next_journal(self.storage)?;
            let contents = serde_json::to_string_pretty(&originals)
                .map_err(|error| invalid(format!("cannot encode migration originals: {error}")))?;
            self.storage.write(&path, &contents)?;
            path
        };
        for (path, content) in &writes {
            if current.get(path) != Some(content) {
                self.storage.write(path, content)?;
            }
        }
        self.storage
            .write(&format!("{journal}.done"), "complete\n")?;
        Ok(FileBatch {
            paths,
            unchanged,
            check,
            backup: Some(journal),
        })
    }
}

fn snapshot(storage: &dyn Storage) -> Result<Files, NotebookError> {
    let mut files = Files::new();
    for record_type in RecordType::ALL {
        for directory in [
            record_type.directory().to_owned(),
            archive_of(record_type.directory()),
        ] {
            for path in storage.list(&directory)? {
                if is_record_file(&path) {
                    files.insert(path.clone(), storage.read(&path)?);
                }
            }
        }
    }
    Ok(files)
}

fn validate(files: &Files) -> Result<(), NotebookError> {
    let mut prospective = MemoryStorage::from_files(files.clone());
    let findings = Notebook::new(&mut prospective).check()?;
    if let Some(first) = findings.iter().find(|located| located.finding.is_error()) {
        return Err(NotebookError::InvalidRecord {
            path: first.path.clone(),
            findings: findings
                .iter()
                .filter(|located| located.path == first.path && located.finding.is_error())
                .map(|located| located.finding.clone())
                .collect(),
        });
    }
    Ok(())
}

fn pending_journal(storage: &dyn Storage) -> Result<Option<String>, NotebookError> {
    let mut pending = None;
    for path in storage.list(JOURNALS)? {
        if path
            .rsplit_once('.')
            .is_some_and(|(_, extension)| extension == "json")
            && !storage.exists(&format!("{path}.done"))?
        {
            if pending.is_some() {
                return Err(invalid(
                    "more than one unfinished migration journal; recover the originals before migrating",
                ));
            }
            pending = Some(path);
        }
    }
    Ok(pending)
}

fn next_journal(storage: &dyn Storage) -> Result<String, NotebookError> {
    for sequence in 1..=u64::MAX {
        let path = format!("{JOURNALS}/{sequence:08}.json");
        if !storage.exists(&path)? && !storage.exists(&format!("{path}.done"))? {
            return Ok(path);
        }
    }
    Err(invalid("migration journal names are exhausted"))
}

fn invalid(reason: impl Into<String>) -> NotebookError {
    NotebookError::InvalidArgument {
        reason: reason.into(),
    }
}
