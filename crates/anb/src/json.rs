//! Shared reply documents. JSON and TOON encode the same selected values;
//! dashboard budgets are measured against their TOON encoding.

use crate::recovery::{Recovery, Subject};
use crate::reply::{
    Reply, SkillReply, lifted, repair_command, shown, slice_command, team_command,
    team_slice_command,
};
use crate::setup::SetUp;
use anb_core::{
    Budget, Counts, DebtSignal, FileFinding, Filter, Graph, GraphEdge, GraphNode, GraphSlice,
    ListedRecord, NotebookError, OpenQuestion, ReadyTask, RecordType, SECTION_ROWS, Status, View,
    encode,
};
use serde_json::{Map, Value, json};

/// Keep a short excerpt before spending the same budget on more record headers.
const RECALL_EXCERPT_CHARACTERS: usize = 256;

#[must_use]
pub fn render(reply: &Reply) -> String {
    value(reply).to_string()
}

#[must_use]
pub fn value(reply: &Reply) -> Value {
    value_with(reply, |_| {})
}

/// Apply host-specific action paths before measuring the output budget.
#[must_use]
pub fn value_with(reply: &Reply, transform: impl Fn(&mut Value)) -> Value {
    let mut document = match reply {
        Reply::Started(started) => started_value(started),
        Reply::Imported(batch) => batch_value("import", batch),
        Reply::Migrated(batch) => batch_value("migrate", batch),
        Reply::Recalled(recalled) => return recall_value(recalled, &transform),
        Reply::Created { command, created } => created_value(command, created),
        Reply::Moved {
            command,
            transition,
        } => transition_value(command, transition),
        Reply::Closed(closed) => closed_value(closed),
        Reply::Held { held, until } => Value::Object(fields([
            ("ok", json!("hold")),
            ("id", json!(held.id)),
            ("already", json!(held.already)),
            ("until", json!(until)),
        ])),
        Reply::Unheld(held) => json!({
            "ok": "unhold", "id": held.id, "already": held.already,
        }),
        Reply::Blocked(edge) => {
            json!({"ok": "block", "id": edge.id, "on": edge.on, "already": edge.already})
        }
        Reply::Unblocked(edge) => {
            json!({"ok": "unblock", "id": edge.id, "on": edge.on, "already": edge.already})
        }
        Reply::Commented(commented) => Value::Object(fields([
            ("ok", json!("comment")),
            ("id", json!(commented.id)),
            ("already", json!(commented.already)),
            (
                "dangling-mention",
                dangling_mentions(&commented.dangling_mentions),
            ),
        ])),
        Reply::Ready { rows, filter, all } => Value::Object(fields([
            ("by", json!(filter.by)),
            ("team", json!(team_command("ready", filter))),
            ("count", json!(rows.len())),
            ("omitted", json!(rows.len() - shown(rows.len(), *all))),
            (
                "more",
                json!((rows.len() > shown(rows.len(), *all)).then(|| lifted("ready", filter))),
            ),
            (
                "ready",
                json!(
                    rows[..shown(rows.len(), *all)]
                        .iter()
                        .map(ready_row)
                        .collect::<Vec<Value>>()
                ),
            ),
        ])),
        Reply::Listing { rows, filter, all } => Value::Object(fields([
            ("by", json!(filter.by)),
            ("team", json!(team_command("list", filter))),
            ("count", json!(rows.len())),
            ("omitted", json!(rows.len() - shown(rows.len(), *all))),
            (
                "more",
                json!((rows.len() > shown(rows.len(), *all)).then(|| lifted("list", filter))),
            ),
            (
                "records",
                json!(
                    rows[..shown(rows.len(), *all)]
                        .iter()
                        .map(listed_row)
                        .collect::<Vec<Value>>()
                ),
            ),
        ])),
        Reply::Viewed { view, all } => view_value(view, *all),
        Reply::Status { status, .. } => return status_value(status, &transform),
        Reply::Checked { findings, all } => checked_value(findings, *all),
        Reply::Debt { signals, all } => json!({
            "count": signals.len(),
            "omitted": signals.len() - shown(signals.len(), *all),
            "more": "anb debt --all",
            "debt": signals[..shown(signals.len(), *all)]
                .iter()
                .map(debt_value)
                .collect::<Vec<Value>>(),
        }),
        Reply::Archived(moved) => archived_value(moved),
        Reply::Restored(moved) => restored_value(moved),
        Reply::Deleted(gone) => json!({"ok": "delete", "id": gone.id, "paths": gone.paths}),
        Reply::Skill(skill) => skill_value(skill),
        Reply::SetUp(done) => setup_value(done),
        Reply::Edited(edited) => edited_value(edited),
        Reply::Graphed { graph, full, all } => graph_value(graph, *full, *all),
    };
    transform(&mut document);
    document
}

