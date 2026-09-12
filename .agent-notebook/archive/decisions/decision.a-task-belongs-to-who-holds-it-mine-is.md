---
id: decision.a-task-belongs-to-who-holds-it-mine-is
type: decision
state: superseded
kind: rule
title: "A Task belongs to who holds it: mine is taken-by, and the pool is one read"
by: Maksim Yaromin
via: claude-code
from: task.mine-is-what-i-hold-a-task-belongs-to
tags: identity, tasks, read-side
supersedes: decision.a-task-is-taken-never-assigned-start
superseded-by: decision.work-responsibility
created: 2026-09-08
updated: 2026-09-12
---

Work belongs to who holds it; authorship is a different fact. A Task's taken-by names its holder and by its author. For a Task, mine is taken-by and nothing else: a Task one person wrote and handed to another is the other's in every read, and a Task nobody holds is nobody's, however many people wrote or planned it. by keeps meaning mine for a Question, a Note or a Decision, which nobody holds. So --by <name>, --mine and scope: mine answer with one person's work, the Tasks they hold and the records they wrote, and Record::belongs_to is the one home of that rule in the Core.

Handing over is a deliberate act and can happen at creation: add task --taken-by <name> writes a Task and hands it over in one command, add task --mine takes it for the identity, and edit --taken-by hands it over later. start takes a Task nobody holds and refuses one somebody else holds, as before. This reverses the earlier ruling that add carries no way to take a Task for someone else: a team in which one person plans and others do needs the planner's session to be one line per Task.

The pool, the Tasks nobody holds, is one read: --untaken on ready, list and graph. It is a whose answer like --by and --team, so it outranks scope: mine and is refused beside --by, --mine or --team. Only work is taken, so a Decision nobody signed is not in the pool.

A dashboard narrowed to one identity hides the pool with the rest of the team's work, so it counts the pool on an untaken: N line pointing at ready --untaken, and a pool alone keeps the notebook from reading as quiet: a session whose own queue is empty is not a session with nothing to do. The whole team's dashboard lists the pool in its queue and carries no count. JSON carries untaken.count on Status and untaken on a listing's filter.

Alternatives priced and rejected: keeping mine as created-or-took with a separate flag for held work, because a planner's own reads then keep opening on work that is not theirs, which is the defect reported; a viewer-relative ready that drops other people's Tasks, because ready is a fact about the notebook that epics and the graph share; a pool line on the team's dashboard too, because the team's queue already names the holder of every row.
