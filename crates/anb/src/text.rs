//! The plain-text rendering: shape-matched output under the contract —
//! a leading `ok:` line with the transition and its computed consequences,
//! header+rows tables for flat lists, labeled `key: value` for one record,
//! and every refusal as a recovery payload with literal next commands.
//!
//! `status --hook` is the one reply this module does not render itself: a
//! session hook's payload is JSON whichever format the caller asked for,
//! because the harness reading it is not the agent reading the dashboard.

use crate::json;
use crate::recovery::{Recovery, Subject};
use crate::reply::{Reply, lifted, repair_command, shown, slice_command};
use anb_core::date;
use anb_core::encode::ROW_BOUND;
use anb_core::encode::quoted_if_delimited;
use anb_core::{
    EdgeKind, FileFinding, Graph, GraphEdge, GraphNode, ListedRecord, NotebookError, Overview,
    ReadyTask, View, counts_phrase, encode,
};
use std::fmt::Write as _;

#[must_use]
pub fn render(reply: &Reply, today: &str) -> String {
    match reply {
        Reply::Created { command, created } => created_lines(command, created),
        Reply::Moved {
            command,
            transition,
        } => transition_line(command, transition),
        Reply::Routed { transition, to } => {
            let mut out = transition_line("answer", transition);
            let _ = writeln!(out, "routed-to: {to}");
            out
        }
        Reply::Dropped(dropped) => {
            let mut out = transition_line("answer", &dropped.transition);
            dangling_mention_line(&mut out, &dropped.dangling_mentions);
            out
        }
        Reply::Closed(closed) => closed_lines(closed),
        Reply::Held { held, until } => {
            let outcome = match until {
                Some(until) => format!("held until {until}"),
                None => "held".to_owned(),
            };
            format!(
                "ok: hold {} — {outcome}{}\n",
                held.id,
                already_mark(held.already)
            )
        }
        Reply::Unheld(held) => format!(
            "ok: unhold {} — unheld{}\n",
            held.id,
            already_mark(held.already)
        ),
        Reply::Blocked(edge) => format!(
            "ok: block {} — waits on {}{}\n",
            edge.id,
            edge.on,
            already_mark(edge.already)
        ),
        Reply::Unblocked(edge) => format!(
            "ok: unblock {} — edge on {} erased{}\n",
            edge.id,
            edge.on,
            already_mark(edge.already)
        ),
        Reply::Commented(commented) => commented_lines(commented),
        Reply::Ready { rows, scope, all } => ready_table(
            rows,
            shown(rows.len(), *all),
            today,
            &lifted("ready", scope.as_deref()),
        ),
        Reply::Listing { rows, scope, all } => listing_table(
            rows,
            shown(rows.len(), *all),
            "records",
            &lifted("list", scope.as_deref()),
        ),
        Reply::Viewed { view, all } => single_record(view, *all),
        Reply::Checked { findings, all } => findings_table(findings, shown(findings.len(), *all)),
        Reply::Archived(moved) => archive_lines(moved),
        Reply::Expunged(gone) => format!(
            "ok: expunge {} — {} removed\n",
            gone.id,
            gone.paths.join(", ")
        ),
        Reply::Edited(edited) => {
            let mut out = if edited.changed.is_empty() {
                format!("ok: edit {} — unchanged (already)\n", edited.id)
            } else {
                format!("ok: edit {} — {}\n", edited.id, edited.changed.join(", "))
            };
            dangling_mention_line(&mut out, &edited.dangling_mentions);
            out
        }
        Reply::Searched { query, rows, all } => listing_table(
            rows,
            shown(rows.len(), *all),
            "matches",
            &format!("anb search {} --all", shell_quoted(query)),
        ),
        Reply::Overviewed { overview, all } => overview_page(overview, *all),
        Reply::Graphed { graph, full, all } => graph_blocks(graph, *full, *all),
        Reply::Mapped { path, tasks, edges } => {
            format!("ok: graph {path} — {tasks} tasks, {edges} edges\n")
        }
        Reply::Status { status, hook } => {
            if *hook {
                json::hook_payload(status)
            } else {
                status.text.clone()
            }
        }
        Reply::Silence => String::new(),
    }
}

/// The refusal as text: the stable code, the message, the detail lines a
/// repair needs, and the literal next commands.
#[must_use]
pub fn render_error(error: &NotebookError, subject: &Subject) -> String {
    render_recovery(&Recovery::new(error, subject))
}

