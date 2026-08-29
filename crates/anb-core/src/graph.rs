//! The dependency graph over `blocked-by` edges, and the walks any edge
//! the notebook draws can be put through: the cycles in an edge map, and
//! the chain from one name to another.
//!
//! Blocked, the ready gate, and unblock consequences are computed from the
//! edges on demand; none of them is ever stored. The graph knows a Task
//! only as an id with a closed flag and its edges — records, files, and
//! validity are the notebook's business.

use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct TaskNode {
    pub closed: bool,
    pub blocked_by: Vec<String>,
}

pub(crate) struct TaskGraph {
    nodes: BTreeMap<String, TaskNode>,
}

enum Visit {
    InProgress,
    Done,
}

impl TaskGraph {
    pub fn new(nodes: BTreeMap<String, TaskNode>) -> Self {
        TaskGraph { nodes }
    }

    /// A Task is blocked iff an edge points at a non-closed Task. A target
    /// outside the graph blocks nothing: whether it
    /// dangles is validity, judged elsewhere.
    pub fn is_blocked(&self, id: &str) -> bool {
        let Some(node) = self.nodes.get(id) else {
            return false;
        };
        node.blocked_by.iter().any(|target| {
            self.nodes
                .get(target)
                .is_some_and(|blocker| !blocker.closed)
        })
    }

    /// The non-closed Tasks that closing `id` releases: they wait on it and
    /// on nothing else still live. `id` itself counts as closed, so the
    /// answer is the same before and after its file says so.
    pub fn unblocked_by(&self, id: &str) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|(_, node)| {
                !node.closed
                    && node.blocked_by.iter().any(|target| target == id)
                    && !node.blocked_by.iter().any(|target| {
                        target != id
                            && self
                                .nodes
                                .get(target)
                                .is_some_and(|blocker| !blocker.closed)
                    })
            })
            .map(|(dependent, _)| dependent.clone())
            .collect()
    }
}

/// The cycles in an edge map, listed in edge direction, found walking ids
/// in order. Each back edge met yields one cycle, a name pointing at
/// itself included; an edge into an already finished node is not
/// re-explored, so a cycle overlapping a named one surfaces on the walk
/// after its neighbor is repaired.
pub(crate) fn cycles(edges: &BTreeMap<&str, Vec<&str>>) -> Vec<Vec<String>> {
    let mut found = Vec::new();
    let mut visits = BTreeMap::new();
    let mut stack = Vec::new();
    for id in edges.keys() {
        collect_cycles(edges, id, &mut visits, &mut stack, &mut found);
    }
    found
}

fn collect_cycles<'a>(
    edges: &BTreeMap<&'a str, Vec<&'a str>>,
    at: &'a str,
    visits: &mut BTreeMap<&'a str, Visit>,
    stack: &mut Vec<&'a str>,
    found: &mut Vec<Vec<String>>,
) {
    if visits.contains_key(at) {
        return;
    }
    visits.insert(at, Visit::InProgress);
    stack.push(at);
    for target in edges.get(at).map(Vec::as_slice).unwrap_or_default() {
        match visits.get(target) {
            Some(Visit::InProgress) => {
                let start = stack
                    .iter()
                    .position(|id| id == target)
                    .expect("an in-progress id is on the stack");
                found.push(stack[start..].iter().map(|id| (*id).to_owned()).collect());
            }
            Some(Visit::Done) => {}
            None => collect_cycles(edges, target, visits, stack, found),
        }
    }
    stack.pop();
    visits.insert(at, Visit::Done);
}

/// The edge chain from `from` to `to`, both ends included, walked over a
/// graph that answers one record at a time. `edges` names what one record
/// points at, whichever edge is being walked; a name it does not know
/// points at nothing.
///
/// Read on demand, the walk costs the closure of `from`, where building a
/// whole graph to answer the same question costs every record the notebook
/// has ever held.
pub(crate) fn chain<E>(
    from: &str,
    to: &str,
    mut edges: impl FnMut(&str) -> Result<Vec<String>, E>,
) -> Result<Option<Vec<String>>, E> {
    let mut visited = BTreeSet::new();
    let mut trail = vec![from.to_owned()];
    Ok(extend_chain(from, to, &mut edges, &mut visited, &mut trail)?.then_some(trail))
}

fn extend_chain<E>(
    at: &str,
    to: &str,
    edges: &mut impl FnMut(&str) -> Result<Vec<String>, E>,
    visited: &mut BTreeSet<String>,
    trail: &mut Vec<String>,
) -> Result<bool, E> {
    if !visited.insert(at.to_owned()) {
        return Ok(false);
    }
    for target in edges(at)? {
        trail.push(target.clone());
        if target == to || extend_chain(&target, to, edges, visited, trail)? {
            return Ok(true);
        }
        trail.pop();
    }
    Ok(false)
}
