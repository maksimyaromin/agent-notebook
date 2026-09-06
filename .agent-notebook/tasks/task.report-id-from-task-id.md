---
id: task.report-id-from-task-id
type: task
state: open
title: close --note mints the report id from the Task id
by: Maksim Yaromin
via: claude-code
from: task.release-0-3-0
link: issue https://github.com/maksimyaromin/agent-notebook/issues/44
priority: 1
created: 2026-09-06
updated: 2026-09-06
---

The report Note of a Task closes into `note.report-<task slug>`, guessable from the Task and never cut on a stopword; the title keeps its Report: prefix; the collision suffix applies as today; existing reports keep their ids. Evidence: the core tests, the regenerated skill session and the book examples show the new id.
