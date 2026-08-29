---
id: decision.check-names-an-origin-cycle
type: decision
state: active
kind: rule
title: Check names an Origin cycle
by: Maksim Yaromin
via: claude-code
from: question.should-check-name-a-from-cycle
tags: core
created: 2026-08-30
updated: 2026-08-30
---

A `blocked-by` cycle is refused at write and named by check; a `from` cycle was refused at write and named by nothing, so two records naming each other as Origin passed check silently. Origin is the edge the Debt clocks key on and the edge an epic's scope walks, so a loop in it is hand-edited corruption exactly as the other is. Check now names it at `origin-cycle`, on every member file at its own `from` line, carrying the walk that closes. It stays a finding two files hold together, which keeps it out of the mutation gate: edit --from still repairs a looped record, where a single-file rule would have frozen both records out of the one verb that fixes them. A record whose `from` names itself is the same finding with one member, because no single-file rule covers that case the way the dependency edge has one.
