//! anb-core — the rules of the game.
//!
//! The Core runs with no filesystem, git, or network access: all data flows
//! through the [`storage::Storage`] seam, fed by the host.

pub mod config;
pub mod debt;
pub mod encode;
pub mod finding;
pub mod grammar;
mod graph;
mod mention;
pub mod notebook;
pub mod record;
pub mod status;
pub mod storage;
pub mod tokens;

pub use config::Config;
pub use debt::{Cited, DebtSignal, DebtThresholds};
pub use finding::{Finding, FindingCode, Severity};
pub use grammar::RecordFile;
pub use notebook::{
    Archived, Closed, Commented, Created, Draft, Dropped, Edged, Edit, Edited, FileFinding, Held,
    Link, ListedRecord, Notebook, NotebookError, Overview, Proof, ReadyTask, Transitioned,
    TypeSection, View,
};
pub use record::{Record, RecordType, TaskState};
pub use status::{ActiveTask, Budget, Counts, Status, StatusRule, counts_phrase};
pub use storage::{MemoryStorage, Storage, StorageError};
pub use tokens::estimate_tokens;
