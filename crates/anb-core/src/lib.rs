//! anb-core — the rules of the game.
//!
//! The Core runs with no filesystem, git, or network access: all data flows
//! through the [`Storage`] seam, fed by the host. A host embeds it by handing
//! a [`Notebook`] an adapter and the day's date:
//!
//! ```
//! use anb_core::{Draft, MemoryStorage, Notebook, NotebookError, RecordType};
//!
//! let today = "2026-09-02";
//! let mut storage = MemoryStorage::new();
//! let mut notebook = Notebook::new(&mut storage);
//! let created = notebook.create(&Draft::new(RecordType::Task, "Parse the fences"), today)?;
//! assert_eq!(created.id, "task.parse-the-fences");
//! notebook.start(&created.id, today)?;
//! assert_eq!(notebook.record(&created.id)?.state(), Some("active"));
//! # Ok::<(), NotebookError>(())
//! ```

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
    Archived, Attribution, Blocker, Cited, CitedProof, Closed, Commented, Counts, Created, Deleted,
    EdgeKind, Edged, Edited, Epic, FileFinding, Filter, Focus, Graph, GraphEdge, GraphNode,
    GraphSlice, Held, ListedRecord, Overview, ReadyTask, Repair, Restored, Transitioned,
    TypeSection, View, carriers_of,
};
pub use request::{Draft, Edit, Link, Proof};
pub use resolve::path_stem;
pub use status::{
    ActiveTask, Budget, DebtClass, HeldTask, SECTION_ROWS, Status, StatusRule, counted,
    counts_phrase, debt_classes, epic_line,
};
pub use storage::{MemoryStorage, Storage, StorageError};
pub use tokens::estimate_tokens;
