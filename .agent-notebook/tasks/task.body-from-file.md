---
id: task.body-from-file
type: task
state: review
title: add and edit take the body from a file or stdin
by: Maksim Yaromin
via: claude-code
from: task.release-0-3-0
link: issue https://github.com/maksimyaromin/agent-notebook/issues/47
priority: 1
created: 2026-09-06
updated: 2026-09-06
---

--body-file <path> beside --body on add and edit, mutually exclusive, - for stdin; the text lands where --body lands, with no new invariant. The shell reads the file, as it reads a report for close --note. Evidence: CLI tests for the file, stdin and the refusal of both flags; the commands reference and the skill regenerated; the knowledge guide shows it once for a model or a spec.
- 2026-09-06 claude-code: --body-file on add and edit, refused beside --body, - for stdin, read through the host's one file seam so close --note - reads stdin too; three CLI tests and one process test through a real pipe, each shown red once; the skill teaches the flag for a model or a spec, the knowledge and domain guides show it; commands reference and skill regenerated; gate and docs check green; smoke check next
- 2026-09-06 claude-code: smoke check: no defect; two should-fix taken (the test harness swept to the file seam's name; an empty body file on edit clears the body, pinned by a test); gate green; submitting
