---
id: decision.a-record-bound-to-a-task-leaves-with-it
type: decision
state: active
kind: rule
title: A record bound to a Task leaves and returns with it
by: Maksim Yaromin
via: claude-code
link: issue https://github.com/maksimyaromin/agent-notebook/issues/94
supersedes: decision.knowledge-lifetime
created: 2026-09-30
updated: 2026-09-30
---

A Decision, Note or Question may name the Task it serves in its `task` field. The binding is explicit only: `add --task`, `edit --task` and `edit --clear task` write it; neither a session focus nor an origin implies it, and a record without the field keeps its own lifetime. The field names a live Task; any other target is refused, and check reports one written by hand or by a merge. A rule Decision cannot be bound: a rule is standing knowledge.

Archiving the Task moves its bound records with it in their current state, so an active Decision in the archive is valid while its Task is archived. An open bound Question refuses the archive and names the moves: close it or clear its task. Restoring the Task returns its bound records. Restoring one bound record alone clears its task, which is how knowledge outlives the Task that produced it. The close reply of a Task lists the live records born from it that are not bound, each with the command that binds it.

The lifetime is declared when the record is written, never inferred when its origin is archived; inferring it from edges is why the earlier cascade was withdrawn. Design work is the main use: the Decisions, specs and Questions a design Task produces leave with it, while the step Tasks it planned cite them by full id and reach them in the archive through those edges.
- 2026-09-30 Maksim Yaromin/claude-code: Implementation note: 'names a live Task' is implemented as 'names a Task in the working set, closed or not'. The close reply lists born but unbound records with a bind command that runs after the Task has closed, so a closed Task must accept a binding; an archived Task is refused until restored.
