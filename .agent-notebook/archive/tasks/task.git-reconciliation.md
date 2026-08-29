---
id: task.git-reconciliation
type: task
state: closed
title: Git reconciliation
by: Maksim Yaromin
via: claude-code
tags: core
link: sha 4b3e0ce
blocked-by: task.core-status-budget
created: 2026-08-29
updated: 2026-08-29
closed: 2026-08-29
---

On Status, the working tree and git log outrank Records; divergence is repaired and reported.
- 2026-08-29 Maksim Yaromin: Review found three blockers, all fixed and verified: the git query deadlocked past ~2700 cited shas because stdin was written before stdout was read, hanging the session-start hook forever (now written from its own thread, tested at 4000 proofs); cited_proofs() propagated outside the hook's fail-soft funnel, so an unreadable record made the hook exit 1 (moved inside); and git ran in the process cwd rather than the notebook's repository, so a healthy notebook read from another repo was falsely accused. It also caught that I shipped a quarter of the sentence and called it the reporting half — the working tree was never examined. Report proofs are now settled by a stat, and the spec states what is deliberately not divergence: a pr proof nothing here can reach, and an uncommitted notebook, which the location ruling makes legitimate.
