//! The mutation gate: how a verb gets from an id to bytes it may write.
//!
//! Every write passes through here, and it leaves by one of two doors. A
//! [`LoadedLive`] is a record cleared for mutation — the verb splices and
//! writes it itself. A [`Repairing`] is a record read *over* its error
//! findings, for the verbs that erase them; it keeps those findings and
//! keeps its own bytes, so the only way on is
//! [`Notebook::commit_repair`], which judges what the splice produced.
//!
//! The two doors are the reason this is a module and not a section of the
//! parent: `Repairing`'s fields are private here, so no verb can reach a
//! record's bytes without the judgement that admits the repair.

use super::{Notebook, NotebookError, error, write};
use crate::finding::Finding;
use crate::grammar::RecordFile;
use crate::record::{Record, RecordType, not_utf8_finding};
use crate::resolve::record_path;
use crate::storage::StorageError;

impl Notebook<'_> {
    /// Resolve `id` to its live record, cleared for mutation: the type
    /// matches the command, the record exists live, and it carries no error
    /// finding — from its own bytes or from a dangling reference.
    pub(super) fn load_live(
        &self,
        id: &str,
        expected: &[RecordType],
    ) -> Result<LoadedLive, NotebookError> {
        self.resolve_live(id, commanded_type(id, expected)?)
    }

    /// [`Notebook::load_live`] for a repair: the record is read over its
    /// error findings, and the only way on from there is
    /// [`Notebook::commit_repair`].
    pub(super) fn load_live_repairing(
        &self,
        id: &str,
        expected: &[RecordType],
    ) -> Result<Repairing, NotebookError> {
        self.resolve_live_repairing(id, commanded_type(id, expected)?)
    }

    /// The one gate every write passes: the record is live and carries no
    /// error finding, so no path — a verb or a supersession flip — can
    /// rewrite an invalid record.
    pub(super) fn resolve_live(
        &self,
        id: &str,
        record_type: RecordType,
    ) -> Result<LoadedLive, NotebookError> {
        let (loaded, errors) = self.read_live(id, record_type)?;
        if errors.is_empty() {
            return Ok(loaded);
        }
        Err(NotebookError::InvalidRecord {
            path: loaded.path,
            findings: errors,
        })
    }

    /// [`Notebook::resolve_live`] for a verb that repairs: it is let past
    /// the record's error findings and judged on the bytes it produces
    /// instead. A file with no envelope is not let past, because splicing
    /// one is not defined.
    pub(super) fn resolve_live_repairing(
        &self,
        id: &str,
        record_type: RecordType,
    ) -> Result<Repairing, NotebookError> {
        let (loaded, errors) = self.read_live(id, record_type)?;
        if !errors.is_empty() && !loaded.record.file().has_envelope() {
            return Err(NotebookError::InvalidRecord {
                path: loaded.path,
                findings: errors,
            });
        }
        Ok(Repairing {
            loaded,
            carried: errors,
        })
    }

    /// The live record `id` names, beside the error findings that exclude
    /// it from mutation — read once here, because the two gates above
    /// differ only in what they do with them.
    fn read_live(
        &self,
        id: &str,
        record_type: RecordType,
    ) -> Result<(LoadedLive, Vec<Finding>), NotebookError> {
        let path = record_path(id, record_type, false);
        let text = match self.storage.read(&path) {
            Ok(text) => text,
            Err(StorageError::NotFound { .. }) => {
                return Err(match self.holder_path(id)? {
                    Some(_) => NotebookError::Archived { id: id.to_owned() },
                    None => NotebookError::UnknownId { id: id.to_owned() },
                });
            }
            Err(StorageError::NotUtf8 { .. }) => {
                return Err(NotebookError::InvalidRecord {
                    path,
                    findings: vec![not_utf8_finding()],
                });
            }
            Err(error) => return Err(error.into()),
        };
        let record = Record::parse(&path, &text);
        let errors = self.exclusion_errors(&record)?;
        Ok((LoadedLive { path, record }, errors))
    }

    /// A repairing verb's whole write, and the only way a [`Repairing`]
    /// reaches storage: the splice runs on the record that was read, the
    /// bytes it produced are judged where the record came in carrying
    /// findings, and a splice that moved nothing is written nowhere.
    /// `splice` answers `None` when the record already reads as asked —
    /// the replay — and otherwise whatever the verb needs to report.
    ///
    /// The obligation cannot be forgotten because it is not the caller's:
    /// [`Repairing`] hands out no bytes to write.
    pub(super) fn commit_repair<T>(
        &mut self,
        repairing: Repairing,
        today: &str,
        splice: impl FnOnce(&mut RecordFile) -> Option<T>,
    ) -> Result<Option<T>, NotebookError> {
        let Repairing {
            loaded: LoadedLive { path, record },
            carried,
        } = repairing;
        let mut file = record.into_file();
        let moved = splice(&mut file);
        if moved.is_some() {
            file.set_field("updated", today);
        }
        if !carried.is_empty() {
            self.guard_repaired(&path, &carried, &file)?;
        }
        if moved.is_some() {
            self.storage.write(&path, &file.render())?;
        }
        Ok(moved)
    }

    /// The repair's other half: a verb let past a record's error findings
    /// must leave it better than it found it — fewer of them, and none it
    /// did not already carry — or the call is refused and nothing is
    /// written. The proof is the bytes the splice produced: a line that
    /// carries a finding is not always the line a splice rewrites, and a
    /// splice that reaches the right line may still leave a duplicate of it
    /// standing.
    ///
    /// Progress, not perfection, because a record broken several ways is
    /// repaired one verb at a time, and each verb erases only what it
    /// names. A call that erases nothing is refused all the same, which is
    /// why this runs before the replay answers: a record that came in
    /// invalid must not be told `already` while it still is.
    fn guard_repaired(
        &self,
        path: &str,
        before: &[Finding],
        file: &RecordFile,
    ) -> Result<(), NotebookError> {
        let repaired = Record::parse(path, &file.render());
        let after = self.exclusion_errors(&repaired)?;
        let already_carried = |finding: &Finding| {
            before
                .iter()
                .any(|had| had.code == finding.code && had.message == finding.message)
        };
        if after.len() < before.len() && after.iter().all(already_carried) {
            return Ok(());
        }
        Err(NotebookError::InvalidRecord {
            path: path.to_owned(),
            findings: after,
        })
    }
}

