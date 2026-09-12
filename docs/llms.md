---
title: llms.txt and AGENTS.md
description: 'Plain Markdown exports of the book and instructions for agents using or developing anb.'
---

## Read the book as Markdown

The published site provides exports following the [llms.txt](https://llmstxt.org/) proposal:

| File | Contents |
|---|---|
| `/llms.txt` | An introduction and links to the documentation |
| `/llms-small.txt` | An abridged copy of each page |
| `/llms-full.txt` | The complete book |

Use the index to choose a page, or the full export when the task needs the whole book.

## Use a notebook in a project

`anb setup --agent <host>` installs the workflow skill and references alongside the agent instructions. Choose `claude-code`, `codex` or `agents-md`. Start with `anb recall` for current work, the conversation's remembered Task and relevant knowledge. Read the selected Task before continuing; several active Tasks do not imply that the first one is yours.

Use `anb --help` or `anb <verb> --help` to look up commands. Default replies use standard TOON, and `--json` provides the same structured result. The installed skill includes the working method, examples and recovery guidance without requiring this site. Team workflow extensions belong in `.agents/anb.md`; personal practices can stay in a personal notebook.

Reading needs no installation or special skill. Notebook records are ordinary Markdown with typed headers, meaningful titles, short bodies and links to related records or external sources. An agent that only reads repository files can still use the knowledge. See [Agent integrations](guides/agents.md) for setup and [Personal memory](guides/your-own-notebook.md) for audience boundaries.

## Change this repository

Read `AGENTS.md` at the repository root; `CLAUDE.md` links to it. The project's working history is in `.agent-notebook/`, and [Development](contributing/development.md) describes the workspace and checks.
