---
id: task.task-graph-visualization-for-fun
type: task
state: closed
title: Graph epic (hub)
by: Maksim Yaromin
via: claude-code
tags: epic
blocked-by: task.graph-emit-html
blocked-by: task.graph-lavish-loop
created: 2026-08-29
updated: 2026-08-31
closed: 2026-08-31
---

Epic hub for the interactive graph: any slice of the task graph as a map before the owner's eyes, with per-node comments returned to the agent as one batch of targeted instructions — instead of opening record files one by one. Task-centric, id-only tiles colored by state, full record in a modal, blocked-by and origin edges unlabeled, mentions only inside the modal; slices: full map, epic branch, ready lens. An intent surface, never a write surface: anb emits a self-contained HTML artifact and ships no server; the feedback loop belongs to the review harness. Children carry the work; closing this hub is the epic's acceptance. Ordered strictly after the release gate: the product is fully usable for read/write and released first.
- 2026-08-29 Maksim Yaromin: Promoted from idea to the graph epic hub (owner's grill 2026-08-29). Title, tag, and body were updated by hand with the owner's explicit permission: the edit verb does not exist yet — it ships with task.cli-check-archive-edit-search-overview. Children: task.graph-emit-html, task.graph-lavish-loop.
- 2026-08-29 Maksim Yaromin: Finding, to groom when the epic starts: no slice owns the picture of finished work. The read contract must state that the graph reads live + archive; the epic-branch slice shows closed/archived children (grey tiles plus a counter on the hub, e.g. 12/17); consider a dedicated progress lens. Origin: the owner's observation 2026-08-29 — the folder view gives no sense of done vs not-done, archive at hundreds of tasks will not either, and visible progress is a core function of the tool, not cosmetics.
- 2026-08-31 Maksim Yaromin: The epic closes with its second child overtaken: the emitted page shipped and was then superseded by the data surface — anb graph serves records and edges, and the drawing belongs to whoever asked for it — and the intent loop left the external harness. The remainder, the notebook drawn and decided on in one page, lives in task.claude-code-skill-the-notebook-drawn-and, which also carries this hub's ungroomed finding: finished work must have a picture (distinct closed tiles, progress counters on hubs).
