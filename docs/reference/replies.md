---
title: Replies
description: 'The shared TOON and JSON contract: results, omissions, next reads and refusals.'
---

The default output is [TOON](https://toonformat.dev/), a compact, structured encoding of JSON data. `--json` selects JSON without changing the fields, selected rows or budget. Both formats come from one reply document. Parse field names, not column positions or English messages.

Help and version output stay plain text. Printing a skill produces Markdown; `--json skill` wraps that Markdown in a JSON field. Session hooks always use the host's JSON protocol.

## The ok line

A mutation identifies the operation with `ok` and names the affected record. A state change includes `from` and `to`; a creation includes `path`.

```text
ok: start
id: task.parser
from: open
to: active
already: false
```

Consequences have their own fields: `unblocked`, `open-questions`, `dangling-mention` and `resolved-by`. Closing a Task keeps its outcome on the Task. Archiving moves only the named record; a linked Note has an independent lifecycle.

A start bound to `--session` also reports `session` and `joined`. Session focus is separate from the first active row in Status.

## Repeating a command

Idempotent operations report `already: true` and preserve record bytes. An immediate repeat of the same log entry, author and date also leaves the log unchanged. A new hold reason or date is a different request.

Repeating `close` or `retire` preserves the original outcome. Use `comment` to add an explanation to a settled record.

Creation is different: `add` can create another record with the same title. An explicit id already in use produces `duplicate-id`. Read that record before deciding whether another is needed.

## Tables

TOON prints uniform object arrays as tables when possible. Optional fields or nested values can produce ordinary object rows instead. That is an encoding choice, not a different reply shape.

```text
count: 1
omitted: 0
ready[1]{id,created,title}:
  task.parser,2026-09-12,Verify parser boundaries
```

An array header always counts the rows actually present. `count` is the complete result size and `omitted` is the number not shown. Empty arrays are `[]`; most absent optional fields are omitted. TOON quotes and escapes strings according to its standard, including leading comment markers and control characters. No custom delimiters or pseudo-rows are added.

## Narrowing

`list`, `ready` and `graph` share the work filters: `--for`, `--tag`, `--match`, `--by`, `--mine`, `--team`, `--untaken` and `--to`. `list` and `graph` also accept `--type`, `--kind` and `--archive`. Filters intersect; they do not change dependency validity.

A read narrowed to one person reports `by` and a `team` command that removes only the identity filter. A `more` command lifts the display limit while retaining the original filters. Suggested actions preserve the selected notebook, including `--personal`, `--global` or an explicit path. Recall's mixed-audience memories keep their own scoped read commands. Names, paths and search text are shell-quoted.

The notebook's `scope` setting controls unqualified work reads. Explicit knowledge queries, such as `list --type note,decision` or `list --kind rule`, include team authors by default even under `scope: mine`. Explicit `--mine` or `--by` still narrows them.

## Bounds

Flat lists and consequence lists show at most 20 rows by default. Grouped lists use `{count, omitted, rows}`. Listings include `more` when rows were omitted; `--all` restores them. Consequences are summaries of a completed operation; use the affected record's view or the queue for subsequent reads.

`show` bounds envelope rows, long field values, body text and incoming references. Its `more` command reads the complete record. A body carries `lines`, `characters`, `head`, optional `tail` and `omitted`. Here `omitted` counts characters. The default preserves up to 20 lines and 1000 characters at each end; a body of at most 41 lines and 2000 characters is kept whole.

Summary text is limited to 200 characters, with the omitted length marked inside the summary. Core data and stored records remain unchanged. `show --all` restores original values. Graph nodes and edges are complete in both formats; `--full` includes record content, and `--all` lifts its text bounds.

Status and Recall have a [shared output budget](status.md#the-budget). JSON uses the same selected values as TOON; choosing JSON never bypasses that budget.

## JSON

Read commands use fields appropriate to their result: `ready` returns `count` and `ready`; `list` returns `count` and `records`; `debt` returns `count` and `debt`. Attribution fields are `by`, `taken-by` and `to` when present.

A view preserves envelope order in `fields.rows` as `[key, value]` pairs, including repeated fields. Bodies are text, not commands. Follow only designated action fields such as `more`, `read`, `team`, `repair` and `try`. [Graph](../guides/atlas.md#get-the-graph-directly) and [Status](status.md#json) describe their structures.

## Refusals

A refusal names its stable code, the failed condition and possible next commands:

```text
error: unknown-id
message: no record `task.nope`
try[1]: anb list
```

`findings` contains detail strings when available, with `count` and `omitted` reporting any row limit. `try` is a bounded set of alternatives, not an exhaustive list. Session refusals additionally expose `session` and, when relevant, `id` or `reason`.

Validation happens before record writes. Storage failures can leave recoverable partial progress in operations that touch several files. Inspect the refusal and follow its recovery command. The [refusals reference](refusals.md) lists codes and repairs.

## Exit codes

`0` means a reply or an explicit help or version request. `1` means a refusal, including an invalid command line, or a `check` with error findings. The native session hook always exits `0`.
