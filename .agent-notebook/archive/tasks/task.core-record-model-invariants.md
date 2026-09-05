---
id: task.core-record-model-invariants
type: task
state: closed
title: Core: record model + invariants
by: Maksim Yaromin
via: claude-code
tags: core
link: note note.report-core-record-model-invariants
blocked-by: task.spike-record-model
blocked-by: task.core-grammar-parser-renderer
created: 2026-08-24
updated: 2026-08-27
closed: 2026-08-27
---

4 record types, transitions, write-time invariants (supersession, routing, proof-on-close); idempotent mutations with already: true.

Progress 2026-08-27 (session 1): DONE pending maintainer review. Implementation + independent review (new protocol) + all 20 review findings fixed test-first. Maintainer decision folded in: routed-to naming a type no answer becomes (note/question) is broken-routing on every surface — rule lives in the record semantic pass (target type read from the id), covered by a red-first unit test and a corpus case. Gate green: fmt, clippy -D warnings pedantic, 114 tests. Remaining open questions in the report: state-vs-location finding awaits l3 archive; Notebook &mut-for-reads friction deferred to c3.
