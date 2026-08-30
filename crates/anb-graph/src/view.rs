//! The records the page reads: one JSON document holding what a tile opens
//! when a reader picks it.
//!
//! What the drawing already carries — where a tile sits, what it waits on,
//! which state it is in — is in the markup, so this holds only what a
//! reader sees after a click and what the page filters by.

use anb_core::Graph;
use serde_json::{Value, json};

pub(crate) fn of(graph: &Graph) -> Value {
    json!({
        "nodes": graph.nodes.iter().map(|node| json!({
            "id": node.id,
            "title": node.title,
            "fields": node.fields.iter().map(|(key, value)| json!([key, value])).collect::<Vec<_>>(),
            "body": node.body,
            "mentions": node.mentions,
        })).collect::<Vec<_>>(),
    })
}

/// The document as the page carries it. A record's text is a notebook's
/// own, so the one sequence that could end the element early is written as
/// the escape JSON already reads it as.
pub(crate) fn embeddable(document: &Value) -> String {
    document.to_string().replace('<', "\\u003c")
}

#[cfg(test)]
mod tests {
    use super::*;
    use anb_core::{GraphNode, GraphSlice};

    fn node(id: &str) -> GraphNode {
        GraphNode {
            id: id.to_owned(),
            state: "open".to_owned(),
            archived: false,
            title: Some("A demo record".to_owned()),
            epic: None,
            blocked_by: Vec::new(),
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

    /// A record's body is a notebook's own text, and the page carries it
    /// inside an element the parser ends at the first closing tag.
    #[test]
    fn a_body_that_closes_an_element_is_carried_as_an_escape() {
        let mut hostile = node("task.demo");
        hostile.body = "</script><b>".to_owned();
        let carried = embeddable(&of(&graph(vec![hostile])));

        assert!(
            !carried.contains('<'),
            "nothing in the document can open a tag: {carried}"
        );
        assert!(carried.contains("\\u003c/script>"), "{carried}");
    }

    /// A tile stands for a record, and the record is what a reader opens on
    /// it, so the page carries the whole of it rather than a summary.
    #[test]
    fn a_tile_carries_the_record_a_reader_opens_on_it() {
        let mut task = node("task.demo");
        task.body = "The plan.".to_owned();
        task.mentions = vec!["decision.rule".to_owned()];
        task.fields = vec![("state".to_owned(), "open".to_owned())];

        let carried = of(&graph(vec![task]));

        assert_eq!(carried["nodes"][0]["body"], "The plan.");
        assert_eq!(carried["nodes"][0]["mentions"][0], "decision.rule");
        assert_eq!(carried["nodes"][0]["fields"][0], json!(["state", "open"]));
        assert_eq!(carried["nodes"][0]["title"], "A demo record");
    }
}
