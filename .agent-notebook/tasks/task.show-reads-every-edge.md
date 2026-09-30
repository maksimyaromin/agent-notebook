---
id: task.show-reads-every-edge
type: task
state: open
title: show reads every direct edge, start replies with it, recall --for goes
by: Maksim Yaromin
via: claude-code
from: task.records-live-with-their-task
link: follows decision.one-graph-read-through-show-start-recall
created: 2026-09-30
updated: 2026-09-30
---

Implement decision.one-graph-read-through-show-start-recall for one record: show lists every direct edge as ids grouped by relation (from, born, task, bound, blocked-by, blocks, links, linked-by, mentions, mentioned-by), unbudgeted and one hop, archived ids unmarked. On 0.9.0 show of a Task omits the records born from it. start, start --next and a resumed focus reply with the same block; today start replies only with the state change. recall --for is removed and its refusal points at show.
