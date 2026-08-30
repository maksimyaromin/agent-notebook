//! anb-core — the rules of the game.
//!
//! The Core runs with no filesystem, git, or network access: all data flows
//! through the [`Storage`] seam, fed by the host.

// Every name a host can reach has exactly one path: the flat re-export
// below, or one of these two namespaces a caller reads as one.
pub mod date;
pub mod encode;

mod config;
mod debt;
mod finding;
mod grammar;
mod graph;
mod mention;
mod notebook;
mod record;
mod reply;
mod request;
mod resolve;
mod status;
mod storage;
mod tokens;

pub use config::Config;
pub use debt::{DebtSignal, DebtThresholds};
pub use finding::{Finding, FindingCode, Severity};
pub use grammar::RecordFile;
pub use notebook::{Notebook, NotebookError};
pub use record::{ARCHIVE_DIR, Record, RecordType, TaskState};
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
