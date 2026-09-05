---
id: task.status-a-held-task-is-not-in-flight
type: task
state: review
title: Status: a held Task is not in flight
by: Maksim Yaromin
via: claude-code
from: task.the-cli-s-vocabulary-judged-term-by-term
tags: cli
priority: 3
created: 2026-09-05
updated: 2026-09-05
---

Met 2026-09-05: holding the active Task and starting the next one left Status printing two in-flight lines, the held one first. A hold is a deliberate pause, so the paused Task is not the one a session resumes from; in-flight should be the active Task that is not held, and Status should say separately that a held Task waits — Debt already has hold-quiet for a hold gone stale. Also worth deciding: whether start should refuse while another Task is active and unheld, since the protocol says there is never more than one in flight. Acceptance: a held active Task never prints as in-flight; a test poses exactly this notebook.
- 2026-09-05 Maksim Yaromin: Done 2026-09-05: active lines exclude held Tasks; a bounded held[N]{id,reason,until} section names what waits, collapsing with review and dropped at the floor; a hold opens no dashboard. start does not refuse a second active Task — decision.the-cli-keeps-a-record-s-invariants-not records why. Report at .tmp/docs/report-status-held-task.md; gate green; smoke check running.
- 2026-09-05 Maksim Yaromin: Smoke check (Sonnet 5): one must-fix — a closed Task still carrying its hold line lingered in the held section; the query now lists live Tasks only, with a test. One should-fix — the quoting of a reason holding a comma had no test; added. Gate green.
