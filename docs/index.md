---
title: Working memory for coding agents
description: 'Working memory for coding agents, with a deterministic CLI and customizable workflow skills.'
template: splash
hero:
  title: agent-notebook
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

The CLI handles record changes and checks that their states and relationships are valid. The supplied skills describe the working method: how to resume work, keep useful knowledge and record an outcome on the Task. Shared records live in `.agent-notebook/` at the repository root, ready to review with the code. Personal practices stay outside the repository.

Setup connects the supplied method to your agent. Keep project-specific instructions in `.agents/anb.md`; upgrades preserve that file, so there is no skill to fork. Each session can continue its own Task while people share definitions, constraints and decisions. The [customization guide](guides/customization.md) explains the available controls.

At the start of a session, the agent recalls its work, relevant project knowledge and personal practices:

```sh
anb recall
anb start
```

The notebook stores Tasks, Decisions, Notes and Questions in plain Markdown files. `recall` provides a bounded starting context; `status` shows the work queue. Both expose what was omitted and how to read more. Other agents and people can read the files without installing anything. Keep tickets and full documentation in their existing systems, and link to them from the concise context that helps the next session act.

agent-notebook uses the same tool and skills for its own development. Its notebook is public with the source; the [development guide](contributing/development.md) explains where to find the work and the decisions behind it.

[Get started](start/quickstart.md) with a complete session, read [the design](start/what-it-is.md), or ask your agent to [draw the work as a map](guides/atlas.md). The [command reference](reference/commands.md) is generated from the binary.
