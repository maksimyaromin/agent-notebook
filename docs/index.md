---
title: agent-notebook
description: 'Working memory for coding agents, with a deterministic CLI and customizable workflow skills.'
template: splash
hero:
  tagline: Working memory for projects built with coding agents.
  actions:
    - text: Get started
      link: start/quickstart/
      icon: right-arrow
    - text: Commands
      link: reference/commands/
      variant: minimal
---

A coding session leaves more than code behind. There are decisions to explain, findings worth keeping and unfinished work to return to. agent-notebook gives agents a way to record these as they work and find them again in a later session.

The CLI handles record changes and checks that their states and relationships are valid. The supplied skills describe the working method: how to resume a Task, record what was learned and close the work with a report. By default, records live in `.agent-notebook/` at the repository root and the agent commits them with the code.

The skills can be changed independently of the CLI. For example, you can keep memory somewhere else, require review before closing a Task, or give each agent its own work. The [customization guide](guides/customization.md) shows how to set this up.

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

agent-notebook uses the same tool and skills for its own development. Its notebook is public with the source; the [development guide](contributing/development.md) explains where to find the work and the decisions behind it.

[Get started](start/quickstart.md) with a complete session, read [the design](start/what-it-is.md), or ask your agent to [draw the work as a map](guides/atlas.md). The [command reference](reference/commands.md) is generated from the binary.
