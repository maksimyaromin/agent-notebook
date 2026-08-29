---
id: task.core-record-model-invariants
type: task
state: closed
title: Core: record model + invariants
by: Maksim Yaromin
via: claude-code
tags: core
link: report .tmp/data/c2/report.md
blocked-by: task.spike-record-model
blocked-by: task.core-grammar-parser-renderer
created: 2026-08-29
updated: 2026-08-29
closed: 2026-08-29
---

4 record types, transitions, write-time invariants (supersession, routing, proof-on-close); idempotent mutations with already: true.

Progress 2026-08-27 (session 1): DONE pending owner review. Implementation + Opus 5 independent review (new protocol) + all 20 review findings fixed test-first. Owner decision folded in: routed-to naming a type no answer becomes (note/question) is broken-routing on every surface — rule lives in the record semantic pass (target type read from the id), covered by a red-first unit test and a corpus case. Gate green: fmt, clippy -D warnings pedantic, 114 tests. Report: .tmp/reports/c2-record-model.md. Working tree unstaged, uncommitted — review pause. Remaining open questions in the report: state-vs-location finding awaits l3 archive; Notebook &mut-for-reads friction deferred to c3.
