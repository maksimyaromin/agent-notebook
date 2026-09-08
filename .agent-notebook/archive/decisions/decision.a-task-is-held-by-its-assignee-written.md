---
id: decision.a-task-is-held-by-its-assignee-written
type: decision
state: superseded
kind: shape
title: A Task is taken, never assigned: start records who took it
by: Maksim Yaromin
via: claude-code
tags: identity, tasks
superseded-by: decision.a-task-is-taken-never-assigned-start
created: 2026-09-08
updated: 2026-09-08
---

Several people share one notebook, and the record of who created a Task (by) is not the record of who does it. Nobody assigns a Task; someone takes it. The Task's taken-by is who took it: start writes the caller's identity there when the Task names nobody, and refuses a Task someone else took, a replay included, because answering already would tell a second person the work is theirs. Handing a Task over is a correction made on purpose with edit --taken-by, never a side effect of start, and add carries no way to take a Task for someone else. The identity is ANB_BY, else the git user.name; a host that knows nobody signs nothing and takes nothing. Reading stays the whole project's by default: ready, list, search and every Status section show everyone's records, and --by or --mine narrow a listing to the records one identity created or took. Status alone is viewer-relative: the caller's active Tasks lead, and a line another person took carries their name. Alternatives priced and rejected: an assignee field and add --assignee, because the notebook does not assign work; a start flag that takes over in one move, because taking someone's work should cost a deliberate step; hiding another person's Tasks from ready, because ready is a fact about the notebook that epics and the graph share, and a queue that differs per viewer would make one notebook read as two.
