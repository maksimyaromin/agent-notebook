---
id: task.memory-contract
type: task
state: closed
title: Ground the memory contract in everyday use
by: Maksim Yaromin
via: codex
taken-by: Maksim Yaromin
from: task.shared-memory
created: 2026-09-12
updated: 2026-09-12
closed: 2026-09-12
---

Unify work audience selection, distinguish work composition from dependencies and context, and make the workflow resolve a named result before assigning work. Explain how selective memory, shared domain meaning, personal practices and readable files fit one model. Verify discovery and use by agents without the CLI or skill, then recheck the complete implementation and documentation.
- 2026-09-12 Maksim Yaromin/codex: Unified the configured work audience across Recall, its hook and the other work reads. Restricted composition to origin descendants while preserving the full readiness graph and contextual neighbourhood. Successful completion now refuses unfinished prerequisites; cancellation and replay retain their contracts. Rewrote the design chapter and clarified the supplied method around capture, consolidation, readable records, named results and responsibility. Full code gate passes: 837 workspace tests, one distinct doctest, lint, format, rustdoc and generated renderings. The documentation build verifies all 20 content pages. Fresh agents demonstrate whole-result and single-part selection and useful plain-file reading with an ordinary folder link. A weaker model still sometimes skips the explicit coordination question despite reading the instruction; this is an observed instruction-following limit, not a reason to add a state or flag. No implementation or source code was committed or published.
- 2026-09-12 Maksim Yaromin/codex: The design now has a consistent mapping from selective memory and shared domain meaning to record lifetimes, audiences, composition and session continuation. Corrected deterministic counterexamples are covered through Core and CLI interfaces. The complete code and documentation gates pass. Plain-file usefulness and named-work selection have observed agent evidence; the coordination instruction-following limit is recorded without weakening the expected behavior or adding workflow machinery. Superseded work-selection rules have explicit replacements. Changes remain uncommitted for review.
