---
id: note.report-the-notebook-knows-who-is-asking
type: note
state: retired
title: Report: The notebook knows who is asking: identity on the read side and taken-by on Tasks
by: Maksim Yaromin
from: task.the-notebook-knows-who-is-asking
created: 2026-09-08
updated: 2026-09-08
---

# Report: the notebook knows who is asking

Closes [issue 65](https://github.com/maksimyaromin/agent-notebook/issues/65): `by` was written and never read back, and a team sharing one notebook could not see whose work was whose.

## Result

Nobody assigns a Task; someone takes it. The tool now knows the identity it acts under and uses it on both sides.

| Operation | Default | Lever |
|---|---|---|
| `add` | signs `by` with the identity | `--by <name>` |
| `comment` | signs the log entry `by/via` | `--via <tool>` |
| `start` | records who took the Task as `taken-by`; refuses a Task someone else took (`taken`) | `edit <id> --taken-by <name>` hands it over; `edit --clear taken-by` erases it |
| `status` | the board; the caller's active Tasks lead, another person's line carries their name | none needed; the hook gets the same |
| `ready` | everyone's queue, a `taken-by` column | `--mine`, `--by <name>`, composable with `--for` |
| `list` | every live record | `--mine`, `--by <name>`, composable with `--for` |
| `search` | matches `by`, `via` and `taken-by` beside id, title, tags and body | none |
| `--json` rows of `list`, `ready`, `search` and Status `active` | carry `by` and `taken-by` when set | none |
| the identity | `ANB_BY`, else git `user.name`; absent, the tool signs nothing, takes nothing, and refuses `--mine` with the fix named | `ANB_BY` |

Rules and every other read stay the whole project's; only work is viewer-relative, and only where a wrong default would make one person resume another's Task.

## Decisions

- `taken-by`, in the family of `blocked-by` and `resolved-by`, after the owner's ruling that Tasks are taken, not assigned; an earlier `assignee` shape with `add --assignee` was superseded in the notebook (`decision.a-task-is-taken-never-assigned-start`). `add` carries no way to take a Task for someone else.
- `start` refuses a Task someone else took even on a replay: `already` would tell a second person the work is theirs. Hand-over is a deliberate `edit`, and the refusal's `try:` leaves the name as a template rather than filling in the caller's.
- `ready` is a fact about the notebook shared by epics and the graph, so another person's Tasks stay in it, marked; `--mine` narrows. Status is the one viewer-relative surface.
- The identity lives on the `Notebook` (`with_identity`), beside the user's notebook (`with_user`): a fact about the session, read by `create`, `comment`, `start` and `status`. `ready`/`ready_for`/`list`/`list_for` collapsed into `ready(&Filter)`/`list(&Filter)`, a `Filter { hub, by }` mirroring `GraphSlice`.
- `Attribution { by, taken_by }` is the one home of "whose": rows, the dashboard line and the filter read it; `encode::author` is the one spelling of `by/via`, shared by the log entry and the cited record.
- `--by X` keeps records X created or took; the dashboard marks and orders by the one name (`taken-by`, else `by`).

## Evidence

- `./scripts/check.sh` green: fmt, clippy `-D warnings`, 600+ tests including 20 new behaviour tests across Core, CLI and process, doctests, rustdoc, skill and reference pages regenerated without drift.
- `pnpm docs:check` green: 20 pages reachable, every link resolves.
- New tests proved red on a wrong expectation before green: dashboard order, the `taken` refusal, the `--mine` hint.
- Skill: SKILL.md orient and plan paragraphs, common-mistake row, worked session (`--via` on a comment, `list --mine`), a `taken` refusal example. Book: records, replies, status, session, tasks, customization, agents, quickstart.

## Smoke check

A Sonnet 5 agent re-read the whole diff against the engineering instruction. Fixed: the driving Task's title and body still said "assignee"; an explicit `--by` was written untrimmed while its neighbours were trimmed (now every signature passes `guarded_name`, test extended); the worked session's `list --mine` showed no contrast (the term Note is now recorded `--by Grace` and drops out of Ada's list); test scaffolding still said `GIT_IDENTITY`. Accepted as is: a malformed `ANB_BY` holding a newline is refused as `by: must be one line`, which names the envelope field rather than the variable; the Core does not know where the host found the name, and the field is what would be malformed.

## Limits

- Matching is exact on the name string; two spellings of one person are two people. `ANB_BY` is the lever.
- The offline race remains: two clones can `start` the same open Task before syncing; the merge conflict now shows two `taken-by` lines instead of two bare state flips.
- A Task active from before this release carries no `taken-by`; Status marks it with its `by`, and `edit --taken-by` claims it.
- `held` and `review` sections carry no name yet; `show` does.
