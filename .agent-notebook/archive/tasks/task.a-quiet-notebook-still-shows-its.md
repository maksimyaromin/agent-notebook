---
id: task.a-quiet-notebook-still-shows-its
type: task
state: closed
title: A quiet notebook still shows its standing rules
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
link: issue https://github.com/maksimyaromin/agent-notebook/issues/66
link: note note.report-a-quiet-notebook-still-shows-its
priority: 1
created: 2026-09-08
updated: 2026-09-08
closed: 2026-09-08
---

Closes issue 66. Status and the session hook print the quiet line when a notebook holds rule Decisions and no Task, so a team that agreed how to work before filing its first Task starts every session blind to the law. A standing rule is a signal on its own: the gate before the ladder counts live rule Decisions, the rules to count rung of the ladder stays.
- 2026-09-08 Maksim Yaromin/claude-code: has_signal counts live rule Decisions; a shape Decision or a Note alone stays quiet; tests in Core and CLI, the gate test proved red inverted; status.md and session.md updated; check.sh and docs check green; report at .tmp/docs/report-standing-rules-open-status.md; smoke check running
