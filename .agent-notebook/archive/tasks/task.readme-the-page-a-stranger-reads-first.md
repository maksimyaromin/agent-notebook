---
id: task.readme-the-page-a-stranger-reads-first
type: task
state: closed
title: README: the page a stranger reads first
by: Maksim Yaromin
via: claude-code
from: task.release-gate-v1
tags: docs
link: note note.report-readme-the-page-a-stranger-reads
blocked-by: task.skills
priority: 2
created: 2026-09-05
updated: 2026-09-05
closed: 2026-09-05
---

Before the repository goes public a stranger must understand the tool from the README alone: what a notebook is, the four record types and their lifecycles, install by npx and by cargo, the ten commands a session actually uses with literal output, how setup wires an agent, where the skill comes from. Correct against the binary at HEAD, no promise the CLI does not keep. Acceptance: every command shown runs as written; a reader who has never seen this repository can create a notebook and close a task by following it.
- 2026-09-05 Maksim Yaromin: README written from a literal run of the binary in a scratch notebook: the problem, the four record types, install by npx and cargo, one session from add to check with every reply as printed, the record file, the shape of work (hub, --from, block, ready, hold), setup with its table of what it writes, the generated skill, --global, development. Added the MIT LICENSE. Links the docs at agent-notebook.supolka.dev; the maintainer sets the DNS or the docs task adjusts. Filed the '1 tasks' plural as its own task. Review running.
- 2026-09-05 Maksim Yaromin: Review: one must-fix (the close step read a report nothing had written; the page now writes it first), one should-fix (ids-never-reused scoped to lifecycle moves, delete named), one nit (rustdoc step in the gate). Every block reproduced byte for byte.
