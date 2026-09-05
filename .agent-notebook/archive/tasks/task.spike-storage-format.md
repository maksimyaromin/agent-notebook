---
id: task.spike-storage-format
type: task
state: closed
title: Spike: storage & format
by: Maksim Yaromin
via: claude-code
tags: spike
created: 2026-08-24
updated: 2026-08-24
closed: 2026-08-24
---

Literature sweep first: serialization formats for LLM read/write efficiency, token-efficient encodings, structured-text round-tripping, agent memory formats; classify findings tried-vs-theory (theory weighs more). Then: formal grammar with escaping; envelope encoding; id scheme (hash vs slug-hash); .anb/ and archive layout; config; negative-corpus design. Close: format spec + concept-spec update + a decision record for anything irreversible, with stated proof our practice beats the theory.

## Progress log

- 2026-08-24: design done: the format spec (grammar, body opacity, raw-span round-trip + 4-equation contract, field lexical layer, structural supersession, slug ids, .anb layout + config, 19 finding codes, negative corpus).
- 2026-08-24: Design review held. 15+ questions answered in place; every "beat the theory" claim re-challenged as claim/solid/overclaim/verdict. Outcomes folded into the spec: (1) id prefix form maintainer-chosen: full type word + dot (`task.parser-fences`, `decision.no-mise-toml`) over single-letter and Greek candidates; (2) supersedes/superseded-by lexically available on every record type; (3) supersession guarantee narrowed honestly: a recorded replacement can never be read as live; undeclared conflicts remain an S2 Debt question; (4) reachability model accepted in maintainer's formulation and recorded in spec: weight on progress/Tasks, task-linked records super-reachable, knowledge-read outside progress context = direct request or another tool on the same Core; (5) TOON verdict: real header+rows idea, overclaimed evidence, rejection provisional pending S3 measurement.
