//! The dependency graph over `blocked-by` edges.
//!
//! Blocked, the ready gate, cycle verdicts, and unblock consequences are
//! computed from the edges on demand; none of them is ever stored. The graph
//! knows a Task only as an id with a closed flag and its edges — records,
//! files, and validity are the notebook's business. Cycle detection walks
//! edges regardless of the closed flag: a closed Task can be reopened, so a
//! latent cycle is a real one.

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

    /// The edge chain from `from` to `to`, both ends included, if the graph
    /// carries one.
    pub fn path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let mut visited = BTreeSet::new();
        let mut trail = vec![from.to_owned()];
        self.extend_trail(from, to, &mut visited, &mut trail)
            .then_some(trail)
    }

    fn extend_trail(
        &self,
        at: &str,
        to: &str,
        visited: &mut BTreeSet<String>,
        trail: &mut Vec<String>,
    ) -> bool {
        if !visited.insert(at.to_owned()) {
            return false;
        }
        let Some(node) = self.nodes.get(at) else {
            return false;
        };
        for target in &node.blocked_by {
            trail.push(target.clone());
            if target == to || self.extend_trail(target, to, visited, trail) {
                return true;
            }
            trail.pop();
        }
        false
    }

    /// Dependency cycles, listed in edge direction, found walking ids in
    /// order. Each back edge met yields one cycle; an edge into an already
    /// finished node is not re-explored, so a cycle overlapping a named one
    /// surfaces on the walk after its neighbor is repaired. A self-edge is
    /// not among them: one record carries that cycle alone and names it
    /// itself.
    pub fn cycles(&self) -> Vec<Vec<String>> {
        let mut found = Vec::new();
        let mut visits = BTreeMap::new();
        let mut stack = Vec::new();
        for id in self.nodes.keys() {
            self.collect_cycles(id, &mut visits, &mut stack, &mut found);
        }
        found
    }

    fn collect_cycles<'a>(
        &'a self,
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
        if let Some(node) = self.nodes.get(at) {
            for target in &node.blocked_by {
                if target == at {
                    continue;
                }
                match visits.get(target.as_str()) {
                    Some(Visit::InProgress) => {
                        let start = stack
                            .iter()
                            .position(|id| *id == target)
                            .expect("an in-progress id is on the stack");
                        found.push(stack[start..].iter().map(|id| (*id).to_owned()).collect());
                    }
                    Some(Visit::Done) => {}
                    None => self.collect_cycles(target, visits, stack, found),
                }
            }
        }
        stack.pop();
        visits.insert(at, Visit::Done);
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
