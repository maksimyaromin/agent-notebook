---
id: note.report-graph-the-data-surface-every
type: note
state: retired
title: Report: Graph: the data surface every drawing is built from
by: Maksim Yaromin
from: task.graph-the-data-surface-every-drawing-is
created: 2026-08-30
updated: 2026-08-30
---

# Graph: the data surface every drawing is built from

## The call that reshaped it

So the CLI serves the graph and draws nothing.

## Removed

`crates/anb-graph` deleted whole — the SVG renderer, `map.js`, `map.css`, `index.html`, the vendored `dagre.min.js`, the browser tests — along with `--out`, `Reply::Mapped`, the `write_artifact` host seam and the tests that covered them. Two crates remain where there were three. **816,837 bytes of vendored JavaScript and 863 lines of client-side code are gone**; the crate that drew pictures held 378 lines of Rust against them, which is the ratio that made the case.

## The defect that made it urgent

`--json` was bounded to `ROW_BOUND`. An agent asking for the graph of this notebook received **20 nodes of 42 and 20 edges of 96**, with the true counts beside them. A drawing built from that is not a smaller picture of the notebook but a picture of one that does not exist. Structural blocks are now never bounded. A record's own prose still obeys `--all`, because prose is prose.

## The graph became the notebook's, not the queue's

| | before | after |
|---|---|---|
| nodes | 16, Tasks only | 36 live: 16 task, 14 decision, 1 note, 5 question |
| edges | `waits`, `born` | plus `mentions` — one record naming another in its prose |
| per node | id, state, archived, degree, epic, title | plus `type`, `ready`, `priority`, `created` |
| slices | `--for --ready --focus --archive` | plus `--type`; `--focus` centres on any record |

`ready` is the flag that removes the second call: whether a Task can be started now follows from rules the caller cannot see. It is three-valued — absent where the question does not arise, since answering "no" about a Decision would answer a question nobody asks of it.

## Review — 16 findings from a separate agent, all real

Four reproduced on live data before touching anything:

- **12 duplicate edges.** Mention suppression consulted the mentioning record's envelope, but `blocked-by` is declared by the waiter, so a blocker whose prose named its waiter drew a second line on the same pair — and every degree counted it twice. One pair of records is now one edge, and the declared word wins.
- **`--type` missing from the echoed slice.** `--type decision` read as a notebook of 14 records holding no tasks: exactly the false-sounding answer the unknown-kind refusal exists to prevent.
- **`--type` with `--focus` answered an empty graph at exit 0.** The kind filter ran before the neighbourhood walk, so the centre was not in its own walk. Kinds now narrow the result, never the walk.
- **`--help` still promised an HTML file** — the one thing the maintainer asked to remove, still in the shipped surface.

Structural fix behind three more: kinds were strings beside an existing `RecordType`. Typing them moved the refusal onto the flag, deleted the validation loop, and removed a fourth hand-written copy of the list of kinds.

Reversed after review: bodies under `--full` had been unbounded "for the same reason" as the structure. The reviewer was right that it is a different reason — an incomplete edge set is a wrong graph, a cut body is a shortened record. The bound came back.

Not taken: counting records whose own `type` field is unreadable when kinds are named explicitly. Stated in `--help` instead.

## Guarded

584 tests. Nine behaviours added or rewritten for this change, every one mutation-proved — each turns red when exactly the line it guards is reverted. Two old tests encoded the rule this change reverses (`the_map_draws_tasks_and_nothing_else`, `a_focus_on_a_record_that_is_no_task_is_refused`); rewritten to guard the new behaviour rather than deleted.

## Proved by drawing it

The whole notebook — 90 records, 179 edges, archive included — rendered as one page from a single `anb --json graph --archive --full --all`, with no library and nothing from this repository. Maintainer's verdict: almost ideal.

That is the argument closed: the CLI never drew it, and it did not need to.

## Left for tomorrow

Comments inside the page, so the loop is open-see-say without an external review harness — the reason the Claude Code skill is worth building.
