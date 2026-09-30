---
id: task.check-finds-unreadable-references
type: task
state: review
title: check finds a reference the tool cannot read
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
from: task.records-live-with-their-task
link: issue https://github.com/maksimyaromin/agent-notebook/issues/93
created: 2026-09-30
updated: 2026-09-30
---

The remainder of issue #93. check reports a body reference shaped like a record reference that does not resolve, such as [[slug]] whose slug names a record once its type prefix is added, and its repair writes the full id. The skill says how to connect: a record born from work is created --from it, and a Task cites the Decisions that govern it by full id. The first point of #93, defaulting the origin to the session focus, is rejected: work happens in the notebook without a focus, and decision.a-record-bound-to-a-task-leaves-with-it keeps binding explicit.
- 2026-09-30 Maksim Yaromin/claude-code: Implemented. The mention scan gained a wiki scan: [[target]] and [[target|label]] outside code, on one line, whose target is not an id. check reports each one whose slug names a record once a type prefix is added as unreadable-reference (warning), naming the full id to write; a slug naming nothing is left alone. There is no repair command: no verb rewrites one line of a body, so the finding carries the replacement in its message. Found on this notebook's own run: the byte walk sliced inside a multi-byte character; reproduced in the scan's test and fixed. The connect guidance lands in the skill with task.skill-shows-the-design-workflow. Docs: records (relations).
