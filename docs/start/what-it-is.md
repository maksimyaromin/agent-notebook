---
title: What it is
description: 'The problem a notebook solves, the four kinds of record, and the rules the tool holds so that nobody has to.'
---

A coding agent starts every session from nothing. The code it wrote is there, the tests are there, but which task was in flight and where it stopped, the decision that overruled the plan last week, the fact about the deployment it learned the hard way, the doubt it put aside: all of that lived in a conversation that is gone. People solve this with a file. A `TODO.md`, a `NOTES.md`, a `.tmp/state/` directory. The file grows until reading it costs more than it saves, nobody knows which lines are still true, and an agent that edits it by hand corrupts it sooner or later.

agent notebooks is that file made durable: a **notebook** in the repository, `.agent-notebook/`, holding typed **records** with lifecycles, and a CLI named `anb` that is the only thing that changes them.

## Four kinds of record

| Record | What it holds | Lifecycle |
|---|---|---|
| Task | a piece of work with a log | open → active → review → closed |
| Decision | a ruling that stands until replaced | active → superseded or retired |
| Note | knowledge kept current in place | active → retired |
| Question | a doubt with an origin, waiting for its answer | open → closed |

Every record is one markdown file: an envelope of `key: value` lines the tool owns, then a body it never parses. Every record has an id an agent can type, `task.parser-accepts-fenced-bodies`, minted from its title. A closed or retired record moves into `archive/`, where it keeps its bytes and its id; nothing is deleted by a lifecycle move. The [records reference](../reference/records.md) has every key and state.

## Three rules

**The files are the truth.** There is no index, no database, no daemon. What is in `.agent-notebook/` is the notebook, and it travels with git like any other file. You can open a record in your editor and fix a title; `anb check` reads every file and names each line it cannot accept, so a hand edit never silently drops a record.

**One tool changes them.** Agents never edit record files. Every change is a command, and a command refuses what would break the notebook: a Task cannot close without a proof or an explicit waiver, a Question cannot close without saying what settled it or why it is moot, a dependency edge that would form a cycle is refused when written. The rules live in the tool, so they bind every agent and every person equally, and nobody has to police them.

**Every reply is cheap to read.** An agent parses the reply at a glance: `ok: <verb> <id> — <what changed>`, tables that name their columns once, refusals as `error[<code>]: <message>` followed by `try:` lines that run as printed. Listings are bounded; `--all` lifts the bound and `--json` gives the same data as JSON. Status fits a token budget and says what it cut. The [replies reference](../reference/replies.md) is the whole contract.

## What the tool does not decide

The tool keeps a record's invariants. It does not run your team's workflow. Whether one Task is active at a time, whether a closed Task is archived at once, how a report is written: those are practices, and they belong to the skill an agent loads, not to the CLI. The `anb` skill the binary renders teaches one opinionated process; a team that works differently writes its own and drives the same commands. [Wiring agents](../guides/agents.md) shows where the skill comes from and how it is installed.
