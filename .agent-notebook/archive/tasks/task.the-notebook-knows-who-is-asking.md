---
id: task.the-notebook-knows-who-is-asking
type: task
state: closed
title: The notebook knows who is asking: identity on the read side and taken-by on Tasks
by: Maksim Yaromin
via: claude-code
link: issue https://github.com/maksimyaromin/agent-notebook/issues/65
link: note note.report-the-notebook-knows-who-is-asking
priority: 1
created: 2026-09-08
updated: 2026-09-08
closed: 2026-09-08
---

Closes issue 65. A team shares one notebook and by is written but never read back. Delivered: a Task records who took it as taken-by, written by start and refused for a Task someone else took (code taken); edit --taken-by hands it over and --clear taken-by erases it; --by and --mine on ready and list; Status leads with the caller's active Tasks and names who took another; search matches by, via and taken-by; list and ready JSON rows carry by and taken-by; comment signs by/via; ANB_BY as the identity the host acts under when git has none or another name is wanted. Nobody assigns a Task: the owner's ruling, decision.a-task-is-taken-never-assigned-start. Skill and book follow every change.
- 2026-09-08 Maksim Yaromin/claude-code: smoke check (Sonnet 5) returned five findings: four fixed (Task wording, --by trimmed through guarded_name, list --mine contrast in the worked session, IDENTITY in test scaffolding), one accepted (a newline in ANB_BY is refused as by: must be one line); check.sh and docs check green again; ready for review
