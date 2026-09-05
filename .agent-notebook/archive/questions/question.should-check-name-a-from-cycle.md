---
id: question.should-check-name-a-from-cycle
type: question
state: closed
title: Should check name a from cycle?
by: Maksim Yaromin
from: task.epic-pattern-scoped-queries-status-hub-g
tags: core
resolved-by: decision.check-names-an-origin-cycle
created: 2026-08-29
updated: 2026-08-30
---

blocked-by cycles are refused at write and named by check when hand-edited in. Origin has neither guard: two records naming each other as from, or a record whose from names itself, pass check silently. Nothing derived breaks today — the scope walk only ever adds ids it has not seen, so it terminates — but Origin is the edge Debt keys its clocks on and the edge an epic's scope follows, and a cycle in it is as hand-edited a corruption as the other. Found during the epic-pattern review.