fn batch_value(command: &str, batch: &anb_core::FileBatch) -> Value {
    Value::Object(fields([
        ("ok", json!(command)),
        ("check", json!(batch.check)),
        ("paths", json!(batch.paths)),
        ("unchanged", json!(batch.unchanged)),
        ("backup", json!(batch.backup)),
        ("notice", json!((command == "migrate").then_some("YAML quotes delimit text. If an older file used surrounding quotes as literal characters, verify that value before applying migration; backups preserve the original bytes."))),
    ]))
}

fn started_value(started: &crate::session::Started) -> Value {
    let mut object = transition_map("start", &started.transition);
    if let Some(session) = &started.session {
        object.insert("session".to_owned(), json!(session));
        object.insert("joined".to_owned(), json!(started.joined));
    }
    Value::Object(object)
}

/// Encode a reply document using the standard TOON codec.
///
/// # Panics
/// If a value exceeds the codec's nesting limit. Reply documents have a bounded shape.
#[must_use]
pub fn toon(value: &Value) -> String {
    reddb_io_toon::encode(&reddb_io_toon::Value::from_json_value(value.clone()))
        .expect("reply documents contain only shallow JSON values")
}

fn recall_value(recalled: &crate::recall::Recall, transform: &impl Fn(&mut Value)) -> Value {
    let visible = shown(recalled.memories.len(), recalled.all);
    let ordered = fair_memories(&recalled.memories);
    let sources = [
        crate::recall::Audience::Project,
        crate::recall::Audience::Personal,
        crate::recall::Audience::Global,
    ]
    .into_iter()
    .filter_map(|audience| {
        let count = ordered
            .iter()
            .filter(|item| item.audience == audience)
            .count();
        let shown = ordered
            .iter()
            .take(visible)
            .filter(|item| item.audience == audience)
            .count();
        (count > 0)
            .then(|| json!({"scope": audience.word(), "count": count, "omitted": count - shown}))
    })
    .collect::<Vec<_>>();
    let mut document = json!({
        "work": status_document(&recalled.work, recalled.all),
        "focus": recalled.focus.as_ref().map(|view| view_value(view, recalled.all)),
        "count": recalled.memories.len(),
        "sources": sources,
        "memories": ordered.iter().take(visible).map(|item| {
            let memory = &item.memory;
            json!({
                "scope": item.audience.word(),
                "id": memory.id,
                "title": memory.title,
                "kind": memory.kind,
                "by": memory.by,
                "body": body_value(&memory.body, recalled.all),
                "links": memory.links,
                "related": memory.related,
                "read": format!("anb show {}{} --all", memory.id, item.audience.flag()),
            })
        }).collect::<Vec<_>>(),
        "omitted": recalled.memories.len() - visible,
        "invalid": section(&recalled.invalid, shown(recalled.invalid.len(), recalled.all), |invalid| json!({
            "scope": invalid.audience.word(),
            "path": invalid.path,
            "repair": format!("anb check{}", invalid.audience.flag()),
        })),
        "more": recalled.more,
    });
    transform(&mut document);
    fit(document, recalled.budget, |document| {
        trim_work(document, "/work")
            || trim_memory_bodies(document, RECALL_EXCERPT_CHARACTERS)
            || trim_memories(document, true)
            || trim_body(document.pointer_mut("/focus/body"))
            || trim_section(document, "/focus/fields", 0)
            || trim_section(document, "/focus/mentions", 0)
            || trim_section(document, "/focus/mentioned-by", 0)
            || trim_section(document, "/focus/linked-by", 0)
            || trim_memory_bodies(document, 0)
            || trim_memories(document, false)
            || trim_section(document, "/invalid", 0)
    })
}

