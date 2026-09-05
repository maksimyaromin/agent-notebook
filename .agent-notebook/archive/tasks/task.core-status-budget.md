---
id: task.core-status-budget
type: task
state: closed
title: Core: Status + Budget
by: Maksim Yaromin
via: claude-code
tags: core
link: note note.report-core-status-budget
blocked-by: task.core-record-model-invariants
created: 2026-08-24
updated: 2026-08-28
closed: 2026-08-28
---

Section assembly by priority; token measurement; 'budget spent / what was cut' line; regression fixtures.

Design fixed 2026-08-28 (maintainer rulings): (1) full Debt incl. mention scan; record-model §8 amended. (2) rules section (US12); ladder: ready rows -> debt->count -> rules->count -> log+review -> floor (in-flight never dropped). (3) ready oldest-first; interaction §2 example re-sorted. (4) Budget Tokens|Unbounded, flag > config > 1500, '0 = no ceiling' both surfaces. Later rulings: review tasks are the gate's 4th signal; mention-borne debt classes bounded at 5 + hint; restore command only when something was cut. status command surface recorded on l1.

Progress 2026-08-28 (session 1): implemented in anb-core. New modules tokens.rs (deterministic o200k estimate ceil((6a+7n)/21), calibrated vs gpt-tokenizer; policy bands as constants), mention.rs (id-token scan, ASCII word boundaries), debt.rs (9 signal classes, origin-keyed clocks on live-state records only, pairs with by/via ranked oldest-first), config.rs (flat key:value, named-const key table, fail-soft), status.rs (gate, assembly, Collapse-enum ladder, budget line with fixed-point spent). notebook.rs: status()/config(), check() covers config, ALL derived queries unified onto resolvable_by_id (canonical-path semantics, matching the mutation gate). Specs updated per rulings (interaction §2/§4, record-model §8).

The independent review 2026-08-28: 28 findings (7 must-fix defects) — all fixed test-first; 3 frontier calls resolved by maintainer ruling; live-states clock gate fixed per the spec; estimator calibration claims rewritten to measured reality; harvested test bands replaced by policy constants.
