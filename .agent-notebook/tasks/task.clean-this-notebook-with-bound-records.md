---
id: task.clean-this-notebook-with-bound-records
type: task
state: review
title: An agent tidies this notebook with the new lifetime
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
from: task.records-live-with-their-task
blocked-by:
  - task.bind-a-record-to-its-task
  - task.show-reads-every-edge
created: 2026-09-30
updated: 2026-09-30
---

The field test of decision.a-record-bound-to-a-task-leaves-with-it on this repository's own notebook. No migration: an agent reviews the live Decisions and Notes, binds the design ones to the Tasks they served or archives them with their Task, and reports every friction it meets.
- 2026-09-30 Maksim Yaromin/claude-code: Field test run on this notebook with the new binary. Reviewed 29 live Decisions and 3 live Notes. Every Decision is standing product knowledge whose origin Task or Question is long archived; none served a single piece of design work still in the working set, so none was bound. Bound and archived: note.the-first-adopters-findings (an idea every issue of which shipped) to task.release-0-3-0, by restore, edit --task, archive; the archive reply named it under bound and check stayed green. Left live on purpose: note.the-first-adopter-s-findings-shipped-in and note.the-release-track-closed-on-2026-09-06, the marathon summary Notes the owner's protocol asks for; whether they leave with their Tasks is the owner's call (the first was bound and unbound during the test, so its updated date moved). Frictions: (1) binding knowledge to a Task already archived takes three commands, restore, edit --task, archive, and the restore also brings back every record already bound to that Task; (2) no edit --kind, filed as task.a-decision-s-kind-can-be-corrected-with; (3) 22 live Decisions here are kind rule, many of them product shapes, so the session opening lists 20 rule titles and omits the rest; re-kinding them needs (2); (4) decision.concurrent-access-serializes-on-a carries no kind, so no session opening shows it.
- 2026-09-30 Maksim Yaromin/claude-code: Correction: the follow-up Task's id is task.a-decision-s-kind-can-be-corrected-with. Its title showed a fifth friction: the slug split `Decision's` into `decision-s`. Fixed under task.readable-id-by-default: an apostrophe now joins its word.
