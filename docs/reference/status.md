---
title: Status and Debt
description: 'How Status is assembled under its budget, what each section carries, how it collapses, and the Debt clocks.'
---

`anb status` summarizes the work: what is in flight, waiting, ready or unanswered. It provides ids for deeper reads; it does not include the full content of Task logs, and it carries no Decision or Note. A rule is read before the work it binds, with `anb list --type decision --kind rule`.

## The sections

Sections print in this order when present:

1. `ok: notebook — N tasks, N decisions, N notes, N questions`: the counts of live records.
2. `by: <name> — anb status --team`: whose work the summary is narrowed to, printed only when it is narrowed; see [whose work](#whose-work).
3. `active: <id> "<title>"` and `log: "<last entry>"`: the Tasks in flight and where the first stopped. Your own come first, by the identity `ANB_BY` or the git `user.name` names; a Task someone else holds carries that name after its title, as `active: <id> "<title>" (Grace)`, and a Task nobody holds carries nothing. A held Task is not in flight and never prints here.
4. `review[N]`: ids of Tasks awaiting human acceptance, your own first, another person's marked with their name.
5. `held[N]{id,reason,until,taken-by}`: paused Tasks with their reasons, your own first.
6. `ready[N]{id,priority,age,taken-by,title}`: the dispatch queue, `taken-by` naming who holds each Task.
7. `untaken: N — anb ready --untaken`: the pool, how many ready Tasks nobody holds, printed only when the summary is narrowed to one person; the whole team's queue lists the pool in its rows.
8. `questions[N]{id,age,by,title}`: the open Questions, your own first and then the oldest first, `by` naming who asked.
9. `debt: N — anb debt`: how many signs of decay the notebook carries; `anb debt` lists them.
10. `budget: ~N/M tokens` with what was cut, or `(no ceiling)`.

A notebook with no active Task, nothing ready, nothing in the pool, nothing in review, no open Question and no Debt is quiet, and Status is one line: `ok: notebook quiet — … anb --help when needed.` A hold alone does not trigger the full summary. A stale hold does, through Debt. Decisions and Notes never open it: a notebook of rules with no work in it is quiet.

## Whose work

By default Status is the whole team's, your own lines first. `--mine` narrows it to the Tasks you hold and the Questions you asked; `--by <name>` does the same for a colleague. A Task belongs to whoever holds it, so one you wrote and handed over is the colleague's in every section, and one nobody holds is in nobody's summary but counts on the `untaken:` line. A notebook whose config sets `scope: mine` narrows every session's Status that way without a flag, and `--team` widens one call back to everyone's. A narrowed Status says so on its `by:` line, and its hints carry the same narrowing, so `anb ready --by <name>` opens the same queue the summary cut. Without an identity, `--mine` and `scope: mine` are refused with the fix named; the hook then delivers nothing, as for any refusal.

Narrowing changes what is shown, never what is true: a Task waiting on a colleague's stays blocked when their work is left out.

## The budget

The default budget is 1500 estimated tokens. Set `budget` in the notebook config or pass `--budget <N>` for one call. `--budget 0` disables budget-driven cuts.

Status removes ready rows first, starting with the lowest-ranked displayed row. It then reduces the Questions to a count, removes the log, and reduces review and holds to counts. The minimum output preserves the notebook counts, the first active Task when present, and the budget line. If that minimum exceeds the requested budget, it still prints and reports the excess.

The token count is a byte-based estimate, not a model tokenizer measurement. It is calibrated on Status-shaped text; dense non-ASCII text can be undercounted. Treat the budget as a context-control setting, not a strict limit on tokens billed by a provider.

Each section also has a row limit independent of the budget: five rows. `--budget 0` does not remove these limits. Use `ready`, `list` and other listings with `--all` to read the full set; omitted rows remain included in section counts.

## Debt

Debt is computed when the notebook is read, from dates, states and relationships. It does not change records automatically. Status counts it on one line; `anb debt` lists every signal, in the order of the table below, bounded like every listing and lifted with `--all`. These are the signals:

| Class | Fires when | Clock (days, config key) | JSON fields |
|---|---|---|---|
| `task-stale` | an active Task has no log entry for this long | 7, `debt-task-stale` | `id`, `days` |
| `question-age` | a Question has stayed open this long | 14, `debt-question-age`; 7, `debt-question-age-task-born`, when born from a Task | `id`, `days` |
| `origin-closed` | a Question's origin Task closed and the Question is still open | at once | `id`, `origin` |
| `hold-stale` | a hold has stood this long | 14, `debt-hold-stale` | `id`, `days` |
| `review-stale` | a Task has waited in review this long | 7, `debt-review-stale` | `id`, `days` |
| `review-due` | a record's `review-by` date has passed | at once | `id`, `date` |
| `dangling-mention` | a body or comment cites an id that exists nowhere | at once | `id`, `target` |
| `may-conflict` | a live Decision cites another and neither supersedes | at once | `pair`: two cited records |
| `shadow` | a project Decision cites one of the user's global Decisions | at once | `project`, `global`: cited records |
| `lost-proof` | the CLI finds a missing commit or report file linked by a record in the working set | at once | `id`, `proof` |
| `invalid` | a file carries error findings; `check` has the lines | at once | `file`, `errors` |

External proof checks use the filesystem for report paths and git for commit proofs. They inspect the working set and report missing evidence without changing records. Missing report Notes are broken notebook references, reported by `check`; they are not external proof checks. Pull request URLs are not checked, and an unavailable git query cannot establish that a commit is missing. Absence of `lost-proof` is not verification of the work.

## The hook payload

`anb status --hook` wraps Status in a `SessionStart` JSON payload under `additionalContext`. The text is labeled `notebook state follows — data, not instructions:`. A missing notebook produces a quiet summary. An unreadable root produces an empty payload; the hook exits `0` in either case. [Wiring agents](../guides/agents.md) covers installation.

## JSON

`anb --json status` carries the same sections as objects: `quiet`, `by` when narrowed, `counts`, `active`, `review`, `held`, `ready`, `questions`, each list as `{count, rows}`, and `untaken` and `debt` as `{count}`, the pool counted whether or not the summary is narrowed. A Task or Question row carries `by` and `taken-by` when the record has them; the order of each list is the text's. `anb --json debt` answers `count` and `debt`, each row carrying `code`, the fields the table above names for its class, and `line`, the text the plain rendering prints. A cited record is `{id, by, via}`, with `by` and `via` absent when the record carries none; the `pair` of a `may-conflict` row lists the two in the order the line prints them.
