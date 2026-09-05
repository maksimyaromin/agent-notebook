---
id: task.setup-sessionstart-hooks
type: task
state: closed
title: setup + SessionStart hooks
by: Maksim Yaromin
via: claude-code
tags: cli
link: note note.report-setup-sessionstart-hooks
blocked-by: task.spike-agent-interaction
blocked-by: task.cli-task-cycle
blocked-by: task.core-status-budget
blocked-by: task.milestone-cli-complete
blocked-by: task.improvements
created: 2026-08-29
updated: 2026-09-05
closed: 2026-09-05
---

setup installs the AGENTS.md snippet and a project-level SessionStart hook; verified working in Claude Code and Codex.
- 2026-09-05 Maksim Yaromin: Verified live 2026-09-05 in scratch projects: Claude Code (claude -p) received the hook's Status and echoed the active line; Codex (codex exec, project trusted, hook reviewed or bypassed) did the same from the project .codex/hooks.json setup writes, and answered NO STATE without the review step — the notice setup prints. Two traps met: codex exec blocks on a non-tty stdin (run it with stdin closed) and project trust must stand in config.toml, a -c override did not load the project hooks.
- 2026-09-05 Maksim Yaromin: Review: seven findings, all taken — plan-then-write so a refusal leaves the project untouched; links left alone; stray markers refused; the header no longer claims a verb. gate green.
