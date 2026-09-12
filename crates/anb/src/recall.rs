//! Compose project knowledge and personal practices without merging their identities.

use anb_core::{Knowledge, Memory, Status, View};

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

/// A session opening. Distinct notebooks may contain the same record id;
/// audience and read command keep those records distinguishable.
#[derive(Debug)]
pub struct Recall {
    pub work: Status,
    pub focus: Option<View>,
    pub memories: Vec<ScopedMemory>,
    pub invalid: Vec<ScopedInvalid>,
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
    }
}
