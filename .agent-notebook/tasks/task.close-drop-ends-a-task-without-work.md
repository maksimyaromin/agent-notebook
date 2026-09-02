---
id: task.close-drop-ends-a-task-without-work
type: task
state: open
title: close --drop: a Task ends without work, stating why
by: Maksim Yaromin
from: task.improvements
tags: cli
priority: 3
created: 2026-09-02
updated: 2026-09-02
---

A Task overtaken before it was started has no honest exit: close accepts active or review only, so the ritual is a start nobody meant, then close --no-proof. Add `--drop "<why>"` to close as the fourth way to end a Task, parallel to `answer --drop`: accepted from open, active and review; the reason is mandatory and lands in the log; the close date is stamped; no proof link is written. `--no-proof` keeps meaning done with nothing to show and keeps refusing open. State stays closed: the log carries the distinction, so epic progress, archive and reopen are untouched. `--drop` beside any other proof is a conflict, as two proofs are today. The generated skill teaches it. Answers question.can-a-task-die-without-ever-being.
