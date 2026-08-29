---
id: task.scan-code-span-quotation-write-time-unre
type: task
state: open
title: Scan: code-span quotation + write-time unresolved-id warning
by: Maksim Yaromin
via: claude-code
from: decision.backticked-id-is-a-quotation
priority: 1
created: 2026-08-29
updated: 2026-08-29
---

Two halves of one rule. (1) The mention scan skips markdown code spans: an id inside backticks is a quotation and creates no mention edge; a bare id stays a reference. (2) The write-time nudge: every mutating command that accepts body or comment text warns in its own output when that text introduces a bare id that does not resolve — suggest backticking to quote, or creating the record. No rejection: forward references stay legal. Acceptance: the four migrated quotation sites (backticked by hand) stop producing dangling-mention debt; a bare unknown id in a fresh comment produces the warning; a backticked unknown id produces neither warning nor debt.
