---
id: task.spike-storage-format
type: task
state: closed
title: Spike: storage & format
by: Maksim Yaromin
via: claude-code
tags: spike
link: report .tmp/docs/spec-anb-format.md
created: 2026-08-24
updated: 2026-08-24
closed: 2026-08-24
---

Mandatory deep-research sweep first (method: .tmp/docs/research-method.md; research subagents: Sonnet 5 only): serialization formats for LLM read/write efficiency, token-efficient encodings, structured-text round-tripping, agent memory formats; classify findings tried-vs-theory (theory weighs more). Then: formal grammar with escaping; envelope encoding; id scheme (hash vs slug-hash); .anb/ and archive layout; config; negative-corpus design. Close: format spec + concept-spec update + ADR for anything irreversible, with stated proof our practice beats the theory.

## Progress log

- 2026-08-24: dispatched. Read research-method.md, CONTEXT.md, spec, digest + dissections 01-03.
- 2026-08-24: research sweep done. 4 Sonnet-5 subagents, 62 verified cards: .tmp/docs/research/05..08-s1-*.md (protocol, hits log, WHY/HOW/WHAT cards, tried-vs-theory, verified refs, bearing-on-anb each).
- 2026-08-24: design done. .tmp/docs/spec-anb-format.md (grammar, body opacity, raw-span round-trip + 4-equation contract, field lexical layer, structural supersession, slug ids, .anb layout + config, 19 finding codes, negative corpus); .tmp/docs/adr/0004-record-file-format.md; concept spec updated.
- 2026-08-24: owner review held via Lavish (.tmp/lavish/anb-format-architecture.html). 15+ questions answered in place; every "beat the theory" claim re-challenged as claim/solid/overclaim/verdict. Outcomes folded into spec/ADR/concept: (1) id prefix form owner-chosen: full type word + dot (`task.parser-fences`, `decision.no-mise-toml`) over single-letter and Greek candidates; (2) supersedes/superseded-by lexically available on every record type; (3) supersession guarantee narrowed honestly: a recorded replacement can never be read as live; undeclared conflicts remain an S2 Debt question; (4) reachability model accepted in owner's formulation and recorded in concept spec: weight on progress/Tasks, task-linked records super-reachable, knowledge-read outside progress context = direct request or another tool on the same Core; (5) TOON verdict: real header+rows idea, overclaimed evidence, rejection provisional pending S3 measurement.
- 2026-08-24: OWNER APPROVED ("Принимаю" on id scheme; "можешь переходить к завершению данной задачи"). Closing with report = format spec. Nothing committed, nothing staged (owner did not ask).
Deliverables: .tmp/docs/spec-anb-format.md (owner-approved), ADR 0004, concept-spec updates, research 05-08 (62 cards), Lavish review artifact .tmp/lavish/anb-format-architecture.html
