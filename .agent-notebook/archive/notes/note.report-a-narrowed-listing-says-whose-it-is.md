---
id: note.report-a-narrowed-listing-says-whose-it-is
type: note
state: retired
title: Report: A narrowed listing says whose it is
by: Maksim Yaromin
from: task.a-narrowed-listing-says-whose-it-is
created: 2026-09-08
updated: 2026-09-08
---

# Report: a narrowed listing says whose it is

## Result

A `list`, `ready` or `graph` narrowed to one identity opens with `by: <name> — anb <verb> … --team`, the line Status already printed, whether a flag or the notebook's `scope: mine` key narrowed it; JSON `list` and `ready` carry `by` beside `count`, as graph JSON already did in `slice`. The widening command carries every other narrowing of the call, so it lifts the same read. Everyone's read, `--team` and `--untaken` print no line.

Why: the `scope` key narrows by nothing the caller typed. An agent that never wrote `--mine` read `count: 0` as an empty epic or an empty notebook, which is the tool's own argument for refusing a malformed filter turned against its own default. Explicit flags were already clear; one rule for both cases costs one line the caller expected and needs no state about where the narrowing came from.

The skill's Orient section gains a table of whose question takes which flag, so the agent asks the right question before it reads the answer: mine, a colleague's, the epic's or the team's, the pool, and why a held Task is not in the queue. The replies reference and the session guide say the same in one sentence each.

`docs/contributing/releasing.md` gains the semantic-versioning rule no committed text carried: patch for a fix that changes no documented shape, minor for anything added, major for a removal, a rename or a changed meaning, and only the maintainer moves the major. The Decision records the same rule and why this release is a minor.

## Evidence

- `./scripts/check.sh` and `pnpm docs:check` green; the skill and the reference pages regenerated without drift.
- The scope-key test specifies the line on `ready`, `list --by`, an empty narrowed listing with its other flags carried, JSON `by`, the graph, and its absence under `--team`; the `--mine` hint test follows. Both proven red with `whose_line` cut.
- A smoke check by a second model replayed every narrowed read on the built binary under `scope: mine`; see below.

## Smoke check

One Sonnet pass over the diff with the engineering instruction loaded, every narrowed read replayed on the built binary under `scope: mine` in text and JSON. Every claim held: the line appears exactly when a read is narrowed to one identity, the widening call runs and answers everyone's, the truncation hint and the line agree on the flags, `--team` and `--untaken` print nothing. One finding, fixed: the graph's widening call was built from the filter alone and dropped `--focus`, `--depth` and `--full`, so it named the whole notebook instead of the same graph; it is now built from the whole slice, as the truncation hint is, and the test covers a focused graph.
