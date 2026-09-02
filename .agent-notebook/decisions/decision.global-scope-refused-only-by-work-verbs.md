---
id: decision.global-scope-refused-only-by-work-verbs
type: decision
state: active
kind: rule
title: A verb refuses the global scope only when it can only create or move a Task or a Question
by: Maksim Yaromin
from: question.should-the-global-scope-accept-more-than
tags: cli
created: 2026-09-02
updated: 2026-09-02
---

The user's notebook holds knowledge that outlives a repository: Decisions and Notes, nothing to work on. So `--global` is refused by exactly the verbs that can only create or move a Task or a Question (add, start, submit, close, return, reopen, hold, unhold, block, unblock, comment, ask, answer) and accepted by every read and every knowledge write (decide, note, retire, edit, view, list, search, ready, check, archive, restore, expunge, graph, overview, status). Narrowing to the six verbs first named for the scope (decide, note, retire, view, list, search) was rejected: without edit a global Note can never be corrected in place, and skills recorded as Notes are the driving use; without check the one corruption-naming tool cannot look at the user's notebook; ready answering an empty queue is as truthful as status and overview printing zero tasks, which they do in that scope either way. A verb added later is placed by this rule, never assumed. `--global` is a scope declaration, not a path check: `--notebook` pointed at the home notebook files work there and nothing can flag it, because a notebook carries no mark saying it is the user's. The knowledge-only invariant is enforced at the flag or nowhere.
