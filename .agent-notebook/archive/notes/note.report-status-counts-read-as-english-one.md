---
id: note.report-status-counts-read-as-english-one
type: note
state: retired
title: Report: Status counts read as English: one task, two tasks
by: Maksim Yaromin
from: task.status-counts-read-as-english-one-task
created: 2026-09-05
updated: 2026-09-05
---

# Counts read as English (2026-09-05)

Report for task.status-counts-read-as-english-one-task, a finding met while writing the README: the first line a stranger reads from the tool said "1 tasks, 1 decisions".

## What changed

One helper in the Core, `counted(count, noun)`, renders a count with its noun agreeing in number: `1 task`, `2 tasks`, `0 tasks`. `counts_phrase`, the one spelling of the per-type tally that Status and `overview` share, is built from it, so the quiet line, the Status header and the overview header read `1 task, 1 decision, 0 notes, 0 questions`. The CLI uses the same helper for the file counts of `setup` and `skill` and for the cut marker of a long body in `show`, `… 20 more lines: anb show <id> --all`, which reads `1 more line` for one.

Two refusal messages counted as well and were missed by the first pass: `is invalid (N findings)` and `is still referenced by N records` now go through the same helper. One count keeps its plural: the `… N more characters …` marker inside a cut body is data the tool parses back to find the head and the tail, and a single dropped character is not a case worth a second parser. The cut marker for lines is plural by construction, since a cut drops two lines at the least, and the code says so instead of carrying a branch that cannot run.

The test expectations that pinned the old spelling now pin the promise (one of each type reads singular), and the literal replies in the README, the front page, the quickstart and the session guide were re-rendered. The skill's worked session regenerated itself.

## Review

One review pass, on a scratch notebook: one of each type reads singular in the quiet line, the Status header and the overview; two read plural; setup and skill report their file counts; no plural after a one remains anywhere in the tree. It found the `invalid-record` refusal still saying `(1 findings)`, now fixed, and showed that the singular branch of the lines marker could never run, now removed; the doc comment on the helper restated its body and now states the one constraint that matters, regular nouns only.
