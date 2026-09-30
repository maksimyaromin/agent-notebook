---
id: task.show-reads-every-edge
type: task
state: review
title: show reads every direct edge, start replies with it, recall --for goes
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
from: task.records-live-with-their-task
link: follows decision.one-graph-read-through-show-start-recall
created: 2026-09-30
updated: 2026-09-30
---

Implement decision.one-graph-read-through-show-start-recall for one record: show lists every direct edge as ids grouped by relation (from, born, task, bound, blocked-by, blocks, links, linked-by, mentions, mentioned-by), unbudgeted and one hop, archived ids unmarked. On 0.9.0 show of a Task omits the records born from it. start, start --next and a resumed focus reply with the same block; today start replies only with the state change. recall --for is removed and its refusal points at show.
- 2026-09-30 Maksim Yaromin/claude-code: Implemented. View carries every direct relation as ids: from, born, task, bound, blocked-by, blocks, links and linked-by (with kinds), mentions, mentioned-by. Incoming relations are read from the whole notebook, the archive included, deduplicated by id with the live file first, and sorted by id; nothing marks archived ids. The groups are plain arrays with no bound (fields and body keep their display bounds and --all). start, start --next and a resumed focus reply with the same block under `record`. recall --for is gone; clap's refusal now tries `anb show <id>` first. Changed promise: view reads the archive (the reading test no longer asserts that show opens nothing there). Docs: records (relations), replies (ok line, bounds), session and tasks guides.
