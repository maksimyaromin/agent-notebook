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
- 2026-08-29 Maksim Yaromin: Scope addition (owner, 2026-08-29): retrofit the eleven existing closed-task report links. The ten pointing at a report file (.tmp/data/{s2,s3,g1,c1,c2,c3,c4,l1,l2,m1}/report.md) get their reports ingested as Notes born from their tasks and the link re-pointed at the note id — closed records have no edit surface, so the retrofit edits the link lines by hand (approved bypass) while the Notes are created through the CLI. One nuance to settle in-task: the storage-format spike links .tmp/docs/spec-anb-format.md — a living spec, not a close report; decide whether it gets a snapshot Note or a differently-kinded link.
- 2026-08-29 Maksim Yaromin: Spec-spike nuance settled: task.spike-storage-format keeps 'link: report .tmp/docs/spec-anb-format.md' unchanged. A snapshot Note would freeze a living document into the notebook and leave two sources of truth disagreeing as the spec moves — the exact bug class this project rejects by design. The proof kinds are equals: --note is the default because a report is finished writing, and a living spec is not a report. The other ten links are retrofitted: their reports are Notes born from their tasks, retired and archived beside the tasks they report on, since those tasks are already history. A close made today leaves its report Note live, and it follows its task into the archive when the task is filed.
