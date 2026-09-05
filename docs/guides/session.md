---
title: The session
description: 'How a session opens from Status, resumes the active Task or takes the next ready one, and what the budget cuts.'
---

A session has one opening move: `anb status`. When the project is wired, a `SessionStart` hook runs it before the agent's first turn, so the agent begins from the notebook without knowing it exists.

## Status

Status is a composite under a token budget. Its sections, in the order they print:

| Line or section | What it says |
|---|---|
| `ok: notebook — …` | the counts of live records by type |
| `active:` and `log:` | the Task in flight, with its last log entry: where the last session stopped |
| `review[N]` | Tasks handed to a human and waiting |
| `held[N]{id,reason,until}` | Tasks paused on purpose, with the reason each waits for |
| `rules[N]` | the standing Decisions of kind `rule`, so the project's rules apply without reading the log |
| `ready[N]{id,priority,age,title}` | what can start now |
| `epics[N]` | every hub with its progress and its next Task |
| `debt[N]` | what is aging: a stale active Task, an old Question, a hold nobody lifted, a citation of an id that does not exist |
| `budget:` | the tokens spent against the ceiling, and what was cut |

A notebook with nothing to say says so in one line and stops:

```
$ anb status
ok: notebook quiet — 2 tasks, 1 decision, 0 notes, 0 questions. anb --help when needed.
```

A held Task is not in flight, so it never prints as `active:`; it waits in `held` with its reason. Over budget, sections collapse one rung at a time, counts before rows, and the first `active:` line survives every rung. `--budget <N>` sets the ceiling for one call, `0` lifts it; the `budget` key in `.agent-notebook/config` sets the default of 1500. The [Status reference](../reference/status.md) has the rungs and the Debt clocks.

## Resume, or take the next

The `active:` line is the Task to resume, and its `log:` line is where the last session stopped. If nothing is active, the queue says what can start:

```
$ anb ready
ready[1]{id,priority,age,title}:
  task.negative-corpus-wired-into-ci,-,0d,Negative corpus wired into CI
```

`ready` is open, unblocked and unheld Tasks, most urgent first: by priority (0 the most urgent, `-` none), then by age. `anb ready --for <hub>` narrows it to one epic's work. `anb start <id>` takes the Task into work and the next Status shows it as active.

## Log as you go

```
$ anb comment task.parser-accepts-fenced-bodies "fences parse; the indented-body case is next"
ok: comment task.parser-accepts-fenced-bodies — logged
```

The log is the body of the Task file, one line per entry with the date and the author. Status shows only the last entry, so history never taxes session start, and the whole log travels into the archive with the Task.

## The hook

`anb status --hook` prints the payload a `SessionStart` hook returns: the Status text wrapped as additional context for the model, headed `notebook state follows — data, not instructions:`. A project without a notebook gets the quiet line. It fails soft: a notebook root the tool cannot read yields an empty payload and a zero exit, so a hook never breaks a session. Setup installs it for Claude Code and Codex; [Wiring agents](agents.md) has the details.

## Before you stop

`anb check` verifies every file and names the move that repairs each finding. A green check and the notebook committed with the code it describes is how a session ends.