/// Round-robin selection prevents a large source from hiding another audience.
/// Each source keeps its own relevance order; selection does not grant authority.
fn fair_memories(memories: &[crate::recall::ScopedMemory]) -> Vec<&crate::recall::ScopedMemory> {
    use crate::recall::Audience;
    let mut sources = [Audience::Project, Audience::Personal, Audience::Global].map(|audience| {
        memories
            .iter()
            .filter(move |item| item.audience == audience)
    });
    let mut ordered = Vec::with_capacity(memories.len());
    while ordered.len() < memories.len() {
        for source in &mut sources {
            if let Some(memory) = source.next() {
                ordered.push(memory);
            }
        }
    }
    ordered
}

#[must_use]
pub fn render_error(error: &NotebookError, subject: &Subject) -> String {
    render_recovery(&Recovery::new(error, subject))
}

#[must_use]
pub fn render_recovery(recovery: &Recovery) -> String {
    recovery_value(recovery).to_string()
}

#[must_use]
pub fn recovery_value(recovery: &Recovery) -> Value {
    let mut object = fields([
        ("error", json!(recovery.code)),
        ("message", json!(recovery.message)),
        (
            "findings",
            if recovery.details.is_empty() {
                Value::Null
            } else {
                json!(&recovery.details[..shown(recovery.details.len(), false)])
            },
        ),
        ("try", json!(recovery.tries)),
    ]);
    if !recovery.details.is_empty() {
        object.insert("count".to_owned(), json!(recovery.details.len()));
        object.insert(
            "omitted".to_owned(),
            json!(recovery.details.len() - shown(recovery.details.len(), false)),
        );
    }
    object.extend(
        recovery
            .context
            .iter()
            .map(|(key, value)| ((*key).to_owned(), json!(value))),
    );
    Value::Object(object)
}

fn created_value(command: &str, created: &anb_core::Created) -> Value {
    Value::Object(fields([
        ("ok", json!(command)),
        ("id", json!(created.id)),
        ("path", json!(created.path)),
        ("superseded", json!(created.superseded)),
        (
            "dangling-mention",
            dangling_mentions(&created.dangling_mentions),
        ),
    ]))
}

fn skill_value(skill: &SkillReply) -> Value {
    match skill {
        SkillReply::Printed(text) => json!({"ok": "skill", "skill": text}),
        SkillReply::Written { dir, files } => json!({"ok": "skill", "dir": dir, "files": files}),
        SkillReply::Checked { dir, drift } => json!({
            "ok": "skill",
            "dir": dir,
            "drift": drift
                .iter()
                .map(|drift| json!({"file": drift.file, "reason": drift.reason}))
                .collect::<Vec<Value>>(),
        }),
    }
}

fn setup_value(done: &SetUp) -> Value {
    let files = done
        .files
        .iter()
        .map(|wired| json!({"path": wired.path, "outcome": wired.outcome.word()}))
        .collect::<Vec<Value>>();
    Value::Object(fields([
        ("ok", json!("setup")),
        ("removed", json!(done.removed)),
        ("files", json!(files)),
        ("skipped", json!(done.skipped)),
        ("notice", json!(done.notice)),
    ]))
}

