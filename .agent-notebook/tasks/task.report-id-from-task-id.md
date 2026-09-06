---
id: task.report-id-from-task-id
type: task
state: review
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
- 2026-09-06 claude-code: the report id is `note.report-` plus the Task's slug, cut at a word boundary only when the id cap leaves no room for the suffix; free_id extracted so the collision suffix applies as before; two core tests proven red once, the CLI snapshots and the tasks guide updated; the rendered skill and the book examples keep their ids since their task slugs equal their title slugs; gate and docs check green; smoke check running
- 2026-09-06 claude-code: smoke check: no defect; two should-fix taken (the report id minted through create's one corpus read, via one minting seam with two callers; the suffix rationale has one home); gate green; submitting
