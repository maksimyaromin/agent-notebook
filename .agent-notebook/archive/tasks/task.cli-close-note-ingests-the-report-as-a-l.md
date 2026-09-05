---
id: task.cli-close-note-ingests-the-report-as-a-l
type: task
state: closed
title: CLI: close --note ingests the report as a linked Note
by: Maksim Yaromin
via: claude-code
from: decision.close-proofs-are-equals-default-note
link: sha 67700e7
priority: 2
created: 2026-08-29
updated: 2026-08-29
closed: 2026-08-29
---

close gains --note <path>: the file's text becomes a Note born from the task being closed, and the close carries that note id as its proof — one motion, no two-step dance. Existing flags stay fully functional equals: --pr, --sha, --report <path> (a file anywhere, machine-local is legitimate). Docs present --note as the recommended default route. Acceptance: closing with --note creates the Note, links it as origin-of the close proof, and a reader of the closed task reaches the report through the notebook alone.
