---
id: question.should-the-global-scope-accept-more-than
type: question
state: open
title: Should the global scope accept more than the six verbs named?
by: Maksim Yaromin
from: task.global-root-and-flags
created: 2026-08-30
updated: 2026-08-30
---

task.global-root-and-flags named six verbs that take --global: decide, note, retire, view, list, search. It also stated the invariant — task and question verbs refuse the global scope — which leaves ready, check, archive, expunge, edit, overview and status unplaced. Shipped: the derived rule. A verb refuses --global when it can only create or move a task or a question; every read and every knowledge write runs in either scope. The accepting set is the six plus edit, check, archive, expunge, overview, status, ready. Option A (shipped): keep the derived rule. Without edit a global note can never be corrected in place, and skills-as-notes is the driving use case; without check the one corruption-naming tool cannot look at the user's notebook; and ready answering an empty queue is as truthful as status and overview printing 0 tasks, which they do in the global scope either way — refusing one while the other two answer zero would give two answers to the same question. Option B: narrow to the six named and refuse the rest, so the global surface is exactly what was ruled and grows one verb at a time. A second, separate call: --global is a scope declaration, not a path check, so anb add --title x --notebook <the home notebook> does file work there and nothing can flag it — a notebook carries no mark saying it is the user's. The knowledge-only invariant is enforceable at the flag or nowhere.
