---
id: task.epic-pattern-scoped-queries-status-hub-g
type: task
state: open
title: Epic pattern: scoped queries + Status hub grouping
by: Maksim Yaromin
via: claude-code
tags: cli
blocked-by: task.core-dependency-graph
blocked-by: task.cli-task-cycle
blocked-by: task.core-status-budget
created: 2026-08-29
updated: 2026-08-29
---

Design: .tmp/docs/spec-anb-epic-pattern.md. Epic = hub Task: children carry from:<hub>, the hub is blocked-by its children. Deliver the query surface the pattern needs: ready --for <id> and list --for <id> (transitive from-scope), Status/overview grouping by hub (closed/total, next ready, hubs awaiting acceptance), and the resolution recipe for 'continue <epic>' (hub by slug/search -> active task in scope -> top of scoped ready -> start) in the generated skill. No format change: the model already carries both edges.
- 2026-08-29 claude-code: Field evidence from the self-host migration: the founding epic task.anb-v1 had to be assembled from hub-side blocked-by edges alone — from:<hub> on the pre-existing children is impossible without an edit surface. The generated skill must teach that Tasks born inside an idea carry --from its hub at add time; retrofitting is the expensive path.
