---
id: task.bind-a-record-to-its-task
type: task
state: open
title: A record bound to a Task leaves and returns with it
by: Maksim Yaromin
via: claude-code
from: task.records-live-with-their-task
link:
  - follows decision.a-record-bound-to-a-task-leaves-with-it
  - issue https://github.com/maksimyaromin/agent-notebook/issues/94
created: 2026-09-30
updated: 2026-09-30
---

Implement decision.a-record-bound-to-a-task-leaves-with-it: the task field on Decision, Note and Question; add --task, edit --task, edit --clear task; refusal for a rule Decision and for a target that is not a live Task; archive and restore of a Task carry its bound records; an open bound Question refuses the archive; restore of one bound record clears its task; check accepts an active bound record in the archive and reports a broken binding; the close reply lists born but unbound records with the bind command. Records, replies and lifecycle reference pages move with it.
