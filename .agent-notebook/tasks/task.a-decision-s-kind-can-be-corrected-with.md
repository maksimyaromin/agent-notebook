---
id: task.a-decision-s-kind-can-be-corrected-with
type: task
state: review
title: A Decision's kind can be corrected with edit
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
from: task.clean-this-notebook-with-bound-records
created: 2026-09-30
updated: 2026-09-30
---

The session opening now carries only rule and drift Decisions, so a Decision's kind decides whether every session sees it. edit cannot change a kind: a kindless Decision (decision.concurrent-access-serializes-on-a in this notebook) or a product shape filed as a rule can only be corrected by supersession, which rewrites the record's id and history for a one-word fix. Proposal: edit --kind on a Decision or Note, refused on a bound record when the new kind is rule. Found in the field test of task.clean-this-notebook-with-bound-records.
- 2026-09-30 Maksim Yaromin/claude-code: Implemented. edit --kind writes a Decision's or Note's kind from its type's vocabulary (shared guard with add --kind; a Task or Question is refused). A bound record cannot become a rule unless the same edit clears its task; the refusal names --clear task. The skill's intent table has a row for it, and the records reference lists edit --kind. Applied here: decision.concurrent-access-serializes-on-a is now kind rule, so the session opening shows it.
- 2026-09-30 Maksim Yaromin/claude-code: Smoke check (Sonnet, one pass) over the id cap and edit --kind found no behaviour defect and three gaps, all closed: a replayed --kind is now tested to move nothing, a Decision given a Note's kind is refused in the test, an explicit id is tested at 96 bytes accepted and 97 refused, and add --help states the 96-byte limit beside the 64-character slug.
