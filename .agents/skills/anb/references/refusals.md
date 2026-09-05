---
name: anb refusals
description: Every refusal code with its cause and the try: line that repairs it, and the findings anb check raises. Open when a refusal's try: line is not enough.
metadata:
  managed-by: anb
---

# anb refusals

Generated from the binary: the same definitions `anb --help` prints, and every example run on a scratch notebook. A committed copy is checked against this rendering in CI.

## Contents

- The scratch notebook
- unknown-id
- invalid-transition
- invalid-argument
- dangling-ref
- would-cycle
- duplicate-id
- wrong-type
- still-referenced
- cannot-supersede
- archived
- invalid-record
- Codes without an example
- Check findings

Every refusal is `error[<code>]: <message>` followed by `try:` lines, which are commands that run as printed. The codes are stable; the messages name the record and the fact.

## The scratch notebook

The scratch notebook the refusals below run against:

```
$ anb add task "Ship the parser" --tag epic
ok: add task.ship-the-parser — tasks/task.ship-the-parser.md
```

```
$ anb add task "Grammar parser accepts fences" --from task.ship-the-parser
ok: add task.grammar-parser-accepts-fences — tasks/task.grammar-parser-accepts-fences.md
```

```
$ anb add decision "Fences never nest" --kind rule
ok: add decision.fences-never-nest — decisions/decision.fences-never-nest.md
```

```
$ anb add note Fence --kind term
ok: add note.fence — notes/note.fence.md
```

## unknown-id

No record carries the id.

```
$ anb start task.parser
error[unknown-id]: no record `task.parser`
try: anb list
```

## invalid-transition

The record's state does not allow the move; the valid moves are listed, each with its command.

```
$ anb close task.ship-the-parser --no-proof
error[invalid-transition]: `task.ship-the-parser` is open; valid: start, close --reason
try: anb start task.ship-the-parser
try: anb close task.ship-the-parser --reason "<why>"
```

## invalid-argument

A flag or value is malformed or foreign to the record; the retry shape is given.

```
$ anb add decision Tabs --priority 2
error[invalid-argument]: priority: applies only to a task
try: anb add decision "<title>"
```

```
$ anb close task.ship-the-parser
error[invalid-argument]: close: pass one of --note <path>, --pr <url>, --sha <sha>, --report <path>, --no-proof, --reason "<why>", or --resolved-by <id>
try: anb close task.ship-the-parser --note <path>
try: anb close task.ship-the-parser --no-proof
try: anb close task.ship-the-parser --reason "<why>"
```

## dangling-ref

An envelope reference names a record that does not exist.

```
$ anb add task "A child" --from task.ghost
error[dangling-ref]: from: `task.ghost` names no record
try: anb add task "<title>" --id task.ghost
try: anb list
```

## would-cycle

The edge would close a dependency cycle, which is walked in full.

```
$ anb block task.ship-the-parser task.grammar-parser-accepts-fences
ok: block task.ship-the-parser — waits on task.grammar-parser-accepts-fences
```

```
$ anb block task.grammar-parser-accepts-fences task.ship-the-parser
error[would-cycle]: the edge would close a dependency cycle: task.grammar-parser-accepts-fences → task.ship-the-parser → task.grammar-parser-accepts-fences
try: anb unblock task.ship-the-parser task.grammar-parser-accepts-fences
```

## duplicate-id

Ids are never reused, the archive included.

```
$ anb add task "Ship the parser again" --id task.ship-the-parser
error[duplicate-id]: `task.ship-the-parser` already exists at tasks/task.ship-the-parser.md
try: anb show task.ship-the-parser
try: anb add task "<title>"
```

## wrong-type

The id names a type the command does not act on.

```
$ anb comment note.fence "a line"
error[wrong-type]: `note.fence` is not a task
try: anb show note.fence
```

## still-referenced

A delete would leave the notebook pointing at nothing; every holder is named.

```
$ anb delete task.ship-the-parser
error[still-referenced]: `task.ship-the-parser` is still referenced by 1 record
  task.grammar-parser-accepts-fences — from
try: anb show task.grammar-parser-accepts-fences
```

## cannot-supersede

The record named by `--supersedes` cannot die by supersession.

```
$ anb retire decision.fences-never-nest
ok: retire decision.fences-never-nest — active→retired
```

```
$ anb add decision "Fences nest once" --kind rule --supersedes decision.fences-never-nest
error[cannot-supersede]: cannot supersede `decision.fences-never-nest`: its state is `retired`, not active
```

## archived

An archived record is read, never mutated in place; `restore` brings it back.

```
$ anb archive decision.fences-never-nest
ok: archive decision.fences-never-nest — decisions/decision.fences-never-nest.md→archive/decisions/decision.fences-never-nest.md
```

```
$ anb edit decision.fences-never-nest --title Fences
error[archived]: `decision.fences-never-nest` is archived
try: anb show decision.fences-never-nest
try: anb restore decision.fences-never-nest
```

## invalid-record

The file carries error findings, which close it to every verb until `check` is answered.

```
$ anb start task.broken
error[invalid-record]: tasks/task.broken.md is invalid (1 findings)
  line 4: bad-value state: `bogus` is not one of open, active, review, closed for a task
try: anb show task.broken
```

## Codes without an example

Three more codes reach the command line without a notebook to show them on: `unknown-command` (a verb anb does not have; the reply offers `anb --help`), `storage` (the file system failed the read or write, in the message) and `not-utf8` (a record file is not UTF-8; `anb check` names it).

## Check findings

`anb check` prints `findings[N]{file,line,severity,code,repair,message}`; an `error` fails the command, a `warning` does not. The `repair` column carries the command that erases the finding when the tool has one.

| Severity | Codes |
|---|---|
| error | `no-envelope`, `unclosed-envelope`, `bad-envelope-line`, `duplicate-field`, `missing-field`, `bad-value`, `bad-date`, `bad-id`, `id-filename-mismatch`, `type-dir-mismatch`, `archived-live-record`, `dangling-ref`, `block-cycle`, `origin-cycle`, `duplicate-id`, `broken-supersession`, `not-utf8` |
| warning | `unknown-field`, `unarchived-settled-record`, `orphan-field`, `crlf`, `bom`, `no-final-newline` |

