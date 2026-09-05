---
id: task.setup-sessionstart-hooks
type: task
state: open
title: setup + SessionStart hooks
by: Maksim Yaromin
via: claude-code
tags: cli
blocked-by: task.spike-agent-interaction
blocked-by: task.cli-task-cycle
blocked-by: task.core-status-budget
blocked-by: task.milestone-cli-complete
blocked-by: task.improvements
created: 2026-08-29
updated: 2026-09-05
---

setup installs the AGENTS.md snippet and a project-level SessionStart hook; verified working in Claude Code and Codex.
- 2026-09-05 Maksim Yaromin: Owner ruling 2026-09-05: live verification runs are permitted with the owner's OpenAI key, read from .env per process and never exported; spend frugally, the account holds about 50 dollars. Codex 0.153.4 is installed and logged in with that key; Pi (@earendil-works/pi-coding-agent) is installed with the OpenAI provider active. Pi gets the AGENTS.md snippet only, hooks are for Claude Code and Codex.
