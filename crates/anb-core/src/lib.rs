//! anb-core — the rules of the game.
//!
//! The Core runs with no filesystem, git, or network access: all data flows
//! through the [`storage::Storage`] seam, fed by the host.

pub mod finding;
pub mod grammar;
pub mod storage;

pub use finding::{Finding, FindingCode, Severity};
pub use grammar::RecordFile;
pub use storage::{MemoryStorage, Storage, StorageError};
