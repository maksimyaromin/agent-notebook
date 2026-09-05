---
id: note.report-the-write-paths-and-check-read
type: note
state: retired
title: Report: The write paths and check read the user's notebook too
by: Maksim Yaromin
from: task.write-paths-and-check-read-the-global-root
created: 2026-09-05
updated: 2026-09-05
---

# The write paths and check read the user's notebook too (2026-09-05)

Report for task.write-paths-and-check-read-the-global-root.

## The hole it closes

Status read the user's notebook behind the project's and named a shadow with both ids; every write path probed a body's citations against the project alone, and `check` verified every id-shaped `link` target against the project alone. Recording the very rule that shadows a global one answered `dangling-mention: decision.tabs` while Status answered `shadow: … <-> global decision.tabs` — one fact, two names, one of them false — and declaring the edge as a `link`, the natural reaction to a shadow row, made `check` exit failure on an id that exists.

## What shipped

The user's notebook is now a property of the `Notebook` handle: `Notebook::new(storage).with_user(user)`, a shared `&dyn Storage` reference, so "read and never written" is a fact the type keeps. `status` lost its fourth parameter and reads the field; the CLI builds the handle once, for every command, from the root it already resolved. Behind the handle, every surface that resolves an id reads the same second root:

- **The write-time nudge.** `dangling_mentions` — reached from `add`, `comment`, `edit`, `close --note` and `close --reason` — no longer names an id the user's notebook holds, live or archived. It probes the user's storage at the id's canonical paths, and a root that cannot answer vouches for nothing: the hint is dropped and the write goes on.
- **The gate.** `check` resolves `link` targets against either root; the envelope edges — `from`, `supersedes`, `superseded-by`, `resolved-by`, `blocked-by` — must still be answered by the project. They are this notebook's own structure: an origin the clocks key on, a supersession the tool writes both halves of, a block the queue follows; a link points outward by nature, at a pull request, a commit, a path or a record, so a record of the user's is within its reach. An unreadable user root leaves the gate to the project alone.
- **The shadow.** Once a link to a global rule is legal, a shadow surfaces from it as from a citation in prose: the Debt heuristic reads the ids a project Decision names in its body and the id-shaped targets of its `link` lines through one `cited_ids`.

## Tests

Each reply that can name a dangling mention has a case posing a global id — `Created`, `Commented`, `Edited`, `Closed` by report, `Closed` by reason — plus: an unreadable user root drops the hint and fails no write; `check` passes a link to a global record and still refuses an origin in the user's home; an unreadable user root leaves the gate to the project alone; a shadow surfaces from a link; and end to end through the CLI, recording the rule that shadows a global one prints no nudge and `check` answers `count: 0`. The `UnreadableNotebook` fixture moved from the Status tests into the shared test root, since three modules now pose it. Three of the new tests were proved red by mutation before counting: the user probe answering false, the gate not reaching across, the shadow dropping link targets.

## Gate

`./scripts/check.sh` green.
