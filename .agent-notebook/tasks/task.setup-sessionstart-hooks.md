---
id: task.setup-sessionstart-hooks
type: task
state: review
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
- 2026-09-05 Maksim Yaromin: Verified live 2026-09-05 in scratch projects: Claude Code (claude -p) received the hook's Status and echoed the active line; Codex (codex exec, project trusted, hook reviewed or bypassed) did the same from the project .codex/hooks.json setup writes, and answered NO STATE without the review step — the notice setup prints. Two traps met: codex exec blocks on a non-tty stdin (run it with stdin closed) and project trust must stand in config.toml, a -c override did not load the project hooks.
- 2026-09-05 Maksim Yaromin: Smoke check (Sonnet 5): seven findings, all taken — plan-then-write so a refusal leaves the project untouched; links left alone; stray markers refused; the header no longer claims a verb. Report at .tmp/docs/report-setup-hooks.md; gate green.
