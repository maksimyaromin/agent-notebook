---
id: task.graph-lavish-loop
type: task
state: closed
title: Graph: the intent loop through the harness
by: Maksim Yaromin
via: claude-code
from: task.task-graph-visualization-for-fun
blocked-by: task.graph-emit-html
created: 2026-08-29
updated: 2026-08-31
closed: 2026-08-31
---

The agent-side flow: open the emitted artifact through the review harness; per-node and general comments return to the agent as one batch of targeted instructions, executed through the CLI. Documented in the generated skill.
- 2026-08-31 Maksim Yaromin: Closed as overtaken, not done: both premises died on 2026-08-30 — anb stopped emitting an artifact (the graph is data; the drawing belongs to whoever asked for it), and the owner took the external review harness out of the loop (the skill draws the page and takes comments in the page itself). The live remainder — the batch-of-instructions loop documented in a skill — is carried onto task.claude-code-skill-the-notebook-drawn-and.
