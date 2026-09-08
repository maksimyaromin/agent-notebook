---
id: task.a-question-and-a-review-name-whom-they-wait-on
type: task
state: closed
title: A Question and a review name whom they wait on
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
link: issue https://github.com/maksimyaromin/agent-notebook/issues/79
link: note note.report-a-question-and-a-review-name-whom-they-wait-on
priority: 1
created: 2026-09-08
updated: 2026-09-08
closed: 2026-09-08
---

A to field on a Task or a Question names whom it waits on: add question --to, submit --to, edit --to and --clear to write it. A narrowed Status lists the Questions and reviews addressed to the reader beside their own, the review and questions tables carry a to column, list --to narrows by addressee, and JSON rows carry to beside by and taken-by. The smallest hand-off: the notebook is working memory, not a channel, so the skill mentions it in one sentence and teaches no workflow around it.
- 2026-09-08 Maksim Yaromin/claude-code: to is a field on a Task or a Question; Record::waits_on derives the fact from the state, so a Task addressed ahead of submit waits on nobody yet and a settled record on nobody; Record::concerns is what mine and --by read. Review is a table with taken-by and to, questions gain a to column, rows carry to in JSON, list --to narrows by addressee. Dropped from the issue: --to-me, start erasing to, Debt naming the addressee; the skill adds one sentence.
- 2026-09-08 Maksim Yaromin/claude-code: Smoke check replayed the issue's transcript under two identities with scope: mine; every claim held. Report at .tmp/docs/report-links-and-addressee.md carries the audit. Both gates green; submitted for the review pause, every file uncommitted.
