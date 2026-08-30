---
id: task.split-notebook-rs-the-reply-vocabulary
type: task
state: open
title: Split notebook.rs: the reply vocabulary and check
by: Maksim Yaromin
via: claude-code
priority: 3
created: 2026-08-30
updated: 2026-08-30
---

notebook.rs is 3363 lines with a ~1500-line impl, and three modules below it import from it — encode takes ReadyTask, status takes Epic and ReadyTask, debt takes CitedProof, Resolver and path_stem — while notebook imports all three. Two cuts fix the inversion: a module for the reply vocabulary (Draft through FileFinding, plus carriers_of), which must carry Counts and Cited along or the cycle only relocates, and a module for check (the check_* functions, cycle_findings, finding_order, task_edges, origin_edges, dangling_finding, error_findings) whose one entry point is the verb. Use a directory module so the shared internals stay pub(super). The cuts rejected as shuffle: a module for the error type, and a read-verb/write-verb split, which would expose twelve internals to buy a file boundary. Acceptance: the module graph is acyclic, every reply is byte-identical to the current build on the project's notebook and on a 1000-record synthetic, and the gate is green.
