---
id: question.how-does-prose-mention-a-record-id-witho
type: question
state: closed
title: How does prose mention a record id without creating a mention edge?
by: Maksim Yaromin
via: claude-code
from: task.milestone-self-host-switch
resolved-by: decision.backticked-id-is-a-quotation
created: 2026-08-29
updated: 2026-08-29
---

Journal bodies imported during the self-host migration name example and corpus-case ids as text; the mention scan reads each id-shaped token as a live reference, so Status now carries four dangling-mention debt rows pointing at records that never existed (the carriers are task.spike-storage-format and task.core-dependency-graph). Needed: either an escape that lets prose quote an id without the edge, or a ruling that mentions inside closed records are exempt from debt. No repair path exists today short of the future edit surface.
