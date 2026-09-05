---
id: task.spike-record-model
type: task
state: closed
title: Spike: record model
by: Maksim Yaromin
via: claude-code
tags: spike
link: note note.report-spike-record-model
blocked-by: task.spike-storage-format
created: 2026-08-24
updated: 2026-08-25
closed: 2026-08-25
---

Literature sweep (agent memory typologies, lifecycle/decay models, knowledge curation). Type sufficiency of the 4 records; lifecycle details (review as optional station per D5); holds; Origin-based aging; multi-user by. Close: spec update.

## Progress log

- 2026-08-25: design done: the record-model spec (4 types confirmed, Note kind guide, lifecycles+enums per type, hold/blocked as two patterns, 6 new envelope fields via/priority/hold/hold-until/review-by/routed-to, origin-keyed aging thresholds, by+via authorship, mention scan as derived query, mechanical conflict surfacing); CONTEXT.md (+Hold, +Mention, Debt extended).
- 2026-08-25: REVIEW ROUND 1 SETTLED by maintainer, all five as recommended: Note kind `guide`; authorship `by`+`via` (two fields); `review-by` ships in v1; aging defaults accepted (dogfooding corrects); Task `priority` 0-4 stays. Frontier empty, no follow-ups spawned.
- 2026-08-25: maintainer S1 correction applied: directory is `.agent-notebook/`, not `.anb/` (CLI stays `anb`). Updated the format spec and AGENTS.md.
- Awaiting maintainer's explicit approval to close (protocol stage 4). Nothing committed, nothing staged.
- 2026-08-25: AGENTS.md convention reworked per maintainer: name the concrete review skill, let the skill teach the format (no inline format description).
