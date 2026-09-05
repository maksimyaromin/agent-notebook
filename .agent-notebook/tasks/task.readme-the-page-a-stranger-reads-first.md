---
id: task.readme-the-page-a-stranger-reads-first
type: task
state: open
title: README: the page a stranger reads first
by: Maksim Yaromin
via: claude-code
from: task.release-gate-v1
tags: docs
blocked-by: task.skills
priority: 2
created: 2026-09-05
updated: 2026-09-05
---

Before the repository goes public a stranger must understand the tool from the README alone: what a notebook is, the four record types and their lifecycles, install by npx and by cargo, the ten commands a session actually uses with literal output, how setup wires an agent, where the skill comes from. Correct against the binary at HEAD, no promise the CLI does not keep. Acceptance: every command shown runs as written; a reader who has never seen this repository can create a notebook and close a task by following it.
