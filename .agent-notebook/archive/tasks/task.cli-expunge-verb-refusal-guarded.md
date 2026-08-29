---
id: task.cli-expunge-verb-refusal-guarded
type: task
state: closed
title: CLI: expunge verb, refusal-guarded
by: Maksim Yaromin
via: claude-code
from: decision.expunge-for-records-born-by-mistake
link: sha 45f0e2c
priority: 2
created: 2026-08-29
updated: 2026-08-29
closed: 2026-08-29
---

expunge deletes a record born by mistake, leaving no trace. It refuses while any inbound reference exists — a mention edge from another body, or an origin link on a record spawned from the target — and the refusal lists every blocker with its carrier so the repair path is obvious. No --force. Acceptance: expunging an unreferenced record removes its file; expunging a referenced one fails with the full blocker list and touches nothing; after repairing the references the same call succeeds.
- 2026-08-29 Maksim Yaromin: Guard covers all five reference keys, body mentions, and an id-shaped link target — the last found by review: close --note writes 'link: note <id>', and neither expunge nor check saw it, so expunging a proof Note left a dangling link no surface reported. Both now read one rule (linked_record). Also from review: the refusal counted edges while calling them records, and its snapshot had locked that in; a duplicate-id record now loses both files.
