---
id: note.report-core-dependency-graph
type: note
state: retired
title: Report: Core: dependency graph
by: Maksim Yaromin
via: claude-code
from: task.core-dependency-graph
created: 2026-08-29
updated: 2026-08-29
---

# c3 — Core: dependency graph (closed 2026-08-27)

Delivered in anb-core, all at the Notebook seam:

- `graph.rs` (crate-private): TaskGraph over `blocked-by` edges — blocked computation, path search, deterministic cycle enumeration (one cycle per back edge, stated in the doc), reverse "who did this close release" query. Nothing derived is stored.
- `block`/`unblock` verbs: cycle rejection at write with the full chain in the `WouldCycle` error (US19); idempotent byte-identical replays; `unblock` is the repair path — admitted through the mutation gate for errors sitting on the record's own `blocked-by` lines (self-block, wrong-typed target, dangling edge).
- `close` names the open Tasks whose last live blocker it was (US7), ready-ordered; held ones included (hold gates `ready`, not the fact).
- `ready()` (US9): open ∧ unblocked ∧ unheld; priority 0 first, unset = neutral 2, then oldest `created`, then id. Rows carry `created`; age is the caller's derivation (the Core holds no clock).
- `check`: `dep-cycle` on every cycle member at its edge line; record-level findings for self-block (dep-cycle) and `blocked-by` into a non-Task (bad-value); corpus cases for both.

Review: mandatory Opus 5 pass returned 17 findings (3 defects: unblock frozen out of its own repair, false module-doc invariant, overclaiming cycles() doc) — all fixed; 2 deliberate stands documented in the task body.

Open owner ruling: interaction spec §2 worked example orders ready younger-first within a priority; implementation (and reviewer) say oldest-first — the spec example should be re-sorted.

Gate: fmt, clippy -D warnings, 145 tests green; every new test proven able to fail via expectation-flip sweeps.
