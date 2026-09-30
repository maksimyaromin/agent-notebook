---
id: task.skill-shows-the-design-workflow
type: task
state: closed
title: The skill shows the design workflow and the three reads
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
from: task.records-live-with-their-task
link:
  - follows decision.a-record-bound-to-a-task-leaves-with-it
  - follows decision.one-graph-read-through-show-start-recall
  - follows decision.search-matches-words-from-their-start
blocked-by:
  - task.bind-a-record-to-its-task
  - task.show-reads-every-edge
  - task.search-matches-from-word-start
created: 2026-09-30
updated: 2026-09-30
closed: 2026-09-30
---

The skill gains a table of which question takes show, start or recall, and a worked design session: a design Task, its Decisions, specs and Questions added with --task, step Tasks born from it citing those Decisions by full id, close, archive, and a step Task reaching an archived Decision through show. Search is lexical: turn a meaning into domain words and run several searches.
- 2026-09-30 Maksim Yaromin/claude-code: Implemented. The skill's recall section has a table of which question takes recall, show, start or a search, each with its reason, and search guidance (turn a meaning into the project's words, search again). The intent section says how to connect: --from for work born from work, full ids or a follows link for governing Decisions, and why [[slug]] connects nothing. A new Design work section walks the bound workflow in six steps, and the memory table has a row for design-only records. The worked session gains 'Design work bound to its Task': a design Task, a bound shape Decision and Question, an unbound Note, a step Task citing the Decision, the close reply's unbound row and its bind command, archive carrying the bound records, and show reaching the archived Decision from the step and back. That section is also the end-to-end run in a scratch notebook the hub asks for. The humanizer pass ran over the new prose.
- 2026-09-30 Maksim Yaromin/claude-code: Owner review asked whether the skill had turned into a list of this epic's fixes. It had, in part: the recall section lost 'Follow the user's subject' and gained a mechanics table and a one-off concurrency example, the [[slug]] warning restated what check already enforces, and a six-step design procedure sat at the weight of the whole method. Rewritten: the recall section keeps its instruction and a two-row table of the reads that answer a question, the connect guidance is one sentence on citing full ids, and binding is one paragraph that says when to bind and when not, pointing at the worked session for the end-to-end design run. The method grew 1970 -> 2243 words (it was 2524). Not done: the writing-skills baseline test with and without the change.
- 2026-09-30 Maksim Yaromin/claude-code: writing-skills test run on the rewritten skill: 10 fresh Sonnet agents, 5 with the change and 5 with the binding and citation guidance cut out, same design request in isolated scratch notebooks, scored from the notebook files. With: 5/5 took the same shape, a design Task whose shape Decisions are bound and archived with it, the open Question left live and unbound, every step Task citing its Decision by full id, check green. Without: 5 different shapes; design knowledge stayed live in 5/5 (2 to 5 records each), 3/5 bound records to the long-lived hub or a step instead, one run's step Tasks cited no Decision at all; check green in all. Frictions seen in both arms: 4+ agents guessed an id from a long title and hit the 40-character slug cap; 2 tried to block a Task on a Question and fell back to hold.
- 2026-09-30 Maksim Yaromin/claude-code: The skill has a reads table, the rule to cite governing Decisions by full id and when to bind a record; the worked session runs a design Task end to end. A test with five agents with and five without the change: with it all five took the bound shape, without it five different shapes. PR #96.
