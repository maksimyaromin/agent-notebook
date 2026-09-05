---
title: agent notebooks
description: 'A project’s working memory as typed records in plain files, read and written by any coding agent through one CLI.'
template: splash
hero:
  tagline: An agent loses everything between sessions that does not land in code. A notebook keeps it in the repository, as records with lifecycles, and one command reads or moves any of them.
  actions:
    - text: Start here
      link: start/what-it-is/
      icon: right-arrow
    - text: Commands
      link: reference/commands/
      variant: minimal
---

`anb` keeps a project's working memory in `.agent-notebook/`: Tasks, Decisions, Notes and Questions, one markdown file each, committed with the code. An agent runs commands and reads short replies; it never edits the files. A session opens with `anb status`, takes the next Task from `anb ready`, records what it learns and decides, closes work with its proof and archives it. What is left is a notebook a person can read in any editor, and a project that remembers.

```
$ anb status
ok: notebook — 1 tasks, 1 decisions, 0 notes, 0 questions
active: task.parser-accepts-fenced-bodies "Parser accepts fenced bodies"
log: "- 2026-09-05 Alex: fences parse; the indented-body case is next"
rules[1]:
  decision.fences-never-nest: "Fences never nest"
budget: ~83/1500 tokens
```

The [quickstart](start/quickstart.md) runs one session end to end. The [record model](reference/records.md) is what the tool enforces; the [commands](reference/commands.md) are rendered from the binary itself.
