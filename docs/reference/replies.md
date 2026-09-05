---
title: Replies
description: 'The reply contract every command keeps: ok lines, tables, bounds, JSON, refusals and the try lines that run as printed.'
---

Every reply is written to be parsed at a glance by an agent and read without effort by a person. The shapes are few and every command uses them.

## The ok line

A mutation answers with one line: `ok: <verb> <id> — <what changed>`. A state move says the transition; a creation says the file; an idempotent repeat says so and changes nothing.

```
ok: start task.parser-accepts-fenced-bodies — open→active
ok: add task.ship-the-parser — tasks/task.ship-the-parser.md
ok: archive task.ship-the-parser — archived (already)
```

Consequences follow on their own lines, each a named list: `unblocked[N]`, `carried[N]`, `may-conflict[N]`, `dangling-mention[N]`, `resolved-by:`, `report:`.

## Tables

A listing names its columns once and prints comma rows:

```
ready[1]{id,priority,age,title}:
  task.negative-corpus-wired-into-ci,-,0d,Negative corpus wired into CI
```

`-` is an absent value. A value holding a comma or a quote is quoted. An empty listing is `count: 0`.

## Bounds

Every listing is bounded by default, so a large notebook never floods a session; the bound is stated and `--all` lifts it. A long body or a crowded mention block in `show` is cut the same way, with the cut marked inline. The JSON of `graph` is the one exception: it is never bounded, because a graph missing edges is not a smaller graph but a wrong one.

## JSON

`--json` on any command gives the same data as compact JSON, keys in the order the text prints them, `ok` first. Lists are `{count, rows}`; a cut body is `{lines, head, tail}`; a refusal is `{error, message, try}`, with `findings` beside them when a record's own findings caused it.

## Refusals

A refusal is `error[<code>]: <message>` followed by `try:` lines, each a command that runs as printed:

```
error[unknown-id]: no record `task.nope`
try: anb list
```

The codes are stable and the message names the record and the fact. A refusal changes nothing: every file is judged before the first is written. The [refusals reference](refusals.md) has every code with an example.

## Exit codes

`0` for a reply, `1` for a refusal or a `check` with an error finding, `2` for a command line clap cannot parse. `status --hook` exits `0` whatever it finds.
