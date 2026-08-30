//! anb-core — the rules of the game.
//!
//! The Core runs with no filesystem, git, or network access: all data flows
//! through the [`storage::Storage`] seam, fed by the host.

pub mod config;
pub mod date;
pub mod debt;
pub mod encode;
pub mod finding;
pub mod grammar;
mod graph;
mod mention;
pub mod notebook;
pub mod record;
pub mod reply;
pub mod request;
mod resolve;
pub mod status;
pub mod storage;
pub mod tokens;

pub use config::Config;
pub use debt::{DebtSignal, DebtThresholds};
pub use finding::{Finding, FindingCode, Severity};
pub use grammar::RecordFile;
pub use notebook::{Notebook, NotebookError};
pub use record::{Record, RecordType, TaskState};
pub use reply::{
    Archived, Blocker, Cited, CitedProof, Closed, Commented, Counts, Created, Dropped, Edged,
    Edited, Epic, Expunged, FileFinding, Held, ListedRecord, Overview, ReadyTask, Repair,
    Transitioned, TypeSection, View, carriers_of,
};
pub use request::{Draft, Edit, Link, Proof};
pub use resolve::path_stem;
pub use status::{
    ActiveTask, Budget, DebtClass, SECTION_ROWS, Status, StatusRule, counts_phrase, debt_classes,
    epic_line,
};
pub use storage::{MemoryStorage, Storage, StorageError};
pub use tokens::estimate_tokens;
