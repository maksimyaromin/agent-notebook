//! The values a verb is asked with: what a caller hands the Core.
//!
//! The pair of [`reply`](crate::reply) — one vocabulary in, one out, both
//! below the Notebook, so nothing that judges or renders a request has to
//! reach up into the verbs that take it.

use crate::record::RecordType;

/// A record to be created; `id: None` mints one from the title.
#[derive(Debug)]
pub struct Draft {
    pub record_type: RecordType,
    pub title: String,
    pub id: Option<String>,
    pub kind: Option<String>,
    pub by: Option<String>,
    pub via: Option<String>,
    pub from: Option<String>,
    pub tags: Vec<String>,
    pub links: Vec<Link>,
    pub supersedes: Option<String>,
    pub priority: Option<u32>,
    pub body: String,
}

impl Draft {
    #[must_use]
    pub fn new(record_type: RecordType, title: &str) -> Self {
        Draft {
            record_type,
            title: title.to_owned(),
            id: None,
            kind: None,
            by: None,
            via: None,
            from: None,
            tags: Vec::new(),
            links: Vec::new(),
            supersedes: None,
            priority: None,
            body: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct Link {
    pub kind: String,
    pub target: String,
}

/// The auditable evidence a close carries. `Waived` is the explicit
/// override: the caller states there is no proof rather than omitting it.
#[derive(Debug)]
pub enum Proof {
    Pr(String),
    Sha(String),
    Report(String),
    /// A Note holding the report, so the proof travels with the notebook.
    Note(String),
    Waived,
}

impl Proof {
    pub(crate) fn link_value(&self) -> Option<String> {
        match self {
            Proof::Pr(target) => Some(format!("pr {target}")),
            Proof::Sha(target) => Some(format!("sha {target}")),
            Proof::Report(target) => Some(format!("report {target}")),
            Proof::Note(target) => Some(format!("note {target}")),
            Proof::Waived => None,
        }
    }
}

/// The deliberate corrections `edit` applies to a live record's own fields.
/// State, id, and the envelope dates stay the commands' territory.
#[derive(Debug, Default)]
pub struct Edit {
    pub title: Option<String>,
    pub body: Option<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub from: Option<String>,
    pub priority: Option<u32>,
    pub review_by: Option<String>,
    /// The optional fields to erase, by their envelope key — `review-by`,
    /// not `review_by`. A record that never carried the field is already
    /// as asked, so the clear is a no-op.
    pub clear: Vec<String>,
}

/// The optional fields `edit` erases: the ones it can also write, minus
/// those with an eraser of their own — a body through an empty `--body`, a
/// tag through `--untag`. A field the record's type does not allow is
/// erasable all the same; erasing it is the repair.
pub const CLEARABLE: [&str; 3] = ["from", PRIORITY, "review-by"];

pub(crate) const PRIORITY: &str = "priority";

impl Edit {
    pub(crate) fn changes_nothing(&self) -> bool {
        self.title.is_none()
            && self.body.is_none()
            && self.add_tags.is_empty()
            && self.remove_tags.is_empty()
            && self.from.is_none()
            && self.priority.is_none()
            && self.review_by.is_none()
            && self.clear.is_empty()
    }
}
