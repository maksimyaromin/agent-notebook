---
id: question.can-a-task-die-without-ever-being
type: question
state: closed
title: Can a Task die without ever being started?
by: Maksim Yaromin
from: task.graph-lavish-loop
tags: cli
resolved-by: task.close-drop-ends-a-task-without-work
created: 2026-08-31
updated: 2026-09-02
---

A Task overtaken before it was started has no honest exit: close accepts only active or review, so the workaround is a start that lies in the log — the task was never taken into work — followed by close --no-proof. Met live on task.graph-lavish-loop, superseded by two later decisions while still open. Options: teach close to accept open with an explicit waiver; add a cancel move (open → closed, stating why); or keep the start-then-close ritual as the answer and say so in the skill.
