---
title: Status and Debt
description: 'Work summaries, shared output budgets, personal scope and computed maintenance signals.'
---

`anb status` summarizes work: active Tasks, reviews, holds, ready work and open Questions. `anb recall` combines that summary with relevant project knowledge and personal practices. Both provide ids and commands for deeper reads. Neither chooses a session's Task merely because it appears first.

## The sections

Status carries the following fields in both TOON and JSON:

| Field | Meaning |
|---|---|
| `quiet` | no active work, review, ready work, untaken pool, open Question or Debt |
| `by` | the identity filter, when narrowed |
| `team` | a command to see the whole team's work, when narrowed |
| `more` | the same Status without an output ceiling |
| `counts` | live Task, Decision, Note and Question counts for the whole notebook |
| `active` | active, unheld Tasks, each with its latest log entry when available |
| `review` | Tasks waiting for review |
| `held` | paused Tasks with reason and optional resumption date |
| `ready` | startable Tasks in queue order |
| `untaken` | the ready, unassigned pool count and its read command |
| `questions` | open Questions, own first and then oldest first |
| `debt` | maintenance signal count and its read command |
| `budget` | estimated output size and the requested ceiling |

Each work section is `{count, omitted, rows}`. A quiet notebook still returns a structured summary. A hold alone does not make it busy; a stale hold does, through Debt. Decisions and Notes do not appear as work rows.

## Whose work

Status is the whole team's by default, with your work first. `--mine` uses `ANB_BY` or git `user.name`; `--by <name>` selects a colleague. Work follows the holder or the person it is addressed to, not merely the author who created it. An unassigned Task remains in the pool.

A notebook configured with `scope: mine` narrows Status without a flag. `--team` widens one call. The reply exposes the resolved identity and a widening command so an empty personal result cannot be mistaken for an empty notebook. Without an identity, personal work filters return an actionable refusal; the native hook carries read failures as diagnostic context.

Narrowing changes visibility, not validity. A Task blocked by a colleague's work stays blocked when that work is outside the selected scope.

## The budget

Status and Recall default to 1500 estimated tokens. Set `budget` in notebook config; Status also accepts `--budget <N>` for one call. Status `--budget 0` disables cuts and includes every row. Recall `--all` lifts its selection and text bounds.

The CLI selects one reply document for both formats and measures its actual TOON encoding, including the trailing newline. `budget.limit` is the requested ceiling, or `null` when unbounded; `budget.spent` is the resulting estimate. JSON carries those same values and selections.

A bounded Status starts with at most five rows per section. To fit the ceiling, it removes ready rows first, then Questions, holds and reviews, then active log text and extra active rows. Counts and omission markers remain. The minimum retains notebook counts, section counts, navigation and the first active row when present. That row is an ordered summary, not remembered session focus. If the minimum exceeds a tiny requested ceiling, `spent` reports the excess honestly.

Recall budgets its entire composite once. Selection takes turns across project, personal and global knowledge, preserving relevance order within each source. A large project therefore cannot consume every initial memory slot. The reply's `sources` array reports matching and omitted record counts for each audience; this selection order is not a rule for resolving conflicting instructions.

To fit the budget, Recall first reduces work rows and long memory excerpts, keeping up to 256 characters before dropping lower-ranked records from sources with more than one visible record. It then trims focused-record text, fields and relationship rows. Under tighter ceilings it can further shorten excerpts, omit the last record from a source or reduce invalid-file rows. Counts, audience omissions and expansion commands remain explicit. The focused record's identity is separate from Status ordering.

The estimate is byte-based, not a provider's tokenizer. It controls context size but does not predict billed tokens exactly. Core callers receive complete ordered models; output selection belongs to the CLI.

## Debt

Debt is computed from dates, states and relationships when read. It never changes records automatically. `anb debt` lists signals in the following order, bounded to 20 rows by default and lifted with `--all`.

| Code | Condition | Default clock or trigger | Row fields |
|---|---|---|---|
| `task-stale` | an active Task has no recent log entry | 7 days; `debt-task-stale` | `id`, `days` |
| `question-age` | a Question remains open | 14 days, or 7 when born from a Task; `debt-question-age`, `debt-question-age-task-born` | `id`, `days` |
| `origin-closed` | an open Question's origin Task closed | immediately | `id`, `origin` |
| `hold-stale` | a hold has stood too long | 14 days; `debt-hold-stale` | `id`, `days` |
| `review-stale` | a Task remains in review | 7 days; `debt-review-stale` | `id`, `days` |
| `review-due` | a record's review date passed | immediately | `id`, `date` |
| `dangling-mention` | body text cites an id absent from the same notebook | immediately | `id`, `target` |
| `lost-proof` | the host establishes that a linked commit or report file is missing | immediately | `id`, `proof` |
| `invalid` | a record has error findings | immediately | `file`, `errors` |

Shared tags and citations do not prove that two Decisions conflict. Personal notebooks do not validate shared references or change shared Debt. Resolve differences explicitly in the relevant records.

External evidence checks inspect filesystem paths and git commits without modifying them. Pull request URLs are not checked. An unavailable git query cannot establish that a commit is missing; absence of `lost-proof` is not proof that work was verified.

## The hook payload

The installed `anb hook` adapter wraps a budgeted Recall document in the host's `SessionStart` JSON envelope, under `additionalContext`. It identifies that content as notebook data, not instructions, and carries the current session's focus when available. Unreadable state produces a diagnostic rather than an empty-memory report, without failing the session hook. [Wiring agents](../guides/agents.md#enable-the-session-hook) covers installation, session input and environment propagation. `status` is a work query; it has no hook flag.

## JSON

`anb --json status` has the same document described above. A row carries `by`, `taken-by` and `to` when present. Debt rows carry `code`, the fields in the table and a human-readable `line`; programs should use the named fields.

Recall adds `work`, `focus`, `memories`, `sources`, `count`, `omitted`, `invalid`, `more` and the single outer `budget`. Each memory names its scope and read command. Each source carries `{scope, count, omitted}`; omitted body characters are reported on the individual body. Equal ids in different notebooks remain distinct; personal guidance is not merged into team knowledge.