fn closed_value(closed: &anb_core::Closed) -> Value {
    let mut object = transition_map("close", &closed.transition);
    object.extend(fields([
        ("resolved-by", json!(closed.resolved_by)),
        (
            "dangling-mention",
            dangling_mentions(&closed.dangling_mentions),
        ),
        (
            "unblocked",
            bounded_section(&closed.unblocked, |id| json!(id)),
        ),
        (
            "open-questions",
            bounded_section(&closed.open_questions, |id| json!(id)),
        ),
    ]));
    Value::Object(object)
}

fn transition_value(command: &str, transition: &anb_core::Transitioned) -> Value {
    Value::Object(transition_map(command, transition))
}

fn transition_map(command: &str, transition: &anb_core::Transitioned) -> Map<String, Value> {
    fields([
        ("ok", json!(command)),
        ("id", json!(transition.id)),
        ("from", json!(transition.from)),
        ("to", json!(transition.to)),
        ("already", json!(transition.already)),
    ])
}

/// Missing body references are informational; they do not refuse the write.
fn dangling_mentions(ids: &[String]) -> Value {
    consequence(ids, |id| json!(id))
}

fn ready_row(row: &ReadyTask) -> Value {
    let mut object = attributed(
        [
            ("id", json!(row.id)),
            ("priority", json!(row.priority)),
            ("created", json!(row.created)),
        ],
        &row.attribution,
    );
    object.extend(fields([(
        "title",
        json!(encode::bounded_text(row.title.clone())),
    )]));
    Value::Object(object)
}

fn listed_row(row: &ListedRecord) -> Value {
    let mut object = attributed(
        [
            ("id", json!(row.id)),
            ("state", json!(row.state)),
            ("priority", json!(row.priority)),
        ],
        &row.attribution,
    );
    object.extend(fields([(
        "title",
        json!(
            row.title
                .as_ref()
                .map(|title| encode::bounded_text(title.clone()))
        ),
    )]));
    Value::Object(object)
}

fn checked_value(findings: &[FileFinding], all: bool) -> Value {
    json!({
        "count": findings.len(),
        "omitted": findings.len() - shown(findings.len(), all),
        "more": "anb check --all",
        "findings": findings[..shown(findings.len(), all)]
            .iter()
            .map(finding_value)
            .collect::<Vec<Value>>(),
    })
}

fn archived_value(moved: &anb_core::Archived) -> Value {
    Value::Object(fields([
        ("ok", json!("archive")),
        ("id", json!(moved.id)),
        ("from", json!(moved.from)),
        ("to", json!(moved.to)),
        ("already", json!(moved.already)),
    ]))
}

fn restored_value(moved: &anb_core::Restored) -> Value {
    Value::Object(fields([
        ("ok", json!("restore")),
        ("id", json!(moved.id)),
        ("from", json!(moved.from)),
        ("to", json!(moved.to)),
        ("already", json!(moved.already)),
    ]))
}

fn edited_value(edited: &anb_core::Edited) -> Value {
    Value::Object(fields([
        ("ok", json!("edit")),
        ("id", json!(edited.id)),
        ("changed", json!(edited.changed)),
        ("already", json!(edited.changed.is_empty())),
        (
            "dangling-mention",
            dangling_mentions(&edited.dangling_mentions),
        ),
    ]))
}

fn finding_value(located: &FileFinding) -> Value {
    Value::Object(fields([
        ("file", json!(located.path)),
        ("line", json!(located.finding.line)),
        ("severity", json!(located.finding.code.severity().as_str())),
        ("code", json!(located.finding.code.as_str())),
        (
            "repair",
            json!(
                located
                    .repair
                    .as_ref()
                    .map(|repair| repair_command(repair, &located.path))
            ),
        ),
        ("message", json!(located.finding.message)),
    ]))
}

fn epic_value(epic: &anb_core::Epic) -> Value {
    Value::Object(fields([
        ("id", json!(epic.id)),
        ("closed", json!(epic.closed)),
        ("total", json!(epic.total)),
        ("next", json!(epic.next)),
    ]))
}

