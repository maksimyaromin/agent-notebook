---
id: task.global-root-and-flags
type: task
state: closed
title: Global root and --global flags
by: Maksim Yaromin
via: claude-code
from: task.global-notebook
link: note note.report-global-root-and-global-flags
blocked-by: task.milestone-cli-complete
created: 2026-08-29
updated: 2026-08-30
closed: 2026-08-30
---

Resolve a second notebook root in the user's home. --global on decide, note, retire, view, list, search. Task and question verbs refuse the global scope with a structured error naming the reason. A record is global by residence: no new envelope field; grammar, Check, and output contract byte-identical across scopes.
- 2026-08-30 Maksim Yaromin: Design settled before code. (1) --global is sugar for --notebook <home>/.agent-notebook: same precedence rung, so naming both is refused the way close refuses two proofs — chosen_root beside notebook_root in fs_storage, the one home of root precedence. (2) The refusal rule is derived, not enumerated: a verb refuses the global scope when it can only create, move, or queue Tasks and Questions — add/start/submit/close/return/reopen/hold/unhold/block/unblock/comment/ask/answer/ready. That covers the owner's six accepting verbs and adds edit/check/archive/expunge/overview/status, without which a global Note can never be corrected or verified. Filed as a Question. (3) No ANB_GLOBAL_NOTEBOOK: --notebook already names any root, and a fourth precedence rung buys nothing the phase needs; tests point HOME at a temp dir.
- 2026-08-30 Maksim Yaromin: Opus 5 review found 13 items; the serious ones were mine. (1) A relative HOME silently wrote the user's private records inside the repository and made --global a different notebook per directory — the exact failure notebook_root's own doc forbids for ANB_NOTEBOOK. A home is exported once and outlives every cd, so a non-absolute one is now refused. (2) chosen_root computed half a decision and left its caller to compose the rest: folded into notebook_root, so the whole precedence has one interface and one Result. (3) refused_globally took no scope, so its name promised a condition only its single call site evaluated; it now takes global and moved to a new scope module — reply.rs promises renderer-shared decisions, and the chosen_proof precedent I cited is a private helper of execute, not a public pre-dispatch guard. (4) ready refused while status and overview print '0 tasks' in the same scope: two answers to one question. The rule is now writes-only — a verb refuses when it can only create or move a task or a question — and ready is accepted. (5) The classification tables transcribed the match arms, so a wrongly placed verb stayed green; split into the six the criteria name and the ones the derived rule adds, each with its reason. (6) Four comment claims were false, including 'every other verb reads or writes records of any type' — retire is decisions and notes only. Added: the precedence claim end to end, byte-identical output across scopes, residence assertions, and the project half of every refusal.
