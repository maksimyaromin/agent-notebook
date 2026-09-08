---
title: The session
description: 'Resume work from Status, choose the next Task, and leave a useful handoff.'
---

Start with `anb status`. It shows the work: active Tasks and the latest log entry, work in review or on hold, the ready queue and the open Questions, so you can continue without reading the notebook's history. A configured session-start hook provides this automatically; otherwise, run the command yourself. The standing rules are not on it: read them with `anb list --type decision --kind rule` before the work they bind.

## Resume the work

Read the active Task with `anb show <id>`. Its latest log entry should say what is established and what remains to do. Check held Tasks before choosing new work: a hold records an intentional pause, and its reason may still apply.

If there is no active Task, inspect the ready queue:

```text
$ anb ready
ready[1]{id,priority,age,taken-by,title}:
  task.negative-corpus-wired-into-ci,-,0d,-,Negative corpus wired into CI
```

`ready` lists open Tasks with no unresolved dependencies and no hold, ordered by priority and then age. Priority `0` is most urgent; `-` means none was set. The `taken-by` column names who holds a Task; `anb ready --untaken` is the pool, the Tasks nobody holds. Use `anb ready --for <hub>` for one epic, then `anb start <id>` to take a Task into work.

The supplied skill keeps one Task active at a time per person. The CLI allows several, so check Status before starting another.

## Several people, one notebook

The CLI knows who is asking: `ANB_BY` names the identity, else the git `user.name` does. Every `add` signs its record `by` that identity, and every `comment` signs its log entry `by/via`, so the person stays in the trail beside the tool.

A Task belongs to whoever holds it, and authorship is a separate fact: `taken-by` names the holder, `by` the author. `start` takes a Task nobody holds and records you as `taken-by`; a Task someone else holds refuses `start` with `taken`. A planner hands a Task over in the command that writes it, `anb add task "<title>" --taken-by <name>`, or later with `anb edit <id> --taken-by <name>`; `anb add task "<title>" --mine` takes a Task for yourself. A Task nobody holds is nobody's, however many people wrote or planned it. Status lists your own active Tasks first and names who holds any other, so you resume your work and not a colleague's.

Reading is the whole project's by default: `status`, `ready`, `list` and `graph` show everyone's records, your own first where the order matters. `--mine` narrows any of them to your work, the Tasks you hold and the Questions, Decisions and Notes you wrote; `--by <name>` does the same for a colleague; `--untaken` is the pool, the Tasks nobody holds. A notebook whose config sets `scope: mine` makes your own work the default for every read, including the session hook; Status then counts the pool on an `untaken:` line when there is one, so a session whose own queue is empty knows where its next work is, and `--team` widens one call to the whole project. `list --match <name>` finds the records that name a person, since it matches `by`, `via` and `taken-by`.

## Leave a useful log

```text
$ anb comment task.parser-accepts-fenced-bodies "fences parse; the indented-body case is next"
ok: comment task.parser-accepts-fenced-bodies — logged
```

Write what the next session needs to act: a result, an unresolved obstacle, or the next concrete check. Status includes the latest entry; `show` reads the full Task. A log that only says "made progress" gives the next session no starting point.

File an uncertainty as a [Question](knowledge.md#questions) with `--from <task>`. Record a ruling as a Decision and cite its id in the Task when it affects the work. These records let you follow an investigation without making the Task log explain every subject in full.

## Read beyond the summary

Status also reports work awaiting review, the open Questions and how much Debt the notebook carries. `anb debt` lists the Debt: matters that need attention, such as an old Question or a reference to a missing record. It does not change their state. Check whether the work is still relevant before acting: resolve the Question, update the Task, or record why it must stay paused. A date is a reason to look again, not evidence that the work is obsolete. Where an epic stands is a read of its own: `anb ready --for <hub>` is its queue, and `anb list --for <hub> --archive` its whole membership.

A compact section still has a count. `ready: 7` means there are seven Tasks, even if the budget omitted their rows. `anb status --budget 0` removes budget-driven cuts; individual sections still limit their rows. Use the relevant listing with `--all` for the complete set. [Status and Debt](../reference/status.md) specifies the sections, limits and clocks.

A quiet notebook produces one line. Knowledge alone leaves it quiet: a rule binds the work, and the skill reads it before the work rather than at every session's start. Holds alone do not trigger the full summary, but a stale hold becomes Debt and makes it visible again.

## Finish or pause

Close completed work with proof and archive it. [Tasks](tasks.md#closing-with-a-proof) explains the proof options. If you need to pause, use `anb hold <id> --reason "<why>"`; the Task leaves the active display and ready queue until you unhold it.

Before ending a session, run `anb check` and resolve its findings. If the notebook is versioned, commit it with the work it describes. The next session can then read both the result and the reason for it.
