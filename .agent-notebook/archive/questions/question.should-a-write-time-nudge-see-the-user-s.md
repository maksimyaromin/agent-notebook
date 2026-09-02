---
id: question.should-a-write-time-nudge-see-the-user-s
type: question
state: routed
title: Should the write paths and check see the user's notebook too?
by: Maksim Yaromin
from: task.global-shadow-surfacing
routed-to: task.write-paths-and-check-read-the-global-root
created: 2026-08-30
updated: 2026-09-02
---

Status now pairs a project rule with a standing rule of the user's notebook, and stops calling that citation a dangling mention. Two write-side surfaces still read this notebook alone. First, the nudge: decide, comment and edit probe a body's citations against the project, so recording the very rule that shadows a global one answers 'dangling: `decision.x`' while Status answers 'shadow: ... <-> global `decision.x`' — two names for one fact, one of them false. Second, and worse, check: a link line whose target parses as an id is verified like any reference, so declaring the edge — the natural reaction to a shadow row — makes check fail with dangling-ref on an id that exists. check is the gate, not a nudge. Option A (shipped): leave both, and let the shadow doc state that prose is the only edge that survives check. Option B: give the write paths and check the same second root, so one probe answers for every surface and the nudge can name the shadow at the moment the rule is written, which is when the author can still say so in the body. Option C: check keeps its scope but stops verifying link targets that name a record type it cannot see, which trades a false error for a silent one.