/// Version of the graph document consumed by atlas and other graph readers.
const GRAPH_CONTRACT: u8 = 4;

/// Graph structure is complete. Only optional record content has display bounds.
fn graph_value(graph: &Graph, full: bool, all: bool) -> Value {
    let degrees = graph.degrees();
    let edges = graph.edges();
    json!({
        "v": GRAPH_CONTRACT,
        "team": team_slice_command(&graph.slice, full),
        "more": format!("{} --all", slice_command(&graph.slice, full)),
        "slice": slice_value(&graph.slice),
        "nodes": whole_section(&graph.nodes, |node| {
            graph_node_value(node, degrees.get(node.id.as_str()).copied().unwrap_or_default(), full, all)
        }),
        "edges": whole_section(&edges, graph_edge_value),
    })
}

/// The applied filters and focus distinguish a slice from the whole notebook.
fn slice_value(slice: &GraphSlice) -> Value {
    let mut object = filter_fields(&slice.filter);
    object.extend(fields([
        ("focus", json!(slice.focus.as_ref().map(|focus| &focus.id))),
        (
            "depth",
            json!(slice.focus.as_ref().map(|focus| focus.depth)),
        ),
    ]));
    Value::Object(object)
}

fn filter_fields(filter: &Filter) -> Map<String, Value> {
    let words = |words: &[String]| {
        if words.is_empty() {
            Value::Null
        } else {
            json!(words)
        }
    };
    fields([
        (
            "type",
            match filter.types.as_slice() {
                [] => Value::Null,
                types => json!(
                    types
                        .iter()
                        .copied()
                        .map(RecordType::word)
                        .collect::<Vec<&str>>()
                ),
            },
        ),
        ("kind", words(&filter.kinds)),
        ("tag", words(&filter.tags)),
        ("for", json!(filter.hub)),
        ("by", json!(filter.by)),
        ("untaken", json!(filter.untaken)),
        ("to", json!(filter.to)),
        ("match", json!(filter.text)),
        ("archive", json!(filter.archive)),
    ])
}

fn graph_node_value(node: &GraphNode, degree: usize, full: bool, all: bool) -> Value {
    let mut object = fields([
        ("id", json!(node.id)),
        ("type", json!(node.kind.map(RecordType::word))),
        ("state", json!(node.state)),
        ("ready", json!(node.ready)),
        ("archived", json!(node.archived)),
        ("degree", json!(degree)),
        ("priority", json!(node.priority)),
        ("created", json!(node.created)),
        ("epic", node.epic.as_ref().map_or(Value::Null, epic_value)),
        ("title", json!(node.title)),
    ]);
    if full {
        object.extend(fields([
            (
                "fields",
                section(
                    &node.fields,
                    shown(node.fields.len(), all),
                    |(key, value)| json!([key, field_value(value, all)]),
                ),
            ),
            ("body", body_value(&node.body, all)),
        ]));
    }
    Value::Object(object)
}

fn graph_edge_value(edge: &GraphEdge<'_>) -> Value {
    json!({"from": edge.from, "to": edge.to, "kind": edge.kind.word()})
}

fn counts_value(counts: &Counts) -> Value {
    json!({
        "tasks": counts.tasks,
        "decisions": counts.decisions,
        "notes": counts.notes,
        "questions": counts.questions,
    })
}

/// Omitted body text stays separate from the preserved head and tail.
fn body_value(body: &str, all: bool) -> Value {
    let lines = body.lines().count();
    let characters = body.chars().count();
    let (head, tail) = if all {
        (body.to_owned(), String::new())
    } else if let Some((head, _, tail)) = encode::body_ends(body) {
        (head.chars().take(1000).collect(), suffix(tail, 1000))
    } else if characters > 2000 {
        (body.chars().take(1000).collect(), suffix(body, 1000))
    } else {
        (body.to_owned(), String::new())
    };
    let omitted = characters - head.chars().count() - tail.chars().count();
    Value::Object(fields([
        ("lines", json!(lines)),
        ("characters", json!(characters)),
        ("head", json!(head)),
        (
            "tail",
            if tail.is_empty() {
                Value::Null
            } else {
                json!(tail)
            },
        ),
        ("omitted", json!(omitted)),
    ]))
}