#[must_use]
pub fn render_recovery(recovery: &Recovery) -> String {
    let mut out = format!("error[{}]: {}\n", recovery.code, recovery.message);
    for detail in &recovery.details {
        let _ = writeln!(out, "  {detail}");
    }
    for suggestion in &recovery.tries {
        let _ = writeln!(out, "try: {suggestion}");
    }
    out
}

fn already_mark(already: bool) -> &'static str {
    if already { " (already)" } else { "" }
}

fn created_lines(command: &str, created: &anb_core::Created) -> String {
    let mut out = format!("ok: {command} {} — {}\n", created.id, created.path);
    if let Some(victim) = &created.superseded {
        let _ = writeln!(out, "superseded: {victim}");
    }
    let named: Vec<String> = created
        .may_conflict
        .iter()
        .map(|cited| format!("{} ({})", cited.id, cited.author()))
        .collect();
    named_line(&mut out, "may-conflict", &named, ROW_BOUND, None);
    dangling_mention_line(&mut out, &created.dangling_mentions);
    out
}

fn commented_lines(commented: &anb_core::Commented) -> String {
    let mut out = format!(
        "ok: comment {} — logged{}\n",
        commented.id,
        already_mark(commented.already)
    );
    dangling_mention_line(&mut out, &commented.dangling_mentions);
    out
}

fn closed_lines(closed: &anb_core::Closed) -> String {
    let mut out = transition_line("close", &closed.transition);
    if let Some(note) = &closed.report_note {
        let _ = writeln!(out, "report: {note}");
    }
    dangling_mention_line(&mut out, &closed.dangling_mentions);
    named_line(&mut out, "unblocked", &closed.unblocked, ROW_BOUND, None);
    named_line(
        &mut out,
        "open-questions",
        &closed.open_questions,
        ROW_BOUND,
        None,
    );
    out
}

fn archive_lines(moved: &anb_core::Archived) -> String {
    let mut out = if moved.already {
        format!("ok: archive {} — archived (already)\n", moved.id)
    } else {
        format!(
            "ok: archive {} — {}\u{2192}{}\n",
            moved.id, moved.from, moved.to
        )
    };
    named_line(&mut out, "carried", &moved.carried, ROW_BOUND, None);
    out
}

/// Body text under its `body: |` header: every line indented, and an empty
/// line left empty rather than indented into whitespace.
fn indented(out: &mut String, text: &str) {
    for line in text.lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            let _ = writeln!(out, "  {line}");
        }
    }
}

/// A reply's inline list: how many there are, then the naming, cut at
/// `bound` — and, where a flag lifts the cut, the command that does.
fn named_line(out: &mut String, label: &str, items: &[String], bound: usize, lift: Option<&str>) {
    if items.is_empty() {
        return;
    }
    let restore = match lift.filter(|_| items.len() > bound) {
        Some(command) => format!(": {command}"),
        None => String::new(),
    };
    let _ = writeln!(
        out,
        "{label}[{}]: {}{restore}",
        items.len(),
        encode::id_list(items, bound)
    );
}

/// The quotation rule's write-time nudge.
fn dangling_mention_line(out: &mut String, ids: &[String]) {
    if ids.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "dangling-mention[{}]: {} — backtick to quote, or create the record",
        ids.len(),
        encode::id_list(ids, ROW_BOUND)
    );
}

/// A move as `from→to`; a replay names the standing state instead of a
/// self-loop.
fn transition_line(command: &str, transition: &anb_core::Transitioned) -> String {
    if transition.already {
        format!(
            "ok: {command} {} — {} (already)\n",
            transition.id, transition.to
        )
    } else {
        format!(
            "ok: {command} {} — {}\u{2192}{}\n",
            transition.id, transition.from, transition.to
        )
    }
}

/// A listing with nothing in it: the one reply that has no table to head,
/// so it states the count the header would have carried.
const EMPTY_LISTING: &str = "count: 0\n";

fn ready_table(rows: &[ReadyTask], shown: usize, today: &str, restore: &str) -> String {
    if rows.is_empty() {
        return EMPTY_LISTING.to_owned();
    }
    let mut out = String::new();
    let today_day = date::day_number(today).unwrap_or(0);
    out.push_str(&ReadyTask::table(rows, shown, today_day));
    truncation_hint(&mut out, rows.len(), shown, restore);
    out
}

