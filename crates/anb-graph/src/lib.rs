//! The map: a notebook's task graph as one page a reader opens from a file.
//!
//! The page is an intent surface. It shows the graph and collects what a
//! reader decides about it; nothing here writes to a notebook, and the
//! page reaches no network — the layout engine, the look and the records
//! all travel inside the file.
//!
//! Every Task is its own element carrying its id, so a reader can point at
//! one and a review harness can anchor a comment to it. Where a tile sits
//! is the layout's answer, computed when the page opens; what a tile *is*
//! is written here, and stands whether or not a script ever runs.

mod svg;
mod view;

use anb_core::{Graph, GraphSlice};

/// The layout engine the page drives: it computes ranks, columns and edge
/// points, and the page draws them.
const DAGRE: &str = include_str!("../assets/vendor/dagre.min.js");
const TEMPLATE: &str = include_str!("../assets/index.html");
const MAP_CSS: &str = include_str!("../assets/map.css");
const MAP_JS: &str = include_str!("../assets/map.js");

/// The map as one page: the graph drawn, the records behind it, and the
/// engine that arranges them, in a single file that opens from disk.
#[must_use]
pub fn render(graph: &Graph) -> String {
    let degrees = graph.degrees();
    // Each value arrives prepared for the place it lands in — a stylesheet,
    // a script, markup, or the page's own text — so the template inserts
    // what it is given. Escaping it a second time for HTML would turn the
    // graph the page draws into text about a graph.
    let values = [
        ("title", text_safe(&title(&graph.slice))),
        ("map_css", inline_safe(MAP_CSS)),
        ("nodes", svg::nodes(graph, &degrees)),
        ("edges", svg::edges(graph)),
        ("data", view::embeddable(&view::of(graph))),
        ("dagre", inline_safe(DAGRE)),
        ("map_js", inline_safe(MAP_JS)),
    ];
    filled(TEMPLATE, &values)
}

/// What the page is a map of. It names the slice, so a file saved from one
/// call says which one it was.
#[must_use]
pub fn title(slice: &GraphSlice) -> String {
    let mut said = Vec::new();
    if slice.ready_only {
        said.push("ready".to_owned());
    }
    if let Some(hub) = &slice.hub {
        said.push(format!("inside {hub}"));
    }
    if let Some(focus) = &slice.focus {
        said.push(format!("around {} within {}", focus.id, focus.depth));
    }
    if said.is_empty() {
        said.push("the whole notebook".to_owned());
    }
    if slice.archive {
        said.push("with the archive".to_owned());
    }
    said.join(" ")
}

/// The template with every `{{ name }}` replaced by the value prepared
/// under it. A replacement is never scanned again, so a record whose own
/// text reads like a hole stays text.
///
/// Template and values are both compiled in, so a name neither side knows
/// is a defect in this crate rather than an answer a caller could correct.
fn filled(template: &str, values: &[(&str, String)]) -> String {
    let mut page = String::with_capacity(template.len());
    let mut rest = template;
    while let Some((before, opened)) = rest.split_once("{{") {
        let (name, after) = opened
            .split_once("}}")
            .expect("the page's own template closes every value it opens");
        let (_, value) = values
            .iter()
            .find(|(known, _)| *known == name.trim())
            .expect("the page's own template asks only for values it prepares");
        page.push_str(before);
        page.push_str(value);
        rest = after;
    }
    page.push_str(rest);
    page
}

/// Text bound for a `<script>` or `<style>` element, where the parser hunts
/// for the closing tag before the language's own rules apply. Breaking the
/// one sequence that ends an element is enough.
fn inline_safe(text: &str) -> String {
    text.replace("</", "<\\/")
}

/// Text bound for the page's own prose or for an attribute, where every
/// character the markup reads has to arrive as its entity.
fn text_safe(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(character),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A closing tag inside a file the page carries whole would end the
    /// element early and spill the rest of it onto the page as markup.
    #[test]
    fn a_closing_tag_inside_an_inlined_file_cannot_end_its_element() {
        assert_eq!(inline_safe("a</script>b"), "a<\\/script>b");
    }

    /// A slice's name is a notebook's own text, printed where the markup
    /// would otherwise act on it: an attribute the page never opened, a
    /// tag it never wrote.
    #[test]
    fn text_the_page_prints_carries_no_character_the_markup_reads() {
        assert_eq!(
            text_safe(r#"<b class="x">Bob's & Co</b>"#),
            "&lt;b class=&quot;x&quot;&gt;Bob&#39;s &amp; Co&lt;/b&gt;"
        );
    }
}
