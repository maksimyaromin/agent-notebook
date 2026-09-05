---
id: task.milestone-self-host-switch
type: task
state: closed
title: Milestone: self-host switch
by: Maksim Yaromin
via: claude-code
tags: milestone
link: note note.report-milestone-self-host-switch
blocked-by: task.cli-task-cycle
blocked-by: task.core-status-budget
created: 2026-08-29
updated: 2026-08-29
closed: 2026-08-29
---

Move this project's backlog from the previous tracker to anb by hand — the first field test; development continues in anb.
- 2026-08-29 claude-code: Backlog migrated from the previous tracker by hand through the CLI only: 22 Tasks (10 closed, replayed open→active→closed with report proofs in dependency order; 12 live), 35 blocked-by edges, 1 hold. Old→new ids: s1→task.spike-storage-format, s2→task.spike-record-model, s3→task.spike-agent-interaction, g1→task.rust-scaffold, c1→task.core-grammar-parser-renderer, c2→task.core-record-model-invariants, c3→task.core-dependency-graph, c4→task.core-status-budget, l1→task.cli-task-cycle, l2→task.cli-decide-note-ask-answer, l3→task.cli-check-archive-edit-search-overview, a1→task.setup-sessionstart-hooks, a2→task.skill-from-help-ci-drift-check, r1→task.npm-distribution, n1→task.gate-final-cli-name, gr→task.git-reconciliation, rel→task.release-gate-v1, ci→task.github-dev-flow-actions-ci-fmt-clippy-te, e1→task.epic-pattern-scoped-queries-status-hub-g, g2→task.why-rust-dossier-release-gate, viz→task.task-graph-visualization-for-fun, m1→task.milestone-self-host-switch.
- 2026-08-29 claude-code: Not migrated, by design: created/closed dates (no backdating in the CLI; history keeps its dates inside the migrated bodies), edges to nothing (the previous tracker kept satisfied edges to done work — replayed here instead), the previous tracker's report-path suffixes in titles (proof carries the path now).
- 2026-08-29 claude-code: Maintainer direction during review: every backlog needs its binding idea. Created task.anb-v1 (tag epic) — the founding hub, blocked-by all 21 plan tasks (closed history included; edges to closed tasks are accepted and read as satisfied). The migrated children cannot gain from:<hub> retroactively — origin is add-time only until the edit surface — so the hub-side edges carry the grouping alone for now.