fn listing_table(rows: &[ListedRecord], shown: usize, label: &str, restore: &str) -> String {
    if rows.is_empty() {
        return EMPTY_LISTING.to_owned();
    }
    let mut out = record_rows(rows, shown, label);
    truncation_hint(&mut out, rows.len(), shown, restore);
    out
}

/// The one shape of a record-listing block: header, then comma rows.
fn record_rows(rows: &[ListedRecord], shown: usize, label: &str) -> String {
    let mut out = format!("{label}[{}]{{id,state,priority,title}}:\n", rows.len());
    for row in &rows[..shown] {
        let priority = row
            .priority
            .map_or_else(|| "-".to_owned(), |priority| priority.to_string());
        let title = row
            .title
            .as_deref()
            .map_or_else(|| "-".to_owned(), quoted_if_delimited);
        let _ = writeln!(out, "  {},{},{priority},{title}", row.id, row.state);
    }
    out
}

fn findings_table(findings: &[FileFinding], shown: usize) -> String {
    if findings.is_empty() {
        return EMPTY_LISTING.to_owned();
    }
    let mut out = format!(
        "findings[{}]{{file,line,severity,code,repair,message}}:\n",
        findings.len()
    );
    for located in &findings[..shown] {
        let line = located
            .finding
            .line
            .map_or_else(|| "-".to_owned(), |line| line.to_string());
        let repair = located.repair.as_ref().map_or_else(
            || "-".to_owned(),
            |repair| quoted_if_delimited(&repair_command(repair, &located.path)),
        );
        let _ = writeln!(
            out,
            "  {},{line},{},{},{repair},{}",
            quoted_if_delimited(&located.path),
            located.finding.code.severity().as_str(),
            located.finding.code.as_str(),
            quoted_if_delimited(&located.finding.message)
        );
    }
    truncation_hint(&mut out, findings.len(), shown, "anb check --all");
    out
}

/// The graph as the rows a caller reads: the tiles, the lines between them,
/// and — when the caller asked for the records whole — each record's
/// envelope and body under them.
fn graph_blocks(graph: &Graph, full: bool, all: bool) -> String {
    let restore = format!("{} --all", slice_command(&graph.slice, full));
    let mut out = String::new();
    tiles_block(&mut out, graph, all, &restore);
    edges_block(&mut out, &graph.edges(), all, &restore);
    if full {
        record_blocks(&mut out, &graph.nodes, all, &restore);
    }
    out
}

/// Every block a graph reply carries is headed, count and all: a reply
/// that dropped an empty one would leave a reader unable to tell a slice
/// with no lines from a slice the verb never drew.
fn tiles_block(out: &mut String, graph: &Graph, all: bool, restore: &str) {
    let degrees = graph.degrees();
    let shown = shown(graph.nodes.len(), all);
    let _ = writeln!(
        out,
        "nodes[{}]{{id,state,archived,degree,epic,title}}:",
        graph.nodes.len()
    );
    for node in &graph.nodes[..shown] {
        let _ = writeln!(
            out,
            "  {},{},{},{},{},{}",
            node.id,
            node.state,
            if node.archived { "yes" } else { "no" },
            degrees.get(node.id.as_str()).copied().unwrap_or_default(),
            node.epic.as_ref().map_or_else(
                || "-".to_owned(),
                |epic| format!("{}/{}", epic.closed, epic.total)
            ),
            node.title
                .as_deref()
                .map_or_else(|| "-".to_owned(), quoted_if_delimited),
        );
    }
    truncation_hint(out, graph.nodes.len(), shown, restore);
}

