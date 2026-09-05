---
title: agent-notebook
description: 'A dependable CLI for agent working memory, with skills you can make your own.'
template: splash
hero:
  tagline: Build on what your agents learn. Keep the memory, choose the method.
  actions:
    - text: Get started
      link: start/quickstart/
      icon: right-arrow
    - text: Commands
      link: reference/commands/
      variant: minimal
---

The effort you put into working with an agent should outlast the conversation. Its findings should be usable by the next session, and its decisions should remain clear when the work changes direction. You should be able to choose a different agent or a different process and keep that accumulated knowledge.

agent-notebook provides a deterministic CLI for that memory and skills for working with it. The CLI checks changes and returns concise, structured results. The skills describe the method, and you can rewrite them. A shared notebook committed with the project, private notes outside git, or a workflow of your own all use the same record rules.

At the start of a session, the agent reads a summary:

```
$ anb status
ok: notebook — 1 task, 1 decision, 0 notes, 0 questions
active: task.parser-accepts-fenced-bodies "Parser accepts fenced bodies"
log: "- 2026-09-05 Alex: fences parse; the indented-body case is next"
rules[1]:
  decision.fences-never-nest: "Fences never nest"
budget: ~82/1500 tokens
```

The notebook stores Tasks, Decisions, Notes and Questions in plain Markdown files. Status summarizes what needs attention; commands let an agent follow the detail and update it without editing files by hand. Sharing through git, personal knowledge across projects and interactive maps are available when your workflow needs them.

[Get started](start/quickstart.md) with a complete session, read [the design](start/what-it-is.md), or ask your agent to [draw the work as a map](guides/atlas.md). The [command reference](reference/commands.md) is generated from the binary.
