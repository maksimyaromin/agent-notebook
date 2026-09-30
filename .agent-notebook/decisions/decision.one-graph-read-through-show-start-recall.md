---
id: decision.one-graph-read-through-show-start-recall
type: decision
state: active
kind: shape
title: One graph, read through show, start and recall
by: Maksim Yaromin
via: claude-code
link: issue https://github.com/maksimyaromin/agent-notebook/issues/91
created: 2026-09-30
updated: 2026-09-30
---

An edge stays an edge wherever its records live. The archive changes only what a read offers unasked: a read reaches an archived record through an edge from a live record or by its named id, never by default.

`show <id>` is the complete read of one record: the record and every direct edge, as ids grouped by relation (`from`, `born`, `task`, `bound`, `blocked-by`, `blocks`, `links`, `linked-by`, `mentions`, `mentioned-by`), without a budget and without marking which ids are archived. It goes one hop; the agent walks further with another `show`. `start` replies with the same block, so taking work delivers its context whether or not the agent read the skill.

`recall --for` is removed: its job belongs to `show`, and two commands answering one question is the duplication this ends. The session hook and a bare `recall` stay a budgeted bet on a session whose work is not yet known: the work summary, the focus as a pointer, and live rule Decisions from every source. Other Decisions and Notes are reached from the work. Readable ids make an id-only edge list legible without further reads.
