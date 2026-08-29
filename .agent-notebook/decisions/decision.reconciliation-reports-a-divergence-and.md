---
id: decision.reconciliation-reports-a-divergence-and
type: decision
state: active
kind: rule
title: Reconciliation reports a divergence and never repairs it
by: Maksim Yaromin
via: claude-code
from: question.what-should-git-reconciliation-repair
tags: core
created: 2026-08-30
updated: 2026-08-30
---

The working tree and the git log are two authorities, and where they disagree the tool prints both sides and stops. Every candidate repair invents a fact the tool cannot know: which commit a lost proof meant, whether a Task a commit mentions is finished, what state a record should hold. A commit naming a task is not a close, and the state machine is what every agent trusts, so nothing here moves a record on a guess. Reporting is the whole of it: a sha proof is settled by asking git, a report proof by asking the filesystem, and a kind neither can settle is left alone — silence means not known to be lost, never verified. The one thing worth adding is prevention rather than repair, and it is carried forward as its own task: settling a --sha at the moment it is typed, while the human who typed it is still there to correct it.