fn suffix(text: &str, characters: usize) -> String {
    let start = text.chars().count().saturating_sub(characters);
    text.chars().skip(start).collect()
}

/// An envelope line as data: cut like every other reply's text unless the
/// caller asked for the record whole.
fn field_value(value: &str, all: bool) -> String {
    if all {
        value.to_owned()
    } else {
        encode::bounded_text(value.to_owned())
    }
}

fn view_value(view: &View, all: bool) -> Value {
    json!({
        "id": view.id,
        "path": view.path,
        "more": format!("anb show {} --all", view.id),
        "archived": view.archived,
        "fields": section(&view.fields, shown(view.fields.len(), all), |(key, value)| {
            json!([key, field_value(value, all)])
        }),
        "body": body_value(&view.body, all),
        "mentions": section(&view.mentions, shown(view.mentions.len(), all), |id| json!(id)),
        "mentioned-by": section(&view.mentioned_by, shown(view.mentioned_by.len(), all), |id| json!(id)),
        "linked-by": section(&view.linked_by, shown(view.linked_by.len(), all), |(kind, id)| {
            json!({"id": id, "kind": kind})
        }),
    })
}

fn status_value(status: &Status, transform: &impl Fn(&mut Value)) -> Value {
    let mut document = status_document(status, status.budget == Budget::Unbounded);
    transform(&mut document);
    fit(document, status.budget, |document| trim_work(document, ""))
}

fn status_document(status: &Status, all: bool) -> Value {
    let rows = |total: usize| {
        if all { total } else { total.min(SECTION_ROWS) }
    };
    Value::Object(fields([
        ("quiet", json!(status.quiet)),
        ("by", json!(status.by)),
        (
            "more",
            json!(status.by.as_ref().map_or_else(
                || "anb status --budget 0".to_owned(),
                |by| format!("anb status --by {} --budget 0", encode::shell_word(by))
            )),
        ),
        (
            "team",
            json!(status.by.as_ref().map(|_| "anb status --team")),
        ),
        ("counts", counts_value(&status.counts)),
        (
            "active",
            section(&status.active, rows(status.active.len()), active_task_value),
        ),
        (
            "review",
            section(&status.review, rows(status.review.len()), |task| {
                Value::Object(attributed([("id", json!(task.id))], &task.attribution))
            }),
        ),
        (
            "held",
            section(&status.held, rows(status.held.len()), held_task_value),
        ),
        (
            "ready",
            section(&status.ready, rows(status.ready.len()), ready_row),
        ),
        (
            "untaken",
            json!({"count": status.untaken, "more": "anb ready --untaken"}),
        ),
        (
            "questions",
            section(
                &status.questions,
                rows(status.questions.len()),
                question_row,
            ),
        ),
        ("debt", json!({"count": status.debt, "more": "anb debt"})),
    ]))
}

fn fit(mut document: Value, budget: Budget, mut trim: impl FnMut(&mut Value) -> bool) -> Value {
    let limit = match budget {
        Budget::Tokens(limit) => Some(limit),
        Budget::Unbounded => None,
    };
    document["budget"] = json!({"limit": limit, "spent": 0});
    loop {
        let mut previous = None;
        loop {
            let spent = anb_core::estimate_tokens(&format!("{}\n", toon(&document)));
            if previous == Some(spent) {
                break;
            }
            document["budget"]["spent"] = json!(spent);
            previous = Some(spent);
        }
        let spent = document["budget"]["spent"].as_u64().unwrap_or_default();
        if limit.is_none_or(|limit| spent <= u64::from(limit)) || !trim(&mut document) {
            return document;
        }
    }
}

