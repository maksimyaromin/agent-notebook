---
id: task.bind-a-record-to-its-task
type: task
state: closed
title: A record bound to a Task leaves and returns with it
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
from: task.records-live-with-their-task
link:
  - follows decision.a-record-bound-to-a-task-leaves-with-it
  - issue https://github.com/maksimyaromin/agent-notebook/issues/94
created: 2026-09-30
updated: 2026-09-30
closed: 2026-09-30
---

Implement decision.a-record-bound-to-a-task-leaves-with-it: the task field on Decision, Note and Question; add --task, edit --task, edit --clear task; refusal for a rule Decision and for a target that is not a live Task; archive and restore of a Task carry its bound records; an open bound Question refuses the archive; restore of one bound record clears its task; check accepts an active bound record in the archive and reports a broken binding; the close reply lists born but unbound records with the bind command. Records, replies and lifecycle reference pages move with it.
- 2026-09-30 Maksim Yaromin/claude-code: Implemented. `task` is an envelope field on Decision, Note and Question (an orphan elsewhere, a bad value on a rule or when it names no Task), written by add --task and edit --task and erased by edit --clear task. A target must be a live Task: settled-task refuses a closed one (try reopen) and an archived one (try restore, reopen). archive of a Task moves every live bound record first, in its state, and replies `bound`; an open bound Question refuses with open-questions naming each. restore of a Task brings its bound records back (replies `bound`); restore of one record whose Task stays archived clears its task, stamps updated and replies `unbound` (Core restore now takes today). close replies `unbound`: live non-rule Decisions, Notes and Questions born from the Task and bound to none, each with its bind command; open-questions also counts bound Questions. check: a live record left by its archived Task is broken-binding (repair edit --clear task); a live-state bound record in the archive while its Task is live is archived-live-record (repair restore). delete refuses a Task with bound records. The graph draws `bound` Task -> record (contract v5; atlas skill updated). Docs: records, replies, knowledge, tasks, session, ideas, README, AGENTS.md; refusal catalog has settled-task and open-questions.
- 2026-09-30 Maksim Yaromin/claude-code: Correction found by the worked design session: the close reply offers `anb edit <id> --task <task>` for a Task that has just closed, so refusing a closed target made the offered command fail. A binding target is now any Task in the working set, closed or not; an archived Task is refused with the existing `archived` code (try show, restore). The settled-task code is gone. This reads the decision's 'live Task' as 'a Task in the working set', which is the reading under which Q9 and Q15 agree.
- 2026-09-30 Maksim Yaromin/claude-code: The task field binds a Decision, Note or Question to a Task in the working set; archive and restore carry bound records, an open bound Question refuses the archive, close lists born but unbound records with their bind command, check reports broken-binding, the graph draws bound. PR #96.
