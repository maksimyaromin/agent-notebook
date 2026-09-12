//! anb — the host shell over `anb_core`: the command surface, the
//! filesystem adapter, and the two renderings of every reply. All notebook
//! logic lives in the Core; the shell owns what the Core must not touch —
//! the filesystem, the identity it acts under, and the clock.

pub mod cli;
pub mod fs_storage;
pub mod git;
pub mod hook;
pub mod identity;
pub mod json;
pub mod lock;
pub mod recall;
pub mod reconcile;
pub mod recovery;
pub mod reply;
pub mod scope;
pub mod session;
pub mod setup;
pub mod skill;
pub mod text;
