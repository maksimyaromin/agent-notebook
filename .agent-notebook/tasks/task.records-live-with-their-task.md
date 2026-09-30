---
id: task.records-live-with-their-task
type: task
state: open
title: Records live with their Task, and the graph reads one way
by: Maksim Yaromin
via: claude-code
link:
  - follows decision.a-record-bound-to-a-task-leaves-with-it
  - follows decision.one-graph-read-through-show-start-recall
  - follows decision.search-matches-words-from-their-start
blocked-by:
  - task.bind-a-record-to-its-task
  - task.show-reads-every-edge
  - task.the-session-opens-with-work-and-rules
  - task.search-matches-from-word-start
  - task.readable-id-by-default
  - task.check-finds-unreadable-references
  - task.skill-shows-the-design-workflow
  - task.clean-this-notebook-with-bound-records
created: 2026-09-30
updated: 2026-09-30
---

The hub for decision.a-record-bound-to-a-task-leaves-with-it, decision.one-graph-read-through-show-start-recall and decision.search-matches-words-from-their-start, settled in one design conversation over issues #91 to #94. Close it when every step is closed and a design Task, run end to end in a scratch notebook, leaves its bound records in the archive while its step Tasks still reach them through show.
- 2026-09-30 Maksim Yaromin/claude-code: All eight steps are in review. One Sonnet smoke check over the whole diff found three defects, each reproduced by a test and fixed at its root: archive carried a Task or a rule that had a stray task line (a binding now requires a bindable type, not a rule, naming a task id; Record::task and the residence rule share that predicate); restore of a Task brought back a bound record settled and archived before it (only records that still bind return); unreadable-reference fired on archived records nobody may edit (history is skipped). Also fixed: archive refuses a conflicting Task copy before any bound record moves; the bound move uses record_path; the word-start match lost its punctuation exception; the budget ladder of the opening has a test; the wiki scan is tested through check. Kept: the reply field names bound and unbound, which are the relation's own words. The end-to-end design run in a scratch notebook is the worked session's 'Design work bound to its Task' section, rendered by the binary on every skill check. Gate: scripts/check.sh and pnpm docs:check green; anb check green.
