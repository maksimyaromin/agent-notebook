---
id: task.the-session-opens-with-work-and-rules
type: task
state: review
title: The session opens with work and rules only
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
from: task.records-live-with-their-task
link:
  - follows decision.one-graph-read-through-show-start-recall
  - issue https://github.com/maksimyaromin/agent-notebook/issues/91
created: 2026-09-30
updated: 2026-09-30
---

Implement the bet half of decision.one-graph-read-through-show-start-recall as issue #91 asks: the hook and a bare recall carry the work summary, the focus as a pointer and live rule Decisions from project, personal and global sources; shape and drift Decisions and Notes are reached from the work. The drift open point of #91 is settled in this Task.
- 2026-09-30 Maksim Yaromin/claude-code: Implemented with the search step. A bare recall and the hook carry the work, the focus as {id, title, read} and every live rule and drift Decision of the project, personal and global notebooks; each source counts the other live Decisions and Notes as `other` with `anb list --type decision,note` as `more`. The drift open point is settled as included: a drift is the agreed exception a rule is misread without, and it binds the same way. `--all` keeps its meaning everywhere (lift row and text bounds) and does not widen the selection. Under the budget, bodies give way first (256 characters, then none and the rows print as one table marked bodies-omitted), then work rows, then memory rows. The Core recall lost its focus parameter and the related flag. Field note: this notebook holds 22 live rule Decisions, many of them product shapes; the opening shows 20 of them as titles within 1500 tokens. Docs: status reference, session and tasks guides, records, index, README, llms, quickstart, your-own-notebook.
