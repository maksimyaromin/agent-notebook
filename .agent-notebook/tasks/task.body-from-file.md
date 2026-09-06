---
id: task.body-from-file
type: task
state: open
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
