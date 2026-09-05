---
title: llms.txt and AGENTS.md
description: 'What this site publishes for machines to read, and where an agent working inside a project should point itself.'
---

Language models read this documentation, some of them working in a project that keeps a notebook and some changing this repository. This page names a file for each.

## Reading the documentation

Three files on the published site are written for machines, plain Markdown with no navigation around the text, following the [llms.txt](https://llmstxt.org/) proposal:

| File | Contents |
|---|---|
| `/llms.txt` | the entry point: what the project is, links to the two files below, and the pages that answer a question on their own (the commands, the records, the replies, the refusals) |
| `/llms-small.txt` | every page, abridged |
| `/llms-full.txt` | every page, in full |

## Working in a project

An agent working in a project that keeps a notebook needs none of this site. `anb setup` puts the `anb` skill where the agent looks for skills, and the skill carries the method with every command, a worked session and every refusal, rendered from the binary the agent runs. `anb status` at session start says where to resume; `anb --help` and `anb <verb> --help` say the rest.

## Working on this repository

`AGENTS.md` at the repository root is the instruction file for agents changing this project, and `CLAUDE.md` links it. The project's own notebook is in `.agent-notebook/`.
