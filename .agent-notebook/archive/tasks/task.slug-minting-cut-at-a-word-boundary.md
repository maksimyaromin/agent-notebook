---
id: task.slug-minting-cut-at-a-word-boundary
type: task
state: closed
title: Slug minting: cut at a word boundary
by: Maksim Yaromin
via: claude-code
from: task.milestone-self-host-switch
tags: cli
link: sha 526ef9a
created: 2026-08-29
updated: 2026-08-29
closed: 2026-08-29
---

Ids minted from long titles truncate mid-word: task.github-dev-flow-actions-ci-fmt-clippy-te and task.epic-pattern-scoped-queries-status-hub-g both came out of the self-host migration with a severed last word. The length cap should land on a hyphen so every kept word survives whole; an explicit --id stays the escape hatch.
- 2026-08-29 Maksim Yaromin: Cut lands on the last hyphen at or before the 40-char cap; a first word longer than the cap is still cut short, since overrunning would spend the id grammar's 64-byte budget. The two severed ids stand: they are cited from six records, there is no rename verb, and ids are never reused. Review confirmed no panic on hostile titles, the collision suffix stays within budget, and all four load-bearing decisions die under mutation.
