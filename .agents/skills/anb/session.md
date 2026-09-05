---
name: anb worked session
description: One notebook worked from empty to archived work, every reply as the tool printed it. Open to see what a reply looks like before you parse one.
metadata:
  generated: anb
---

# anb worked session

Generated from the binary — the same definitions `anb --help` prints, every example run on a scratch notebook; a committed copy is checked against this rendering in CI.

Every reply below is what the tool printed, run on 2026-01-15 by an agent whose git identity is `Ada`.

An idea becomes a hub, and the work inside it is born from the hub:

```
$ anb add task "Ship the parser" --tag epic --body "The parser reads every record file byte for byte."
ok: add task.ship-the-parser — tasks/task.ship-the-parser.md
```

```
$ anb add task "Grammar parser accepts fences" --from task.ship-the-parser --priority 1
ok: add task.grammar-parser-accepts-fences — tasks/task.grammar-parser-accepts-fences.md
```

```
$ anb add task "Negative corpus wired into CI" --from task.ship-the-parser --priority 2
ok: add task.negative-corpus-wired-into-ci — tasks/task.negative-corpus-wired-into-ci.md
```

```
$ anb block task.ship-the-parser task.grammar-parser-accepts-fences
ok: block task.ship-the-parser — waits on task.grammar-parser-accepts-fences
```

```
$ anb block task.ship-the-parser task.negative-corpus-wired-into-ci
ok: block task.ship-the-parser — waits on task.negative-corpus-wired-into-ci
```

```
$ anb block task.negative-corpus-wired-into-ci task.grammar-parser-accepts-fences
ok: block task.negative-corpus-wired-into-ci — waits on task.grammar-parser-accepts-fences
```

The queue shows what can start now; the blocked child waits:

```
$ anb ready
ready[1]{id,priority,age,title}:
  task.grammar-parser-accepts-fences,1,0d,Grammar parser accepts fences
```

```
$ anb ready --for task.ship-the-parser
ready[1]{id,priority,age,title}:
  task.grammar-parser-accepts-fences,1,0d,Grammar parser accepts fences
```

A session takes the top of the queue, logs as it goes, and parks a doubt without widening its scope:

```
$ anb start task.grammar-parser-accepts-fences
ok: start task.grammar-parser-accepts-fences — open→active
```

```
$ anb comment task.grammar-parser-accepts-fences "fences parse; the indented-body case is next"
ok: comment task.grammar-parser-accepts-fences — logged
```

```
$ anb add question "Do fences nest?" --from task.grammar-parser-accepts-fences
ok: add question.do-fences-nest — questions/question.do-fences-nest.md
```

A ruling is a Decision; a term is a Note. A second Decision on the same ground is nudged about the first — read it before going on:

```
$ anb add decision "Fences never nest" --kind rule --tag parser --tag grammar --body "A fence closes at the first closing marker. Answers question.do-fences-nest."
ok: add decision.fences-never-nest — decisions/decision.fences-never-nest.md
```

```
$ anb add decision "A fence body is opaque" --kind rule --tag parser --tag grammar --body "Nothing inside a fence is parsed."
ok: add decision.a-fence-body-is-opaque — decisions/decision.a-fence-body-is-opaque.md
may-conflict[1]: decision.fences-never-nest (Ada)
```

```
$ anb add note Fence --kind term --body "A fence is a pair of triple-backtick lines; the parser treats the lines between as one opaque body."
ok: add note.fence — notes/note.fence.md
```

The doubt closes into the record that settled it, and is archived right after:

```
$ anb close question.do-fences-nest --resolved-by decision.fences-never-nest
ok: close question.do-fences-nest — open→closed
resolved-by: decision.fences-never-nest
```

```
$ anb archive question.do-fences-nest
ok: archive question.do-fences-nest — questions/question.do-fences-nest.md→archive/questions/question.do-fences-nest.md
```

Status is the session's opening — the active Task with its last log line, the rules, the queue, the epics:

```
$ anb status --budget 0
ok: notebook — 3 tasks, 2 decisions, 1 notes, 0 questions
active: task.grammar-parser-accepts-fences "Grammar parser accepts fences"
log: "- 2026-01-15 Ada: fences parse; the indented-body case is next"
rules[2]:
  decision.a-fence-body-is-opaque: "A fence body is opaque"
  decision.fences-never-nest: "Fences never nest"
epics[1]:
  task.ship-the-parser: 0/2 closed — nothing ready
budget: ~122 tokens (no ceiling)
```

The work closes with its report as a Note, and is archived right after; the reply names what the close unblocked:

```
$ anb close task.grammar-parser-accepts-fences --note report.md
ok: close task.grammar-parser-accepts-fences — active→closed
report: note.report-grammar-parser-accepts-fences
unblocked[1]: task.negative-corpus-wired-into-ci
```

```
$ anb archive task.grammar-parser-accepts-fences
ok: archive task.grammar-parser-accepts-fences — tasks/task.grammar-parser-accepts-fences.md→archive/tasks/task.grammar-parser-accepts-fences.md
carried[1]: note.report-grammar-parser-accepts-fences
```

A Task overtaken before it started ends with its reason, from open, and is archived like any closed record; a pause carries its reason too:

```
$ anb add task "Port the parser to Go"
ok: add task.port-the-parser-to-go — tasks/task.port-the-parser-to-go.md
```

```
$ anb close task.port-the-parser-to-go --reason "overtaken by decision.a-fence-body-is-opaque"
ok: close task.port-the-parser-to-go — open→closed
```

```
$ anb archive task.port-the-parser-to-go
ok: archive task.port-the-parser-to-go — tasks/task.port-the-parser-to-go.md→archive/tasks/task.port-the-parser-to-go.md
```

```
$ anb hold task.negative-corpus-wired-into-ci --reason "waits for the CI runner" --until 2026-01-20
ok: hold task.negative-corpus-wired-into-ci — held until 2026-01-20
```

Reading back: one record, the whole notebook, a search that reaches the archive, and the gate — clean, because every settled record was archived as it settled:

```
$ anb show task.ship-the-parser
id: task.ship-the-parser
type: task
state: open
title: Ship the parser
by: Ada
tags: epic
blocked-by: task.grammar-parser-accepts-fences
blocked-by: task.negative-corpus-wired-into-ci
created: 2026-01-15
updated: 2026-01-15
body: |

  The parser reads every record file byte for byte.
```

```
$ anb list
records[5]{id,state,priority,title}:
  task.negative-corpus-wired-into-ci,open,2,Negative corpus wired into CI
  task.ship-the-parser,open,-,Ship the parser
  decision.a-fence-body-is-opaque,active,-,A fence body is opaque
  decision.fences-never-nest,active,-,Fences never nest
  note.fence,active,-,Fence
```

```
$ anb search fence
matches[6]{id,state,priority,title}:
  task.grammar-parser-accepts-fences,closed,1,Grammar parser accepts fences
  decision.a-fence-body-is-opaque,active,-,A fence body is opaque
  decision.fences-never-nest,active,-,Fences never nest
  note.fence,active,-,Fence
  note.report-grammar-parser-accepts-fences,retired,-,Report: Grammar parser accepts fences
  question.do-fences-nest,closed,-,Do fences nest?
```

```
$ anb check
count: 0
```

