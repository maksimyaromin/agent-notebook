---
id: task.core-dependency-graph
type: task
state: closed
title: Core: dependency graph
by: Maksim Yaromin
via: claude-code
tags: core
link: report .tmp/data/c3/report.md
blocked-by: task.core-record-model-invariants
created: 2026-08-29
updated: 2026-08-29
closed: 2026-08-29
---

Cycle rejection on edge write; reverse index; close names unblocked Tasks; ready = open AND unblocked AND unheld, ordered by priority and age.

Progress 2026-08-27 (claude): implemented in anb-core, all tests green, self-review pending Opus.
- New `graph.rs` (pub(crate)): TaskGraph over blocked-by edges — is_blocked, path (for would-cycle), cycles (DFS, deterministic, self-loops excluded), unblocked_by (treats the closed id as closed regardless of stored state). Pure model, unit-tested without storage per the concept spec's testing decision.
- notebook.rs: `block`/`unblock` verbs (Blocked reply; WouldCycle error carries the full chain id → … → id); `ready()` -> Vec<ReadyTask{id,priority,created,title}>; `Closed` gains `unblocked` (open+valid dependents whose last live blocker this close was, ready-ordered; held ones included — hold gates ready, not the fact); check() gains dep-cycle findings on every member file at its edge line; mutation_errors renamed exclusion_errors.
- record.rs: blocked-by target must be task.* (bad-value) and self-block is a record-level dep-cycle — both judgeable from one file via self-describing id prefixes. grammar.rs: remove_field_value for repeatable keys. finding.rs: DepCycle (error).
- Decisions taken (not fixed by spec, flag at review): priority 0 = most urgent, unset ranks as neutral 2; within priority oldest `created` first, id tiebreak — NOTE: interaction spec §2 worked example shows younger-first within a priority, contradicts oldest-first, ask owner. Cycle findings live in check() only (like broken-supersession pairs), NOT in the mutation gate — so `unblock` stays the repair path for a hand-edited cycle; ready excludes cycle members via blockedness, not via invalidity.
- Tests: graph unit tests; tests/notebook.rs mods dependency_graph + ready_queue + check additions; corpus invalid cases task.self-block, task.blocks-on-a-note; props: Block/Unblock verbs + blocker-bytes-untouched assert. Every new test proven to fail (expectation-flip sweep, 31 flips, all red, restored).

Opus 5 review round 2026-08-27: 17 findings (3 defects), all fixed except two left deliberately.
- Fixed: unblock now admitted through the mutation gate for errors sitting on the record's own blocked-by lines (load_live_admitting + edge_borne), so a corrupted edge — self-block, wrong-typed target, dangling — never freezes its own repair; module doc rewritten to state the true gate (own errors + dangling refs; cross-record findings are check's alone); cycles() doc states the one-cycle-per-back-edge guarantee with a two-pass test proving the overlapped cycle surfaces after repair; open_questions_from now excludes invalid Questions like every derived surface; ready/unblocked_by_close share one open_rows body; Blocked reply struct renamed Edged (record model reserves "blocked" for the computed state); archive-cycle write refusal pinned by test; ready-rank test now exercises the id tiebreak; graph.rs in-module tests deleted (every boundary specified at the Notebook seam per the testing skill); proptest-regressions artifact removed; comment restatements trimmed.
- Left: ready keeps oldest-first within priority — reviewer independently concludes the interaction spec's worked example (younger-first) is the wrong side and its rows should be re-sorted; owner ruling requested. unblock deliberately accepts any well-formed id (not just task.*) — that laxness is the repair path for wrong-typed edges, now stated in its doc.
- Gate green after fixes: fmt, clippy -D warnings, 145 tests; the 11 new/reworked tests proven able to fail (flip sweep, all red, restored).
