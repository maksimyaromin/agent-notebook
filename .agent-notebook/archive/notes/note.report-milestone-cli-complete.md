---
id: note.report-milestone-cli-complete
type: note
state: retired
title: Report: Milestone: CLI complete
by: Maksim Yaromin
from: task.milestone-cli-complete
created: 2026-08-29
updated: 2026-08-29
---

# Milestone: CLI complete

The acceptance close of the hub gating the integration-and-release phase. Every condition its body named is met, and the CLI is usable end to end for read and write without touching a record file by hand.

## What the milestone asked for, and where it landed

| Condition | Shipped |
|---|---|
| check / archive / edit / search / overview | `14ea75d` (before this phase) |
| `close --note` ingestion | `67700e7`, with the eleven existing proofs retrofitted in `cd5d06c` |
| expunge | `45f0e2c` |
| slug word-boundary minting | `526ef9a` |
| epic-pattern queries | `bfb0b40` |
| git reconciliation | `4b3e0ce`, corrected by `add9725` |
| notebook location / commit-policy ruling | `2ed0816` |

One further condition was added to the phase after the hub was written — the state-vs-residence Check finding, `9b5a693` — and closed with the rest.

## What changed in the tool, in one line each

- **Residence is a named axis.** A record's state and its live-or-archive home may disagree only as a finding: `archived-live-record` (error — no verb can move it, so it can never be settled) and `unarchived-settled-record` (warning — it is unfiled, not lost). `check` sorts errors first so a warning the settling verbs leave behind can never crowd out a real one.
- **A close can carry its report into the notebook.** `--note <path>` reads the file, lands it as a Note born from the Task, and links that Note as the proof. `--pr`, `--sha`, `--report` and `--no-proof` remain equals; `--report` is right for a living document, which a Note would freeze into a second source of truth.
- **A record born by mistake can leave.** `expunge` deletes it and refuses while any inbound edge exists — the five reference keys, a body citation, or an id-shaped `link` target — naming every blocker with its carrier. There is no override.
- **Minted ids cut at a word boundary**, so an id no longer arrives reading like a typo.
- **The notebook's location is configuration.** `--notebook` for a call, `ANB_NOTEBOOK` for a shell, discovery otherwise; a relative variable anchors on the project so one export cannot mean a different notebook in every directory.
- **Epics are hub Tasks.** `ready --for` and `list --for` scope the queue and the listing; Status and `overview` say where each epic stands. A hub is a Task blocked by a record that also carries it as Origin, and scope is what the epic waits on plus what was born inside it, both followed as far as they go.
- **Status names proofs the world no longer holds.** A `sha` git cannot find, a `report` file that is gone — reported, never repaired.

## What the phase cost, and what it taught

Seven tasks, sixteen commits, 495 tests green. Every task went through an independent Opus 5 review; every review found something real, and several found defects that unit tests could not have.

Four findings are worth carrying forward, because each was a class of mistake rather than a slip:

1. **A wrong claim in a comment is the same defect as a wrong branch.** The residence finding shipped with a message asserting "no derived query reads the archive" — disproved by reproduction, twice, since the first correction was equally false. Claims in committed text now get verified the way branches do.
2. **An expectation harvested from output certifies whatever the code does.** The expunge refusal said "2 records" for one record holding two edges, and an accepted snapshot locked it in. Expected values are derived from the contract first.
3. **A guard keyed on a derived value recognises the wrong things.** `close --note` resumed an interrupted write by recomputing an id, so a re-close after `reopen` silently kept the stale report. It now recognises only what its own interrupted run left: live, born from this Task, holding this very report.
4. **Running the tool on its own notebook finds what tests do not.** The last defect of the phase — a report path resolved against the notebook directory, accusing this repository of losing a file plainly present — passed every unit test and failed the first field test.

## What this milestone deliberately did not settle

Five questions are open and reachable from the records that raised them:

- `question.what-should-git-reconciliation-repair` — the reporting half shipped; every candidate repair invents a fact the tool cannot know. It also names the one prevention that invents nothing: `close --sha` accepts any string without asking git, at the one moment a human is present to correct it.
- `question.how-is-an-epic-assembled-only-from-the` — a hub with no Origin children is scopeable but never reaches the dashboard, and the tag convention and the design doc disagree about the tie-break.
- `question.should-check-name-a-from-cycle` — `blocked-by` cycles are guarded at write and named by `check`; Origin has neither guard.
- `question.should-archiving-a-task-carry-its-report` — a report Note does not follow its Task into the archive, and nothing names the split.
- `task.restore-verb-bring-a-record-back-out-of` — `archived-live-record` is an error with no repair path, because every verb refuses an archived id.

None of them blocks the phases this milestone opens.
