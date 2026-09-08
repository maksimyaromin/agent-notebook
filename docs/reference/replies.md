---
title: Replies
description: 'Success replies, retries, listing limits, JSON and refusal codes.'
---

The CLI uses short plain text replies for interactive work and compact JSON for programs. Named fields distinguish a result, its consequences and any suggested next action.

## The ok line

A mutation starts with `ok: <verb> <id> — <what changed>`. A state move says the transition; a creation says the file; an idempotent repeat says so and changes nothing.

```
ok: start task.parser-accepts-fenced-bodies — open→active
ok: add task.ship-the-parser — tasks/task.ship-the-parser.md
ok: archive task.ship-the-parser — archived (already)
```

Consequences follow on named lines: `unblocked[N]`, `carried[N]`, `may-conflict[N]`, `dangling-mention[N]`, `resolved-by:`, `report:`.

## Repeating a command

Commands that move an existing record report `(already)` when the requested change is already applied. They preserve the record bytes on that repeat. An immediate repeat of the same log entry, with the same author and date, also leaves the log unchanged.

Creation is different: `add` without `--id` can mint another id from the same title. An explicit id already in use produces `duplicate-id`. Read the named record before deciding whether to create another.

## Tables

A listing names its columns once and prints comma rows:

```
ready[1]{id,priority,age,taken-by,title}:
  task.negative-corpus-wired-into-ci,-,0d,-,Negative corpus wired into CI
```

`-` is an absent value. A value holding a comma or a quote is quoted. An empty listing is `count: 0`.

## Narrowing

`list`, `ready` and `graph` take the same narrowing flags: `--for <hub>`, `--tag`, `--match <text>`, `--by <name>`, `--mine`, `--team` and `--untaken`; `list` and `graph` also take `--type`, `--kind` and `--archive`, which a queue of live Tasks has no use for. Each is a predicate over the same notebook, so two flags ask for the intersection, and a narrowed listing's truncation hint carries every flag it was asked with. `--by` and `--mine` answer with one person's work, the Tasks they hold and the records they wrote; `--untaken` with the pool, the Tasks nobody holds. `status` takes `--by`, `--mine` and `--team`. Whose records a read answers with when the call names nobody is the notebook's `scope` config key, everyone's by default.

## Bounds

Listings have default row limits and report omissions. Use `--all` to lift them. `show` also bounds long bodies and mention lists, marking where content was omitted. Graph JSON is unbounded so consumers receive the complete selected graph.

## JSON

`--json` selects compact JSON. Mutation replies identify the operation with `ok`; read commands use fields appropriate to the result. For example, `ready` returns `count` and `ready`, `list` returns `count` and `records`, and `debt` returns `count` and `debt`.

A `list` or `ready` row carries `by` and `taken-by` when the record has them, so a script filters by identity without reading the files. Nested lists of consequences use `{count, rows}`. A truncated body uses `{lines, head, tail}`. Refusals provide `error`, `message`, `findings` and `try`. Parse these fields by name. [Graph](../guides/atlas.md#get-the-graph-directly) and [Status](status.md#json) describe their own result structures.

## Refusals

A refusal is `error[<code>]: <message>` followed by `try:` lines suggesting the next command:

```
error[unknown-id]: no record `task.nope`
try: anb list
```

Refusal codes are stable. Messages identify the failed condition, and `try:` lines suggest a command or an argument template to fill in. Validation happens before record writes. Storage failures during a multi-file operation can leave partial progress; inspect the reported findings and follow the recovery instructions. The [refusals reference](refusals.md) lists codes and repairs.

## Exit codes

`0` for a reply, `1` for a refusal or a `check` with an error finding, `2` for a command line clap cannot parse. `status --hook` exits `0` whatever it finds.
