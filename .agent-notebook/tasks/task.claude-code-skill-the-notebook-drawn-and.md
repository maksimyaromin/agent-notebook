---
id: task.claude-code-skill-the-notebook-drawn-and
type: task
state: open
title: Claude Code skill: the notebook drawn and decided on in one page
by: Maksim Yaromin
from: task.graph-the-data-surface-every-drawing-is
blocked-by: task.improvements
created: 2026-08-30
updated: 2026-09-05
---
- 2026-08-30 Maksim Yaromin: Owner's intent, 2026-08-30: a skill that draws the notebook out of the box using nothing but Claude Code, and takes comments in the page itself — open it, see it, say what to do. No external review harness in the loop.
- 2026-08-30 Maksim Yaromin: The worked reference is in .tmp/atlas/: the page, the snapshot it was built from, and PRINCIPLES.md, which carries what the skill should inherit — one command as the whole input, the encoding, the five lessons that cost the most, and the rule that the picture belongs to whoever asked for it. What it must not inherit: a vendored library, or styling decided in this repository.
- 2026-08-30 Maksim Yaromin: Owner's verdict on the reference page: almost ideal; the missing functionality is his to enumerate.
- 2026-08-31 Maksim Yaromin: Carried from task.graph-lavish-loop and question.does-the-record-open-in-a-side-panel-as, both closed as overtaken (2026-08-31). Beyond drawing, the skill must teach: (1) the intent loop — the page is an intent surface, never a write surface: per-node and general comments collected in the page return to the agent as one batch of targeted instructions, each addressed to a record id, executed only through the CLI. (2) The record opens beside the map, not over it: the reader reviews by pointing, weighing a record against its neighbours, which a covering modal makes impossible — the reasoning that chose a side panel on the deleted emitted page holds for any page this skill draws. (3) Finished work has a picture: closed and archived records drawn distinct, hubs carrying progress counters — the graph data already serves state, archived and epic progress for this.
- 2026-09-05 Maksim Yaromin: Owner ruling 2026-09-05: proceed on PRINCIPLES.md and the three carried requirements without waiting for the enumeration; whatever the owner lists later becomes follow-up tasks. The bar: nothing over-engineered, no hardcode, nothing clumsy or dubious; a clean, efficient tool ready for personalisation, written by the engineer's own rules for skills.
