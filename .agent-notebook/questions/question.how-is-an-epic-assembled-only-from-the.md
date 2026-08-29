---
id: question.how-is-an-epic-assembled-only-from-the
type: question
state: open
title: How is an epic assembled only from the hub side discovered?
by: Maksim Yaromin
from: task.epic-pattern-scoped-queries-status-hub-g
tags: cli
created: 2026-08-29
updated: 2026-08-29
---

A hub is detected by the pairing: blocked by a record that also carries it as Origin. Scope needs no pairing, so task.release-gate-v1 — ten blocked-by edges, no Origin children — is fully scopeable by list --for and ready --for, yet never reaches the epics block, so no session can discover it from Status. task.milestone-cli-complete qualifies through exactly one archived child; expunge that child and a 6/8 epic vanishes from the dashboard. Widening detection to any Task with a blocked-by edge would make every blocked task an epic. The notebook already carries another signal: the hubs wear tags epic, gate and milestone, and the owner ruled that an idea's hub carries tags: epic — while the design doc rules tags out for decomposition. The two statements are in tension and the tie-break is the owner's.
