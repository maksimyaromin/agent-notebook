---
id: task.mine-is-what-i-hold-a-task-belongs-to
type: task
state: closed
title: Mine is what I hold: a Task belongs to who holds it, hand-over at creation, and the pool as one read
by: Maksim Yaromin
taken-by: Maksim Yaromin
tags: cli, read-side
link: issue https://github.com/maksimyaromin/agent-notebook/issues/75
link: note note.report-mine-is-what-i-hold-a-task-belongs-to
priority: 1
created: 2026-09-08
updated: 2026-09-08
closed: 2026-09-08
---

Under scope: mine a Task written by one person and handed to another still reads as the author's, because mine means created or took. The ruling asked for: a Task belongs to who holds it (taken-by) and nothing else; by stays the author and keeps meaning mine for a Question, a Note or a Decision. add task --taken-by hands a Task over as it is written; the Tasks nobody holds are one read, ready --untaken; status --team, --by and list --by stay. Beyond the ask: a narrowed Status whose own queue is empty must not read as quiet while the pool has work, so it counts the pool on an untaken: line; add task --mine takes a Task for the identity without spelling the name; the skill's session protocol takes from the pool when the user's queue is empty.
- 2026-09-08 Maksim Yaromin/claude-code: Core and shell done: Record::belongs_to is the one home of whose (holder for a Task, author otherwise); Filter.untaken and ready/list/graph --untaken are the pool; Draft.taken_by with add --taken-by and add --mine; Status counts the pool on an untaken: line under a narrowing and a pool alone is not quiet; JSON carries filter.untaken and status.untaken.count. Nine new tests, each proven red once. Next: skill text, worked session, the book, regenerated references.
- 2026-09-08 Maksim Yaromin/claude-code: Smoke check by a second model: no behaviour defect, four text findings fixed (status reference active line, the taken refusal's catalog sentence, a mistakes row, the quickstart starting row). Decision recorded and the taken-never-assigned ruling superseded. Both gates green. Report at .tmp/docs/report-mine-is-what-i-hold.md. Submitting for review; nothing committed.
