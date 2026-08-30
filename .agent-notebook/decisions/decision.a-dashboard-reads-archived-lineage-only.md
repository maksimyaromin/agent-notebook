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

The archive answers one question for status and overview: where a live record sits inside an epic. Whether the notebook holds an epic at all is settled one step in, because a hub names its own child on its own blocked-by line — so establishing that there is no epic costs that one hop and nothing more, and the lineage behind those children is read only once a hub is there to place a record in. A notebook that keeps no epic therefore pays a bounded price at session start instead of one that grows with its history: 200 live records over a 9800-deep archived lineage cost 403 ms before this rule and 16 ms after, flat in depth. A scope named explicitly (list --for, ready --for) still walks in full: the caller asked for it.
