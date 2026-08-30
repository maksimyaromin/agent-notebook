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
    for id in edges.keys() {
        walk_cycles(edges, id, &mut visits, &mut found);
    }
    found
}

/// The depth-first walk from one id, carrying its own stack of frames: a
/// frame is a node and how many of its edges have been taken, so the frames
/// are the path in progress and a cycle is the tail of them. The edges come
/// from files a hand can edit, where a chain is bounded by nothing — a walk
/// that recursed would abort the process on a long enough one.
fn walk_cycles<'a>(
    edges: &BTreeMap<&'a str, Vec<&'a str>>,
    from: &'a str,
    visits: &mut BTreeMap<&'a str, Visit>,
    found: &mut Vec<Vec<String>>,
) {
    if visits.contains_key(from) {
        return;
    }
    visits.insert(from, Visit::InProgress);
    let mut frames: Vec<(&'a str, usize)> = vec![(from, 0)];
    while let Some(&(at, taken)) = frames.last() {
        let Some(&target) = edges.get(at).map_or(&[][..], Vec::as_slice).get(taken) else {
            visits.insert(at, Visit::Done);
            frames.pop();
            continue;
        };
        let deepest = frames.len() - 1;
        frames[deepest].1 += 1;
        match visits.get(target) {
            Some(Visit::InProgress) => {
                let start = frames
                    .iter()
                    .position(|(id, _)| *id == target)
                    .expect("an in-progress id is on the walk's own stack");
                found.push(
                    frames[start..]
                        .iter()
                        .map(|(id, _)| (*id).to_owned())
                        .collect(),
                );
            }
            Some(Visit::Done) => {}
            None => {
                visits.insert(target, Visit::InProgress);
                frames.push((target, 0));
            }
        }
    }
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
    let mut visited = BTreeSet::from([from.to_owned()]);
    let mut trail = vec![from.to_owned()];
    // One pending-edge iterator per name on the trail, so the walk's depth
    // lives on the heap: the edges are read from files, and a chain long
    // enough to overflow the call stack is one hand edit away.
    let mut pending = vec![edges(from)?.into_iter()];
    while let Some(edges_left) = pending.last_mut() {
        let Some(target) = edges_left.next() else {
            pending.pop();
            trail.pop();
            continue;
        };
        trail.push(target.clone());
        if target == to {
            return Ok(Some(trail));
        }
        if visited.insert(target.clone()) {
            pending.push(edges(&target)?.into_iter());
        } else {
            trail.pop();
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Edges are read from files a hand can edit, so their depth is bounded
    /// by nothing the tool controls. Both walks must answer on a chain far
    /// longer than a call stack holds — a recursive walk aborts the process
    /// instead of refusing the edge.
    const DEEP: usize = 100_000;

    fn name(index: usize) -> String {
        format!("task.link-{index:06}")
    }

    #[test]
    fn a_chain_deeper_than_the_call_stack_is_still_walked_end_to_end() {
        let walked = chain::<()>(&name(0), &name(DEEP - 1), |at| {
            let index: usize = at["task.link-".len()..].parse().unwrap();
            Ok(if index + 1 < DEEP {
                vec![name(index + 1)]
            } else {
                Vec::new()
            })
        })
        .unwrap()
        .expect("the far end is reachable");
        assert_eq!(walked.len(), DEEP);
    }

    #[test]
    fn a_cycle_closing_a_chain_deeper_than_the_call_stack_is_still_named() {
        let names: Vec<String> = (0..DEEP).map(name).collect();
        let mut edges: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for (index, id) in names.iter().enumerate() {
            edges.insert(id, vec![names[(index + 1) % DEEP].as_str()]);
        }
        let found = cycles(&edges);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].len(), DEEP);
    }
}
