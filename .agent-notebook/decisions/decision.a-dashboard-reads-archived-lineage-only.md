---
id: decision.a-dashboard-reads-archived-lineage-only
type: decision
state: active
kind: rule
title: A dashboard reads archived lineage only for an epic
by: Maksim Yaromin
tags: perf
created: 2026-08-30
updated: 2026-08-30
---

The archive answers one question for status and overview: where a live record sits inside an epic. Whether the notebook holds an epic at all is settled one step in, because a hub names its own child on its own blocked-by line; the lineage behind those children is read only once a hub is found. A notebook that keeps no epic therefore opens nothing in the archive at session start — 200 live records over a 9800-deep archived lineage cost 403 ms before this rule and 16 ms after, flat in depth. A scope named explicitly (list --for, ready --for) still walks in full: the caller asked for it.
