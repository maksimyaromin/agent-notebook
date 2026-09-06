---
id: note.report-report-id-from-task-id
type: note
state: retired
title: Report: close --note mints the report id from the Task id
by: Maksim Yaromin
from: task.report-id-from-task-id
created: 2026-09-06
updated: 2026-09-06
---


# close --note mints the report id from the Task id

`close --note` titled the ingested report `Report: <task title>` and minted its id from that title through the same slug cut every title goes through, so the prefix spent seven of the forty characters and the cut landed on whatever word the title had there: `note.report-grill-rounds-2-and-3-how`, and in this notebook `note.report-setup-appends-its-snippet-as-its` and `note.report-status-json-carries-debt-as`, both minted this same day. An id is read far more often than it is minted, and a reader could not name a Task's report from the Task.

## What changed

- The report's id is `note.report-` followed by the Task's own slug: `task.setup-snippet-paragraph` closes into `note.report-setup-snippet-paragraph`. The Task slug is already unique and already cut at a word boundary, so the report id is guessable from the Task and never ends on a stopword the cut left behind.
- The collision suffix applies as before: `create` mints every id through one seam, the title for an ordinary record and the Task id for a report, so a second report on the same Task after a reopen lands beside the first with the suffix the notebook has always used.
- A hand-given Task id whose slug leaves no room under the id cap for the prefix and a suffix is cut at a word boundary, through the same cut a title goes through. Every minted Task id fits whole.
- The title keeps its `Report: ` prefix for listings; existing reports keep their ids; an interrupted close still resumes by finding the Note through its origin and body, whatever its id.
- The tasks guide states the rule beside the close example.

## Evidence

- Two core tests, shown red once: a Task titled `A demo record` with id `task.demo` closes into `note.report-demo`, and a fifty-six-character slug is cut to `note.report-alpha-beta-gamma-delta-epsilon-zeta-eta-theta`, the last word boundary that leaves room for a suffix. The existing tests for the replay, the resumed close, the second report after a reopen and the archived report hold; the CLI snapshots name the new id.
- The rendered skill and the book examples print the same report ids as before, since their Task slugs equal their title slugs, so the skill check and the reference pages stayed green without regeneration.
- `scripts/check.sh` green; `pnpm docs:check` green.
- Smoke check: no correctness defect; the boundary arithmetic, the collision path against a live and an archived Note, the resumed close and the printed ids in the skill and the book were each traced. Should-fix, taken: the report's id was minted from a second read of the corpus on every close, so the minting is now one seam with two callers, the title for every record and the Task id for a report, inside the one corpus read create already makes. Should-fix, taken: the rationale for the collision suffix had two homes after the extraction and has one. Nit, noted: a Task id without a dot is unreachable through the close path, so the fallback in the report id stays.

Pull request: https://github.com/maksimyaromin/agent-notebook/pull/51, squash-merged on a green CI check.
