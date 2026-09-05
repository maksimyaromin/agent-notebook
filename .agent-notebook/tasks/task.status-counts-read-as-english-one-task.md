---
id: task.status-counts-read-as-english-one-task
type: task
state: review
title: Status counts read as English: one task, two tasks
by: Maksim Yaromin
from: task.readme-the-page-a-stranger-reads-first
tags: cli
created: 2026-09-05
updated: 2026-09-05
---

Met while writing the README: the quiet Status line prints '1 tasks, 1 decisions, 0 notes, 0 questions', and the first thing a stranger reads from the tool is a plural on one. The count line is a summary an agent parses, so the shape stays; the noun agrees with its number. Same for every other count rendered as '<n> <noun>s' in a reply, if any.
- 2026-09-05 Maksim Yaromin: counted(n, noun) in the Core: 1 task, 2 tasks; counts_phrase uses it for the four types (Status, overview), the CLI for setup's and skill's file counts and show's cut marker (N more lines). The characters marker inside a cut body stays: it is parsed back. Expectations and the literal blocks in README and docs updated; gate and docs check green; smoke check running.
- 2026-09-05 Maksim Yaromin: Smoke check: the invalid-record refusal counted findings in the plural too, now through the helper with still-referenced; the dead singular branch of the lines marker removed. Report at .tmp/docs/report-status-counts.md.
