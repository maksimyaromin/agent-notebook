---
title: Sessions and collaboration
description: 'Continue the intended work, keep parallel sessions separate and inspect a colleague’s context without claiming it.'
---

Start with `anb recall`. It combines work with shared project knowledge and private practices. Follow the user's subject, open the relevant record and read source material when the current decision depends on it. `status` is the work-only dashboard; `recall` adds the knowledge needed to interpret it.

## Resume the intended Task

```sh
anb start
anb recall --for task.customer-exports
```

`start` without an id resumes a remembered session focus. The host supplies that session through `--session`, `ANB_SESSION` or `CODEX_THREAD_ID`, in that order. A session id identifies an agent conversation, not a person.

`start <id>` names the Task explicitly. If no unambiguous focus exists, the command reports the choice the agent needs to make. The latest update is not evidence that a Task belongs to this conversation.

A closed focus remains inspectable: it explains what finished before the agent chooses more work. A missing session file means there is no remembered focus. Malformed, unreadable or interrupted state produces a recovery error; neither case licenses a guess.

## Take the next part

```sh
anb start --next
anb start --next --for task.customer-exports
```

When this session has active, unheld work within the requested scope, `--next` resumes it. Otherwise it starts eligible work assigned to the current person before taking an untaken Task. An explicit `--for` can select a different scope without holding the previous Task. Selection and claiming happen under the same notebook lock, so two local sessions cannot both claim the same next Task.

`--for` selects the hub and its descendants through `from`. External prerequisites can block these Tasks without becoming part of the result. A Task must still be ready, and an assignment to somebody else is not ignored. `ready --for <hub> --team` inspects the full scoped queue without taking anything.

## Work in parallel

A person can have several active Tasks in separate sessions. Each session remembers one focus. Starting a different Task in one session neither holds another Task nor steals the other session's focus.

```sh
anb --session export start task.customer-exports
anb --session pricing start task.price-rounding
anb --session export recall
```

A second local session cannot silently treat another session's Task as exclusive work. Use `start <id> --join` for intentional collaboration. Joining does not transfer human assignment, so a colleague's Task still requires an explicit reassignment before it can be started by someone else.

Session state lives in the ignored `.sessions.tmp/` directory of the selected project notebook. Separate worktrees have separate notebook roots and session state while sharing project-specific personal practices. Switching branches in the same worktree does not switch session files. Session claims are local coordination, not a distributed lease across disconnected clones. Coordinate overlapping offline work through the team's existing process and reconcile the resulting Git changes.

Interrupted starts retain enough intent to compare the before and after record bytes. A retry completes the same transition when that comparison proves it is safe. It refuses intervening changes it cannot reconcile rather than overwriting them.

## Inspect or assign work

The accountable identity comes from `ANB_BY`, otherwise Git's `user.name`. `by` records authorship; `taken-by` records a Task's assignment. Neither is the session identity.

| Intent | Command |
|---|---|
| Inspect your work | `status --mine` |
| Inspect the team | `status --team` |
| Inspect Grace's work | `status --by Grace` |
| Inspect unassigned work | `ready --untaken` |
| Assign a Task to Grace | `edit <id> --taken-by Grace` |
| Release an assignment | `edit <id> --clear taken-by` |
| Request acceptance from Grace | `submit <id> --to Grace` |
| Address a Question to Grace | `add question "<title>" --to Grace` |

Explicit assignments require the user's direction. Reading another person's work does not authorize taking it.

Recall, its automatic hook and the other work views honor the same notebook `scope`: `mine` for personal work, `team` for everyone's work. The default is `team`; explicit audience flags override it for one request. Knowledge-only lists remain shared even under `scope: mine`; an explicit `--mine` or `--by` still filters their authorship. In Recall those flags narrow work, not shared knowledge. Replies identify narrowed reads and supply a command to widen them.

## Leave a useful continuation

```sh
anb comment task.customer-exports --via codex --body "The export remains tenant-scoped. The CSV fixture passes. Next: verify the empty-workspace case."
```

Write the result, supporting evidence and next concrete action. A multiline comment stays one attributed log entry. Each active Task retains its own latest entry, so parallel continuations do not overwrite one another.

Use a Question when an uncertainty must survive independently, a Decision for an established choice and a Note for reusable knowledge. A routine check need not become another record.

## Finish, wait or read further

Close verified work with an outcome on the Task. Archive that record when it no longer belongs in the working set; related knowledge stays live until it is explicitly retired or superseded.

Hold work only when something prevents progress, with a reason naming that condition. Changing attention is not a hold. `unhold` makes it eligible again when the condition is resolved.

Truncated replies name the omitted content and an expansion command. `recall --all` expands the complete memory read; `status --budget 0` removes dashboard budget cuts. `debt` lists matters needing attention, not instructions to discard old records. See [the reply contract](../reference/replies.md) for exact bounds.

After notebook changes, run `anb check`. Commit shared memory with the related work only when the user has authorized commits.