fn edges_block(out: &mut String, edges: &[GraphEdge<'_>], all: bool, restore: &str) {
    let shown = shown(edges.len(), all);
    let _ = writeln!(out, "edges[{}]{{from,to,kind}}:", edges.len());
    for edge in &edges[..shown] {
        let word = match edge.kind {
            EdgeKind::BlockedBy => "waits",
            EdgeKind::Origin => "born",
        };
        let _ = writeln!(out, "  {},{},{word}", edge.from, edge.to);
    }
    truncation_hint(out, edges.len(), shown, restore);
}

/// What a tile opens: each record's envelope a line at a time, and each
/// body under its own id. A body carries newlines, which the quoting rule
/// escapes, so one record is still one row.
fn record_blocks(out: &mut String, nodes: &[GraphNode], all: bool, restore: &str) {
    let envelope: Vec<(&str, &str, &str)> = nodes
        .iter()
        .flat_map(|node| {
            node.fields
                .iter()
                .map(|(key, value)| (node.id.as_str(), key.as_str(), value.as_str()))
        })
        .collect();
    let fields_shown = shown(envelope.len(), all);
    let _ = writeln!(out, "fields[{}]{{id,key,value}}:", envelope.len());
    for (id, key, value) in &envelope[..fields_shown] {
        let _ = writeln!(
            out,
            "  {id},{key},{}",
            quoted_if_delimited(&field_value(value, all))
        );
    }
    truncation_hint(out, envelope.len(), fields_shown, restore);

    let written: Vec<&GraphNode> = nodes.iter().filter(|node| !node.body.is_empty()).collect();
    let bodies_shown = shown(written.len(), all);
    let _ = writeln!(out, "bodies[{}]{{id,text}}:", written.len());
    for node in &written[..bodies_shown] {
        let _ = writeln!(
            out,
            "  {},{}",
            node.id,
            quoted_if_delimited(&field_value(&node.body, all))
        );
    }
    truncation_hint(out, written.len(), bodies_shown, restore);
}

fn overview_page(overview: &Overview, all: bool) -> String {
    let mut out = format!("notebook: {}\n", counts_phrase(&overview.live));
    if !overview.epics.is_empty() {
        let shown = shown(overview.epics.len(), all);
        let _ = writeln!(out, "epics[{}]:", overview.epics.len());
        for epic in &overview.epics[..shown] {
            let _ = writeln!(out, "  {}", anb_core::epic_line(epic));
        }
        truncation_hint(&mut out, overview.epics.len(), shown, "anb overview --all");
    }
    for section in &overview.sections {
        if section.rows.is_empty() {
            continue;
        }
        let shown = shown(section.rows.len(), all);
        out.push_str(&record_rows(
            &section.rows,
            shown,
            section.record_type.directory(),
        ));
        truncation_hint(&mut out, section.rows.len(), shown, "anb overview --all");
    }
    let archived = &overview.archived;
    if archived.tasks + archived.decisions + archived.notes + archived.questions > 0 {
        let _ = writeln!(out, "archive: {}", counts_phrase(archived));
    }
    out
}

/// The query as one shell word, single-quoted so every character stays
/// literal and the truncation hint stays executable.
fn shell_quoted(query: &str) -> String {
    let bare = query
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if bare && !query.is_empty() {
        query.to_owned()
    } else {
        format!("'{}'", query.replace('\'', "'\\''"))
    }
}

fn truncation_hint(out: &mut String, total: usize, shown: usize, restore: &str) {
    if total > shown {
        let _ = writeln!(out, "  \u{2026} {} more: {restore}", total - shown);
    }
}

/// An envelope line as `view` shows it: a record's own text, cut like
/// every other reply's unless the caller asked for the record whole.
fn field_value(value: &str, all: bool) -> String {
    if all {
        value.to_owned()
    } else {
        encode::bounded_text(value.to_owned())
    }
}

fn single_record(view: &View, all: bool) -> String {
    let mut out = String::new();
    let fields = shown(view.fields.len(), all);
    for (key, value) in &view.fields[..fields] {
        if value.is_empty() {
            let _ = writeln!(out, "{key}:");
        } else {
            let _ = writeln!(out, "{key}: {}", field_value(value, all));
        }
    }
    truncation_hint(
        &mut out,
        view.fields.len(),
        fields,
        &format!("anb view {} --all", view.id),
    );
    if view.archived {
        out.push_str("archived: true\n");
    }
    if !view.body.is_empty() {
        out.push_str("body: |\n");
        match encode::body_ends(&view.body).filter(|_| !all) {
            Some((head, dropped, tail)) => {
                indented(&mut out, head);
                let _ = writeln!(
                    out,
                    "  \u{2026} {dropped} more lines: anb view {} --all",
                    view.id
                );
                indented(&mut out, tail);
            }
            None => indented(&mut out, &view.body),
        }
    }
    for (label, ids) in [
        ("mentions", &view.mentions),
        ("mentioned-by", &view.mentioned_by),
    ] {
        named_line(
            &mut out,
            label,
            ids,
            shown(ids.len(), all),
            Some(&format!("anb view {} --all", view.id)),
        );
    }
    out
}
