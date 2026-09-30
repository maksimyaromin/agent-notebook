//! Compose project knowledge and personal practices without merging their identities.

use anb_core::{Knowledge, Memory, Status};

/// Where a record applies and which notebook a follow-up read must select.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Audience {
    #[default]
    Project,
    Personal,
    Global,
}

impl Audience {
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Personal => "personal",
            Self::Global => "global",
        }
    }

    #[must_use]
    pub fn flag(self) -> &'static str {
        match self {
            Self::Project => "",
            Self::Personal => " --personal",
            Self::Global => " --global",
        }
    }
}

#[derive(Debug)]
pub struct ScopedMemory {
    pub audience: Audience,
    pub memory: Memory,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ScopedInvalid {
    pub audience: Audience,
    pub path: String,
}

/// The Task a session is focused on, named rather than read: `show` reads
/// it whole when the work needs it.
#[derive(Debug, PartialEq, Eq)]
pub struct Focus {
    pub id: String,
    pub title: String,
}

/// A session opening, or a search across every audience. Distinct
/// notebooks may contain the same record id; audience and read command keep
/// those records distinguishable.
#[derive(Debug)]
pub struct Recall {
    /// The work a session opens with; a search answers only what it found.
    pub work: Option<Status>,
    pub focus: Option<Focus>,
    /// The phrase a search looked for; `None` for a session opening.
    pub text: Option<String>,
    pub memories: Vec<ScopedMemory>,
    pub invalid: Vec<ScopedInvalid>,
    /// Per audience, the live Notes and Decisions a session opening leaves
    /// to the work that cites them.
    pub other: Vec<(Audience, usize)>,
    pub all: bool,
    pub budget: anb_core::Budget,
    pub more: String,
}

impl Recall {
    pub fn include(&mut self, audience: Audience, knowledge: Knowledge) {
        self.invalid.extend(
            knowledge
                .invalid
                .into_iter()
                .map(|path| ScopedInvalid { audience, path }),
        );
        self.memories.extend(
            knowledge
                .records
                .into_iter()
                .map(|memory| ScopedMemory { audience, memory }),
        );
        if let Some(other) = knowledge.other {
            self.other.push((audience, other));
        }
    }
}
