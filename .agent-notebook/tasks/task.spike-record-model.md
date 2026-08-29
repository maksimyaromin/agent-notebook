---
id: task.spike-record-model
type: task
state: closed
title: Spike: record model
by: Maksim Yaromin
via: claude-code
tags: spike
link: report .tmp/data/s2/report.md
blocked-by: task.spike-storage-format
created: 2026-08-24
updated: 2026-08-25
closed: 2026-08-25
---

Sweep per research-method.md (agent memory typologies, lifecycle/decay models, knowledge curation). Type sufficiency of the 4 records; lifecycle details (review as optional station per D5); holds; Origin-based aging; multi-user by. Close: spec update + ADR.

## Progress log

- 2026-08-25: dispatched. Read research-method.md, CONTEXT.md, concept spec, digest, research 08 (S1 agent-memory cards) to scope the sweep without duplication.
- 2026-08-25: research sweep done. 3 Sonnet-5 subagents per method, 47 verified cards: 09-s2-memory-typologies (17), 10-s2-lifecycle-aging (15), 11-s2-curation-mentions-multiuser (15). Key: persistence-semantics typing independently validated; Origin-based aging = confirmed gap + proven cousin principle; red-link evidence for mentions; conflicts = surface-don't-resolve.
- 2026-08-25: design done. .tmp/docs/spec-anb-record-model.md (4 types confirmed, Note kind guide, lifecycles+enums per type, hold/blocked as two patterns, 6 new envelope fields via/priority/hold/hold-until/review-by/routed-to, origin-keyed aging thresholds, by+via authorship, mention scan as derived query, mechanical conflict surfacing); ADR 0005 (proposed); concept-spec updates; CONTEXT.md (+Hold, +Mention, Debt extended).
- 2026-08-25: owner review via Lavish (.tmp/lavish/anb-record-model.html), three course corrections applied and memorized: (1) review artifacts must be fully self-contained — digested proofs, examples, diagrams in place, never research-card pointers, no internal cross-references (memory: research-is-for-me-not-owner); (2) reviews run in the grill-with-docs protocol — numbered frontier questions with recommended answers in rounds (memory: review-defense-style; convention added to AGENTS.md); (3) DDD/domain language answered in place: Note kind term IS the ubiquitous-language record.
- 2026-08-25: GRILL ROUND 1 SETTLED by owner, all five as recommended: Note kind `guide`; authorship `by`+`via` (two fields); `review-by` ships in v1; aging defaults accepted (dogfooding corrects); Task `priority` 0-4 stays. Frontier empty, no follow-ups spawned.
- 2026-08-25: owner S1 correction applied: directory is `.agent-notebook/`, not `.anb/` (CLI stays `anb`). Updated spec-anb-format.md (§1, §9), ADR 0004, concept spec (2 places), AGENTS.md. Research files left as historical snapshots.
- Awaiting owner's explicit approval to close (protocol stage 4). Nothing committed, nothing staged.
- 2026-08-25: AGENTS.md convention reworked per owner: name the concrete grill-with-docs skill, let the skill teach the format (no inline format description).
- 2026-08-25: OWNER APPROVED ("Ок. Закрывай задачу"). ADR 0005 flipped to accepted; spec status owner-approved; concept-spec theme closed. Closing with report = record-model spec. Nothing committed, nothing staged (owner did not ask).
Deliverables: .tmp/docs/spec-anb-record-model.md (owner-approved), ADR 0005 (accepted), .agent-notebook rename in format spec/ADR 0004/concept spec/AGENTS.md, CONTEXT.md (+Hold +Mention), research 09-11 (47 cards), Lavish artifact .tmp/lavish/anb-record-model.html