fn trim_work(document: &mut Value, prefix: &str) -> bool {
    for section in ["ready", "questions", "held", "review"] {
        if trim_section(document, &format!("{prefix}/{section}"), 0) {
            return true;
        }
    }
    if let Some(rows) = document
        .pointer_mut(&format!("{prefix}/active/rows"))
        .and_then(Value::as_array_mut)
    {
        for row in rows.iter_mut().rev() {
            if row
                .as_object_mut()
                .is_some_and(|row| row.remove("log").is_some())
            {
                row["log-omitted"] = json!(true);
                return true;
            }
        }
    }
    trim_section(document, &format!("{prefix}/active"), 1)
}

fn trim_section(document: &mut Value, path: &str, minimum: usize) -> bool {
    let Some(section) = document.pointer_mut(path) else {
        return false;
    };
    let Some(rows) = section["rows"].as_array_mut() else {
        return false;
    };
    if rows.len() <= minimum {
        return false;
    }
    rows.pop();
    let visible = rows.len();
    section["omitted"] = json!(section["count"].as_u64().unwrap_or_default() - visible as u64);
    true
}

fn trim_body(body: Option<&mut Value>) -> bool {
    trim_body_above(body, 0)
}

fn trim_body_above(body: Option<&mut Value>, minimum: usize) -> bool {
    let Some(body) = body else {
        return false;
    };
    let head = body["head"].as_str().unwrap_or_default();
    let tail = body["tail"].as_str().unwrap_or_default();
    let visible = head.chars().count() + tail.chars().count();
    if visible <= minimum {
        return false;
    }
    let remaining = (visible / 2).max(minimum);
    let head_count = if tail.is_empty() {
        remaining
    } else {
        remaining.div_ceil(2)
    };
    let head: String = head.chars().take(head_count).collect();
    let tail = suffix(tail, remaining - head.chars().count());
    body["omitted"] = json!(
        body["characters"].as_u64().unwrap_or_default()
            - head.chars().count() as u64
            - tail.chars().count() as u64
    );
    body["head"] = json!(head);
    if body.get("tail").is_some() {
        body["tail"] = json!(tail);
    }
    true
}

fn trim_memory_bodies(document: &mut Value, minimum: usize) -> bool {
    let Some(memories) = document["memories"].as_array_mut() else {
        return false;
    };
    memories
        .iter_mut()
        .rev()
        .any(|memory| trim_body_above(memory.get_mut("body"), minimum))
}

fn trim_memories(document: &mut Value, keep_sources: bool) -> bool {
    let Some(memories) = document["memories"].as_array_mut() else {
        return false;
    };
    let removable = memories.iter().rposition(|candidate| {
        !keep_sources
            || memories
                .iter()
                .filter(|row| row["scope"] == candidate["scope"])
                .count()
                > 1
    });
    let Some(index) = removable else {
        return false;
    };
    let removed = memories.remove(index);
    let visible = memories.len();
    document["omitted"] = json!(document["count"].as_u64().unwrap_or_default() - visible as u64);
    if let Some(sources) = document["sources"].as_array_mut() {
        for source in sources {
            if source["scope"] == removed["scope"] {
                source["omitted"] = json!(source["omitted"].as_u64().unwrap_or_default() + 1);
            }
        }
    }
    true
}

/// The fields of a row about a record, with who wrote it, who holds it
/// and whom it is addressed to beside them when the record names them.
fn attributed<'a>(
    entries: impl IntoIterator<Item = (&'a str, Value)>,
    attribution: &anb_core::Attribution,
) -> Map<String, Value> {
    let mut object = fields(entries);
    object.extend(fields([
        ("by", json!(attribution.by)),
        ("taken-by", json!(attribution.taken_by)),
        ("to", json!(attribution.to)),
    ]));
    object
}

