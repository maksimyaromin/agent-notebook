---
id: task.anb-v1
type: task
state: open
title: anb v1
by: Maksim Yaromin
via: claude-code
tags: epic
blocked-by: task.spike-storage-format
blocked-by: task.spike-record-model
blocked-by: task.spike-agent-interaction
blocked-by: task.rust-scaffold
blocked-by: task.core-grammar-parser-renderer
blocked-by: task.core-record-model-invariants
blocked-by: task.core-dependency-graph
blocked-by: task.core-status-budget
blocked-by: task.cli-task-cycle
blocked-by: task.cli-decide-note-ask-answer
blocked-by: task.cli-check-archive-edit-search-overview
blocked-by: task.setup-sessionstart-hooks
blocked-by: task.skill-from-help-ci-drift-check
blocked-by: task.npm-distribution
blocked-by: task.gate-final-cli-name
blocked-by: task.git-reconciliation
blocked-by: task.why-rust-dossier-release-gate
blocked-by: task.github-dev-flow-actions-ci-fmt-clippy-te
blocked-by: task.epic-pattern-scoped-queries-status-hub-g
blocked-by: task.release-gate-v1
blocked-by: task.milestone-self-host-switch
blocked-by: task.milestone-cli-complete
blocked-by: task.global-notebook
blocked-by: task.task-graph-visualization-for-fun
created: 2026-08-29
updated: 2026-08-29
---

The founding idea: ship anb v1 — a project's working memory as typed records with lifecycles, self-hosted, agent-operated through the CLI, released through the v1 Definition of Done gate. Every task of the plan is a child of this hub: the hub is blocked-by each of them and closes when the last one does. Children born from now on also carry from:<this hub>; the migrated ones predate the origin edge and only the future edit surface can add it retroactively.
