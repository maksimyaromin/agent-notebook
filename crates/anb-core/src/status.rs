//! The session dashboard model. Hosts select and encode a bounded view;
//! the Core retains every row so other hosts can apply their own limits.

use crate::reply::{Attribution, Counts, ReadyTask};

/// The default row limit for a dashboard section.
pub const SECTION_ROWS: usize = 5;

/// The token budget a host applies to its dashboard rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Budget {
    Tokens(u32),
    Unbounded,
}

impl Budget {
    pub const DEFAULT_TOKENS: u32 = 1500;

    #[must_use]
    pub fn from_ceiling(ceiling: u32) -> Budget {
        if ceiling == 0 {
            Budget::Unbounded
        } else {
            Budget::Tokens(ceiling)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveTask {
    pub id: String,
    pub title: String,
    pub attribution: Attribution,
    pub log: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewTask {
    pub id: String,
    pub attribution: Attribution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldTask {
    pub id: String,
    pub reason: String,
    pub until: Option<String>,
    pub attribution: Attribution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenQuestion {
    pub id: String,
    pub created: String,
    pub attribution: Attribution,
    pub title: String,
}

/// Complete query results with the requested display budget.
/// List ordering does not select a task for the session.
#[derive(Debug, PartialEq, Eq)]
pub struct Status {
    pub budget: Budget,
    pub quiet: bool,
    pub by: Option<String>,
    pub counts: Counts,
    pub active: Vec<ActiveTask>,
    pub review: Vec<ReviewTask>,
    pub held: Vec<HeldTask>,
    pub ready: Vec<ReadyTask>,
    pub untaken: usize,
    pub questions: Vec<OpenQuestion>,
    pub debt: usize,
}

pub(crate) struct StatusInputs {
    pub by: Option<String>,
    pub counts: Counts,
    pub active: Vec<ActiveTask>,
    pub review: Vec<ReviewTask>,
    pub held: Vec<HeldTask>,
    pub ready: Vec<ReadyTask>,
    pub untaken: usize,
    pub questions: Vec<OpenQuestion>,
    pub debt: usize,
}

pub(crate) fn assemble(inputs: StatusInputs, budget: Budget) -> Status {
    let quiet = inputs.active.is_empty()
        && inputs.review.is_empty()
        && inputs.ready.is_empty()
        && inputs.untaken == 0
        && inputs.questions.is_empty()
        && inputs.debt == 0;
    Status {
        budget,
        quiet,
        by: inputs.by,
        counts: inputs.counts,
        active: inputs.active,
        review: inputs.review,
        held: inputs.held,
        ready: inputs.ready,
        untaken: inputs.untaken,
        questions: inputs.questions,
        debt: inputs.debt,
    }
}

/// Regular nouns only.
#[must_use]
pub fn counted(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}