fn question_row(row: &OpenQuestion) -> Value {
    Value::Object(attributed(
        [
            ("id", json!(row.id)),
            ("created", json!(row.created)),
            ("title", json!(encode::bounded_text(row.title.clone()))),
        ],
        &row.attribution,
    ))
}

/// Preserve field order while omitting absent optional values.
fn fields<'a>(entries: impl IntoIterator<Item = (&'a str, Value)>) -> Map<String, Value> {
    entries
        .into_iter()
        .filter(|(_, value)| !value.is_null())
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

/// One section of a grouped reply as data: how many there are, and the
/// first `shown` of them. A reply an agent reads must not grow with the
/// notebook, whichever format it asks for.
fn section<T>(rows: &[T], shown: usize, row: impl Fn(&T) -> Value) -> Value {
    json!({
        "count": rows.len(),
        "omitted": rows.len() - shown,
        "rows": rows[..shown].iter().map(row).collect::<Vec<Value>>(),
    })
}

/// [`section`] carrying every row it counted, for a block whose meaning
/// depends on holding all of them.
fn whole_section<T>(rows: &[T], row: impl Fn(&T) -> Value) -> Value {
    section(rows, rows.len(), row)
}

/// [`section`] at the default bound, for a consequence a command names in
/// passing rather than a listing a caller asked for: no flag lifts it.
fn bounded_section<T>(rows: &[T], row: impl Fn(&T) -> Value) -> Value {
    section(rows, shown(rows.len(), false), row)
}

/// [`bounded_section`] for a consequence that is news only when it
/// happened: nothing to report lands no key.
fn consequence<T>(rows: &[T], row: impl Fn(&T) -> Value) -> Value {
    if rows.is_empty() {
        Value::Null
    } else {
        bounded_section(rows, row)
    }
}

fn held_task_value(task: &anb_core::HeldTask) -> Value {
    Value::Object(attributed(
        [
            ("id", json!(task.id)),
            ("reason", json!(encode::bounded_text(task.reason.clone()))),
            ("until", json!(task.until)),
        ],
        &task.attribution,
    ))
}

fn active_task_value(task: &anb_core::ActiveTask) -> Value {
    let mut object = attributed(
        [
            ("id", json!(task.id)),
            ("title", json!(encode::bounded_text(task.title.clone()))),
        ],
        &task.attribution,
    );
    object.extend(fields([(
        "log",
        json!(
            task.log
                .as_ref()
                .map(|log| encode::bounded_text(log.clone()))
        ),
    )]));
    Value::Object(object)
}

/// The signal's own values ride beside the printed line, so a program
/// reads them as data and never parses the sentence.
fn debt_value(signal: &DebtSignal) -> Value {
    let mut object = fields([("code", json!(signal.code()))]);
    object.extend(debt_fields(signal));
    object.insert("line".to_owned(), json!(signal.line()));
    Value::Object(object)
}

fn debt_fields(signal: &DebtSignal) -> Map<String, Value> {
    match signal {
        DebtSignal::TaskStale { id, days }
        | DebtSignal::QuestionAge { id, days }
        | DebtSignal::HoldStale { id, days }
        | DebtSignal::ReviewStale { id, days } => {
            fields([("id", json!(id)), ("days", json!(days))])
        }
        DebtSignal::OriginClosed { id, origin } => {
            fields([("id", json!(id)), ("origin", json!(origin))])
        }
        DebtSignal::ReviewDue { id, date } => fields([("id", json!(id)), ("date", json!(date))]),
        DebtSignal::DanglingMention { id, target } => {
            fields([("id", json!(id)), ("target", json!(target))])
        }
        DebtSignal::Invalid { path, errors } => {
            fields([("file", json!(path)), ("errors", json!(errors))])
        }
        DebtSignal::LostProof { id, proof } => fields([("id", json!(id)), ("proof", json!(proof))]),
    }
}
