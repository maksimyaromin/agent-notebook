---
id: note.report-mine-is-what-i-hold-a-task-belongs-to
type: note
state: retired
title: Report: Mine is what I hold: a Task belongs to who holds it, hand-over at creation, and the pool as one read
by: Maksim Yaromin
from: task.mine-is-what-i-hold-a-task-belongs-to
created: 2026-09-08
updated: 2026-09-08
---

# Report: mine is what I hold

Closes [issue 75](https://github.com/maksimyaromin/agent-notebook/issues/75): under `scope: mine` a Task the planner wrote and handed to a colleague stayed in the planner's queue and opened the planner's session, because `mine` meant created or took.

## Result

Work belongs to who holds it; authorship is a different fact. The issue asked for four things and the change delivers them, plus three the team workflow needs beside them.

| Asked | Delivered |
|---|---|
| For a Task, `mine` is `taken-by` and nothing else; `by` keeps meaning `mine` for a Question, a Note or a Decision | `Record::belongs_to` is the one home of the rule: the holder of a Task, the author of anything else. `--by`, `--mine`, `scope: mine`, the dashboard's own-first order and its marks all read it. A Task nobody holds is nobody's |
| Handing over at creation | `add task --taken-by <name>`, guarded like `edit --taken-by` (Task only, one non-empty line, trimmed) |
| The pool is one read | `--untaken` on `ready`, `list` and `graph`: the Tasks nobody holds. A whose answer, so it outranks `scope: mine` and is refused beside `--by`, `--mine` or `--team`; the truncation hint carries it; JSON filters carry `untaken` |
| `status --team`, `--by`, `list --by` stay | untouched; `--by <name>` now answers with the colleague's work rather than what they wrote |

Beyond the ask:

- **A narrowed Status counts the pool.** Under `scope: mine` a developer who holds nothing saw `notebook quiet` while ten Tasks waited for someone. The dashboard now carries `untaken: N — anb ready --untaken` when it is narrowed and the pool is not empty, and a pool alone opens the gate. The team's dashboard lists the pool in its queue and carries no count. JSON Status carries `untaken.count`.
- **`add task --mine`.** A developer's agent files a follow-up the developer will do without spelling their name; refused with the fix named when the host knows nobody.
- **The skill's session protocol takes from the pool.** Orient: resume the active Task, or the top of `ready`; when the own queue is empty and Status counts the pool, the top of `ready --untaken`. The team paragraph is rewritten around holder and author, and two common-mistake rows name the pool and `--mine`.

The issue's transcript, replayed on the built binary with `scope: mine`: Maks's `ready` holds only the Task taken for himself, `ready --untaken` the unassigned one, Alex's `start` succeeds, Maks's `status` shows no active line and counts the pool, Alex's opens on the Task, and `status --team` shows the whole board with Alex's line marked.

## Decisions

- `decision.a-task-belongs-to-who-holds-it-mine-is` records the ruling and supersedes `decision.a-task-is-taken-never-assigned-start`, whose "add carries no way to take a Task for someone else" this reverses.
- The filter gains a field, not an enum: every narrowing is a predicate over the same notebook and the command line refuses the contradictory pair, as it already does for `--mine` and `--team`.
- `Attribution` stays a pair of names; the rule that picks one lives on `Record`, where the type is known. The dashboard's mark reads `taken-by` outright because every marked line is a Task line.
- `--untaken` lives on `Narrowing`, not on `Whose`, so `status` does not take it: a dashboard of nobody's work is a queue, and `ready --untaken` is that read.

## Evidence

- `./scripts/check.sh` green: fmt, clippy, every test, doctests, rustdoc, the skill and the reference pages regenerated without drift. `pnpm docs:check` green: 20 pages reachable, every link resolves.
- Nine new tests (Core: the pool, the hand-over on a draft and its refusals, the handed-over Task off the author's dashboard, the pool count and the gate; shell: `add --taken-by` and `--mine`, the `--mine` refusal, `--untaken` under `scope: mine` with its hint and JSON count, the refused pairs), each proven red once on an inverted expectation.
- Six existing tests encoded the reversed promise and were rewritten to the new one; the worked session shows a hand-over at creation, `ready --untaken`, and a narrowed Status that counts the pool.
- Book: session, tasks, customization, records, status, replies; the commands reference regenerated.

## Limits

- Matching on names stays exact: two spellings of one person are two people.
- `comment`, `submit`, `close` and `hold` on a Task somebody else holds are not refused; only `start` is. Whether accepting a colleague's work should need a hand-over first is a policy the notebook does not rule on.
- The pool line names ready Tasks only; a blocked or held Task nobody holds is in `list --untaken`, not in the count.

## Smoke check

One Sonnet pass over the whole diff, with the engineering instruction loaded and every behaviour claim replayed on the built binary in scratch notebooks under several identities. No behaviour defect. Four text findings, all fixed: the status reference's active line still said "someone else took"; the `taken` refusal's catalog sentence said another person took the Task, untrue for a Task handed over at creation; a common-mistakes row mixed two temptations in one; the quickstart's starting row did not know the pool. A fifth line of the same kind, a comment in the JSON renderer, was swept with them.
