//! The graph as markup: one element per tile, one per line.
//!
//! A tile is an element carrying its id, so a reader points at a Task and a
//! review harness anchors a comment to it. Where a tile sits is the
//! layout's answer and arrives when the page opens; what a tile is stands
//! in the file.

use anb_core::{EdgeKind, Graph, GraphNode};
use std::collections::BTreeMap;
use std::fmt::Write as _;

const SMALLEST: f64 = 7.0;

/// How far from a tile's centre a pointer still means that tile. A tile is
/// drawn at the weight of its dependencies, so a task nothing waits on is a
/// few pixels wide; asking a reader to hit those pixels is asking them to
/// aim rather than to point. The reach is what a pointer is tested against
/// and is never painted, so a leaf takes the same aim as a hub.
const REACH: f64 = 13.0;
const WIDEST: f64 = 20.0;

/// How far a tile reaches from its centre, from how much of the slice meets
/// at it. A web reads by weight, on a curve that keeps a hub visible
/// without letting the difference between two ordinary Tasks disappear.
fn radius(degree: usize) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a degree past 2^53 is a notebook no disk holds"
    )]
    let weight = (degree as f64).sqrt() * 3.5;
    (SMALLEST + weight).min(WIDEST)
}

pub(crate) fn nodes(graph: &Graph, degrees: &BTreeMap<&str, usize>) -> String {
    let mut out = String::new();
    for node in &graph.nodes {
        let degree = degrees.get(node.id.as_str()).copied().unwrap_or_default();
        let radius = radius(degree);
        let reach = radius.max(REACH);
        let _ = write!(
            out,
            "<g class=\"node{}{}\" data-id=\"{}\" data-state=\"{}\" data-degree=\"{degree}\">\
             <circle class=\"reach\" r=\"{reach:.1}\"/>\
             <circle class=\"tile\" r=\"{radius:.1}\"/><text>{}</text></g>",
            if node.epic.is_some() { " hub" } else { "" },
            if node.archived { " archived" } else { "" },
            crate::text_safe(&node.id),
            crate::text_safe(&node.state),
            crate::text_safe(&label(node)),
        );
    }
    out
}

/// The word under a tile: the id, and for a hub how far its epic has come.
/// An id is never shortened — it is what a reader types back into the CLI.
fn label(node: &GraphNode) -> String {
    match &node.epic {
        Some(epic) => format!("{}  {}/{}", node.id, epic.closed, epic.total),
        None => node.id.clone(),
    }
}

pub(crate) fn edges(graph: &Graph) -> String {
    let mut out = String::new();
    for edge in graph.edges() {
        let kind = match edge.kind {
            EdgeKind::BlockedBy => "waits",
            EdgeKind::Origin => "born",
        };
        let _ = write!(
            out,
            "<polyline class=\"edge\" data-kind=\"{kind}\" data-source=\"{}\" data-target=\"{}\"{}/>",
            crate::text_safe(edge.from),
            crate::text_safe(edge.to),
            // A line into what waits carries the arrow; a line out of an
            // Origin carries none, and its stroke says which it is.
            if edge.kind == EdgeKind::BlockedBy {
                " marker-end=\"url(#arrow)\""
            } else {
                ""
            },
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use anb_core::{Epic, GraphSlice};

    fn node(id: &str, blocked_by: &[&str]) -> GraphNode {
        GraphNode {
            id: id.to_owned(),
            state: "open".to_owned(),
            archived: false,
            title: None,
            epic: None,
            blocked_by: blocked_by.iter().map(|id| (*id).to_owned()).collect(),
            origin: None,
            fields: Vec::new(),
            body: String::new(),
            mentions: Vec::new(),
        }
    }

    fn graph(nodes: Vec<GraphNode>) -> Graph {
        Graph {
            slice: GraphSlice::default(),
            nodes,
        }
    }

    /// A tile stands for one Task and says which, because that is what a
    /// reader points at and what a harness anchors a remark to.
    #[test]
    fn every_tile_names_the_task_it_stands_for() {
        let markup = nodes(
            &graph(vec![node("task.one", &[]), node("task.two", &[])]),
            &BTreeMap::new(),
        );
        assert!(markup.contains("data-id=\"task.one\""), "{markup}");
        assert!(markup.contains("data-id=\"task.two\""), "{markup}");
    }

    /// A hub's word says how far its epic has come; every other tile
    /// carries the id alone.
    #[test]
    fn a_hub_carries_its_progress_under_the_tile() {
        let mut hub = node("task.hub", &[]);
        hub.epic = Some(Epic {
            id: "task.hub".to_owned(),
            closed: 2,
            total: 5,
            next: None,
        });
        assert_eq!(label(&hub), "task.hub  2/5");
        assert_eq!(label(&node("task.plain", &[])), "task.plain");
    }

    /// Weight is how much of the slice meets at a tile. A lone Task is
    /// still a tile a reader can hit, and the busiest one in a notebook of
    /// any size still fits beside its neighbours.
    #[test]
    fn a_tile_grows_with_what_meets_at_it_and_never_past_the_map() {
        assert!(radius(0) >= SMALLEST);
        assert!(radius(4) > radius(0));
        assert!(radius(100_000) <= WIDEST);
    }
}
