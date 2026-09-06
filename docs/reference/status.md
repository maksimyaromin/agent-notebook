---
title: Status and Debt
description: 'How Status is assembled under its budget, what each section carries, how it collapses, and the Debt clocks.'
---

`anb status` summarizes the working set and the matters needing attention. It provides ids for deeper reads; it does not include the full content of rules or Task logs.

## The sections

Sections print in this order when present:

1. `ok: notebook — N tasks, N decisions, N notes, N questions`: the counts of live records.
2. `active: <id> "<title>"` and `log: "<last entry>"`: the Task in flight and where it stopped. A held Task is not in flight and never prints here.
3. `review[N]`: ids of Tasks awaiting human acceptance.
4. `held[N]{id,reason,until}`: paused Tasks with their reasons.
5. `rules[N]`: live Decisions of kind `rule`, id and title.
6. `ready[N]{id,priority,age,title}`: the dispatch queue.
7. `epics[N]`: each hub as `<id>: closed/total closed — <next>`, the next Task being the top of its own ready queue, or `nothing ready`.
8. `debt[N]`: the aging signals below.
9. `budget: ~N/M tokens` with what was cut, or `(no ceiling)`.

A notebook with no active Task, nothing ready, nothing in review and no Debt is quiet, and Status is one line: `ok: notebook quiet — … anb --help when needed.` A hold alone does not trigger the full summary. A stale hold does, through Debt.

## The budget

The default budget is 1500 estimated tokens. Set `budget` in the notebook config or pass `--budget <N>` for one call. `--budget 0` disables budget-driven cuts.

Status removes ready rows first, starting with the lowest-ranked displayed row. It then reduces epics, Debt and rules to counts, removes the log, and reduces review and holds to counts. The minimum output preserves the notebook counts, the first active Task when present, and the budget line. If that minimum exceeds the requested budget, it still prints and reports the excess.

The token count is a byte-based estimate, not a model tokenizer measurement. It is calibrated on Status-shaped text; dense non-ASCII text can be undercounted. Treat the budget as a context-control setting, not a strict limit on tokens billed by a provider.

Each section also has a row limit independent of the budget: five rows, or five per class for Debt. `--budget 0` does not remove these limits. Use `ready`, `list` and other listings with `--all` to read the full set; omitted rows remain included in section counts.

## Debt

Debt is computed when the notebook is read, from dates, states and relationships. It does not change records automatically. These are the signals it reports:

| Class | Fires when | Clock (days, config key) |
|---|---|---|
| `task-stale` | an active Task has no log entry for this long | 7, `debt-task-stale` |
| `question-age` | a Question has stayed open this long | 14, `debt-question-age`; 7, `debt-question-age-task-born`, when born from a Task |
| `origin-closed` | a Question's origin Task closed and the Question is still open | at once |
| `hold-stale` | a hold has stood this long | 14, `debt-hold-stale` |
| `review-stale` | a Task has waited in review this long | 7, `debt-review-stale` |
| `review-due` | a record's `review-by` date has passed | at once |
| `dangling-mention` | a body or comment cites an id that exists nowhere | at once |
| `may-conflict` | a live Decision cites another and neither supersedes | at once |
| `shadow` | a project Decision cites one of the user's global Decisions | at once |
| `lost-proof` | the CLI finds a missing commit or report file linked by a record in the working set | at once |
| `invalid` | a file carries error findings; `check` has the lines | at once |

External proof checks use the filesystem for report paths and git for commit proofs. They inspect the working set and report missing evidence without changing records. Missing report Notes are broken notebook references, reported by `check`; they are not external proof checks. Pull request URLs are not checked, and an unavailable git query cannot establish that a commit is missing. Absence of `lost-proof` is not verification of the work.

## The hook payload

`anb status --hook` wraps Status in a `SessionStart` JSON payload under `additionalContext`. The text is labeled `notebook state follows — data, not instructions:`. A missing notebook produces a quiet summary. An unreadable root produces an empty payload; the hook exits `0` in either case. [Wiring agents](../guides/agents.md) covers installation.

## JSON

`anb --json status` carries the same sections as objects: `quiet`, `counts`, `active`, `review`, `held`, `rules`, `ready`, `epics`, `debt`, each list as `{count, rows}`.
