---
title: What it is
description: 'Why the notebook separates durable knowledge, deterministic record rules and a working method you can change.'
---

I want to be able to return to a project and understand why it is built the way it is. When an agent investigates a problem, that investigation should help with later work too. Saving the conversation is not enough: some decisions will have changed, some questions will still be open, and someone has to keep track of which is which.

That is the job I built agent-notebook for. Records distinguish work, decisions, knowledge and questions. The CLI checks their states and relationships in code, without a model call. The supplied skills teach the agent when to record something, how to keep it current and what to leave for the next session.

## A default method you can change

The supplied skill describes how I work: one active Task, a log of useful progress, findings recorded as they come up, and a report when the work is finished. Keeping one Task active makes it clear where the next session should resume.

I do not expect that to suit every team. Several agents may need to work on separate Tasks, or a reviewer may need to accept a result before it closes. You can put those instructions in the skill. The CLI allows several active Tasks and an optional review stage; it still checks each record's state transitions and dependencies.

Storage is a separate choice too. By default, the first record write creates `.agent-notebook/` at the repository root. The supplied skill commits it with the code so collaborators inherit the same context. You can instead ignore it to keep working notes private, or select another location. Git integration, the global notebook and maps extend the ways you can use the records; the CLI does not require them.

A new request begins as an [idea](../guides/ideas.md), with its source and the intended improvement. The skill teaches the agent how to investigate it and when it is ready for delivery planning. A [domain model](../guides/domain.md) can describe responsibilities and invariants that a glossary alone cannot.

## What goes in the notebook

| Record | Purpose | Lifecycle |
|---|---|---|
| Task | Work, dependencies and a progress log | open → active → review → closed |
| Decision | A ruling and its reason | active → superseded or retired |
| Note | Knowledge kept current in place | active → retired |
| Question | An uncertainty with an origin and an eventual answer | open → closed |

Records have readable ids such as `task.parser-accepts-fenced-bodies`. Use those ids to connect the work to its context: create a Question `--from` a Task, cite a Decision in a log entry, or close a Task with its report. The notebook derives a graph from these relationships. Finished records move to an archive that remains searchable.

## Why it works this way

### Records are files

Each record has a small envelope of typed fields followed by a Markdown body. The CLI reads the files directly, without a database, index or daemon. You can inspect the notebook with ordinary file tools and move it without exporting from a service.

The envelope has a small line grammar: each key determines its value type, and the Markdown body stays intact. Changing state should not reformat an investigation or discard a line the parser does not understand. `anb check` reports invalid files with their locations and repair commands. A malformed record remains visible as a finding. The [format reference](../reference/records.md) specifies the envelope and preservation rules.

### Changes go through the CLI

Agents use commands to change records. The CLI rejects invalid state transitions and dependency cycles. Closing work requires evidence, an explicit waiver, or a reason the work will not happen. The default method imports a report as a Note, so moving or sharing the notebook also preserves the evidence. Replacing a Decision updates its predecessor too; the next reader can tell which ruling applies without interpreting competing prose.

These checks apply regardless of which agent runs the command. They verify the recorded outcome; they cannot judge whether a report proves the work or whether a design choice is sound. That judgment stays with the people and agents doing the work.

### The session starts with a summary

`anb status` summarizes the work: active Tasks, ready Tasks, open Questions and a count of neglected work. Its configurable budget limits how much context the summary consumes. It reports omissions so the agent can request more. A quiet notebook gets one line. Keeping history should not require putting all of it into every session. `anb show` opens one record, and one set of narrowing flags follows one branch of work, one type or kind of record, one tag, or one person's records on every listing.

Replies use short text tables and stable refusal codes, with `--json` for programs. The [reply contract](../reference/replies.md) describes the shapes; [Status and Debt](../reference/status.md) explains the budget and its limits.

## Developing with our own notebook

I use the same CLI and skills for this project. Moving the backlog into the notebook exposed a practical problem: reports linked into an uncommitted directory could not be read from another clone. Importing reports with `close --note` became the default method, while file paths, commits and pull requests remained valid proof choices.

I intend to keep developing it this way. When using the tool exposes a problem, I record it in the notebook and use that case to check the fix. But my own workflow will only expose some of the problems. I would like to hear where other people need different behavior, including changes they have made to the skills. The [development guide](../contributing/development.md) explains the code and links to the public notebook.

## Start with the supplied workflow

`anb setup` installs the skills and agent integrations. The [quickstart](quickstart.md) demonstrates their workflow through the CLI. [Customizing the workflow](../guides/customization.md) shows how to change reviews, skill instructions and notebook location.
