//! Who the host acts as: the accountable identity a new record's `by`, a
//! log entry's signature and a taken Task's `taken-by` carry, and the one
//! `--mine` means.
//!
//! `ANB_BY` names it outright, for a checkout with no git identity or one
//! whose git name is not the name the notebook should know; otherwise git's
//! `user.name` answers. Absence is legal: the notebook then signs nothing
//! and takes nothing, and `--mine` is refused for want of a name.

use crate::git;

pub const IDENTITY_ENV: &str = "ANB_BY";

/// The identity, or `None` when neither the environment nor git names one.
#[must_use]
pub fn name() -> Option<String> {
    std::env::var(IDENTITY_ENV)
        .ok()
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .or_else(git::user_name)
}
