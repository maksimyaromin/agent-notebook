//! The values a verb is asked with: what a caller hands the Core.
//!
//! The pair of [`reply`](crate::reply) — one vocabulary in, one out, both
//! below the Notebook, so nothing that judges or renders a request has to
//! reach up into the verbs that take it.

use crate::record::RecordType;

/// Which records a read answers with. Every narrowing is a predicate over
/// the same notebook, so asking for two asks for the intersection; the
/// default is every live record.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    /// Only these types; empty is every type.
    pub types: Vec<RecordType>,
    /// Only these kinds; empty is every record, the ones carrying no kind
    /// included.
    pub kinds: Vec<String>,
    /// Only records carrying every one of these tags.
    pub tags: Vec<String>,
    /// One record's scope: the hub, what it waits on, what was born
    /// inside it, and what links it.
    pub hub: Option<String>,
    /// One identity's work: the Tasks it holds, the other records it
    /// wrote, and the records waiting on it.
    pub by: Option<String>,
    /// Only the Tasks nobody holds: the pool anyone may take.
    pub untaken: bool,
    /// Only the records addressed to this identity.
    pub to: Option<String>,
    /// Only records whose id, title, tags, people or body hold this text,
    /// matched without regard to case.
    pub text: Option<String>,
    /// The archive too. Most of what a long-lived notebook holds is
    /// finished, and reading all of it buries the work in flight, so the
    /// archive stays closed until it is asked for.
    pub archive: bool,
}

/// Which records a graph is asked for: the filter every listing takes, and
/// one record with the graph around it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphSlice {
    pub filter: Filter,
    pub focus: Option<Focus>,
}

/// A record and how far around it the graph reaches, counted in edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Focus {
    pub id: String,
    pub depth: usize,
}

/// A record to be created; `id: None` mints one from the title, `by: None`
/// signs it with the notebook's identity, `taken_by` hands a Task over as
/// it is written, `to` addresses a Task or a Question to someone.
#[derive(Debug)]
pub struct Draft {
    pub record_type: RecordType,
    pub title: String,
    pub id: Option<String>,
    pub kind: Option<String>,
    pub by: Option<String>,
    pub via: Option<String>,
    pub taken_by: Option<String>,
    pub to: Option<String>,
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
            taken_by: None,
            to: None,
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

impl Link {
    /// The envelope line the link is written as: the kind, one space, the
    /// target with its ends trimmed.
    pub(crate) fn line(&self) -> String {
        format!("{} {}", self.kind, self.target.trim())
    }

    /// Whether a standing `link` line names this link, however it spaces
    /// its two halves: a line written by hand is matched as the grammar
    /// reads it, never byte for byte.
    pub(crate) fn matches(&self, line: &str) -> bool {
        crate::grammar::split_link(line) == Some((self.kind.as_str(), self.target.trim()))
    }
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
    pub add_links: Vec<Link>,
    pub remove_links: Vec<Link>,
    pub from: Option<String>,
    pub priority: Option<u32>,
    pub review_by: Option<String>,
    pub taken_by: Option<String>,
    pub to: Option<String>,
    /// The optional fields to erase, by their envelope key — `review-by`,
    /// not `review_by`. A record that never carried the field is already
    /// as asked, so the clear is a no-op.
    pub clear: Vec<String>,
}

/// The optional fields `edit` erases: the ones it can also write, minus
/// those with an eraser of their own — a body through an empty `--body`, a
/// tag through `--untag`. A field the record's type does not allow is
/// erasable all the same; erasing it is the repair.
pub(crate) const CLEARABLE: [&str; 5] = [FROM, PRIORITY, REVIEW_BY, TAKEN_BY, TO];

pub(crate) const FROM: &str = "from";
pub(crate) const PRIORITY: &str = "priority";
pub(crate) const REVIEW_BY: &str = "review-by";
pub(crate) const TAKEN_BY: &str = "taken-by";
pub(crate) const TO: &str = "to";

impl Edit {
    pub(crate) fn changes_nothing(&self) -> bool {
        self.title.is_none()
            && self.body.is_none()
            && self.add_tags.is_empty()
            && self.remove_tags.is_empty()
            && self.add_links.is_empty()
            && self.remove_links.is_empty()
            && self.from.is_none()
            && self.priority.is_none()
            && self.review_by.is_none()
            && self.taken_by.is_none()
            && self.to.is_none()
            && self.clear.is_empty()
    }
}
