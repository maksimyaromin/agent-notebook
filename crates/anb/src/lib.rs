//! anb — the host shell over `anb_core`: the command surface, the
//! filesystem adapter, and the two renderings of every reply. All notebook
//! logic lives in the Core; the shell owns what the Core must not touch —
//! the filesystem, git identity, and the clock.

pub mod cli;
pub mod fs_storage;
pub mod json;
pub mod reply;
pub mod text;
