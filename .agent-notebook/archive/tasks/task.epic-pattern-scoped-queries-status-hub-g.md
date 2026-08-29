---
id: task.epic-pattern-scoped-queries-status-hub-g
type: task
state: closed
title: Epic pattern: scoped queries + Status hub grouping
by: Maksim Yaromin
via: claude-code
tags: cli
link: sha bfb0b40
blocked-by: task.core-dependency-graph
blocked-by: task.cli-task-cycle
blocked-by: task.core-status-budget
created: 2026-08-29
updated: 2026-08-29
closed: 2026-08-29
---

Design: .tmp/docs/spec-anb-epic-pattern.md. Epic = hub Task: children carry from:<hub>, the hub is blocked-by its children. Deliver the query surface the pattern needs: ready --for <id> and list --for <id> (transitive from-scope), Status/overview grouping by hub (closed/total, next ready, hubs awaiting acceptance), and the resolution recipe for 'continue <epic>' (hub by slug/search -> active task in scope -> top of scoped ready -> start) in the generated skill. No format change: the model already carries both edges.
- 2026-08-29 claude-code: Field evidence from the self-host migration: the founding epic task.anb-v1 had to be assembled from hub-side blocked-by edges alone — from:<hub> on the pre-existing children is impossible without an edit surface. The generated skill must teach that Tasks born inside an idea carry --from its hub at add time; retrofitting is the expensive path.
- 2026-08-29 Maksim Yaromin: Review found the scope rule blind on a hub-side-assembled middle tier: with blocked-by counting only at depth 1, an epic reported an empty queue while its own tasks sat dispatchable — the exact shape of anb-v1 to milestone-cli-complete to its eight children. Settled by making both edges transitive: what a child waits on is work the epic waits on. Also fixed: a closed hub kept asking for its acceptance close, an invalid hub was listed as a live epic against the module's own stated invariant, the epic line had three homes, and the rendered surface shipped untested. Status is no longer slower than ready at 4400 records (0.53s vs 0.58s) after hoisting the per-hub ready_rows. Two open questions filed: hub-side-only epics are scopeable but undiscoverable, and check names no from cycle.