/// A live record cleared for mutation, so it carries no error finding.
pub(super) struct LoadedLive {
    pub(super) path: String,
    pub(super) record: Record,
}

impl LoadedLive {
    /// A record with no state carries an error finding, and one of those
    /// is what [`Notebook::resolve_live`] refuses, so a value of this type
    /// always has one.
    pub(super) fn state_word(&self) -> &str {
        self.record.state().expect("a clean record carries a state")
    }
}

/// A live record read over its error findings, for the verbs that erase
/// them. Its fields are this module's, so a verb outside it can read the
/// record and nothing else: the only way from here to the record's bytes
/// is [`Notebook::commit_repair`], which judges them. That is what makes
/// the repair's second half impossible to forget.
pub(super) struct Repairing {
    loaded: LoadedLive,
    carried: Vec<Finding>,
}

impl Repairing {
    pub(super) fn record(&self) -> &Record {
        &self.loaded.record
    }
}

/// The type `id` names, refused when the command does not accept it.
fn commanded_type(id: &str, expected: &[RecordType]) -> Result<RecordType, NotebookError> {
    let record_type = write::parsed_type(id)?;
    if expected.contains(&record_type) {
        return Ok(record_type);
    }
    Err(NotebookError::WrongType {
        id: id.to_owned(),
        expected: error::type_list(expected),
    })
}
