---
id: task.cli-expunge-verb-refusal-guarded
type: task
state: open
title: CLI: expunge verb, refusal-guarded
by: Maksim Yaromin
via: claude-code
from: decision.expunge-for-records-born-by-mistake
priority: 2
created: 2026-08-29
updated: 2026-08-29
---

expunge deletes a record born by mistake, leaving no trace. It refuses while any inbound reference exists — a mention edge from another body, or an origin link on a record spawned from the target — and the refusal lists every blocker with its carrier so the repair path is obvious. No --force. Acceptance: expunging an unreferenced record removes its file; expunging a referenced one fails with the full blocker list and touches nothing; after repairing the references the same call succeeds.
