---
id: question.what-should-git-reconciliation-repair
type: question
state: routed
title: What should git reconciliation repair?
by: Maksim Yaromin
from: task.git-reconciliation
tags: core
routed-to: decision.reconciliation-reports-a-divergence-and
created: 2026-08-29
updated: 2026-08-30
---

The task names two authorities, the working tree and the git log, and says divergence is repaired and reported. Both halves of reporting shipped: a sha proof settled by git in the notebook's own repository, a report proof settled by a stat. The repair half was not built, because every candidate repair invents a fact. The tool cannot know which commit a lost proof meant, so rewriting it would be fabrication; it cannot know whether an active Task whose id appears in a commit is finished, since a commit mentioning a task is not a close; and it must not move a record's state on a guess, because the state machine is what every agent trusts. Printing both sides and stopping is this project's standing posture on an undeclared conflict. Two shapes if repair is wanted: a verb that clears a lost proof and reopens the task for a fresh close, or a Status line that offers the command without running it. There is also one prevention that invents nothing and was left out of scope: close accepts any --sha without asking git, and that is the one moment the claim can be settled while the human is still there to correct it.
