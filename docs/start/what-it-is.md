---
title: What it is
description: 'Project memory in plain files, with explicit lifecycles and a small session-start summary.'
---

An agent learns more about a project as it works: which approach failed, why a constraint matters, where an investigation stopped. That knowledge is useful beyond the session that produced it. Keeping it usable means knowing what still applies, what was replaced, and what remains unresolved.

agent notebooks gives that memory a reliable interface. The `anb` CLI owns the record rules and reports the result of each operation. Skills describe how an agent uses those operations to work. This separation lets you change the process without asking the agent to reimplement the bookkeeping.

## The method is separate from the rules

The supplied skill is a complete workflow you can start using and then adapt. It teaches agents to resume an active Task, log progress, record findings, and close with proof before archiving. It prefers one Task in flight. The CLI permits several and leaves archiving to the caller.

You can rewrite that skill, supply your own, or drive the CLI directly. A team might add a review stage or use a different convention for decomposing work. Those choices change the instructions; state transitions, dependency checks and reply shapes remain the tool's responsibility.

Storage is a separate choice too. By default, the first record write creates `.agent-notebook/` at the repository root. Commit it to share the history, ignore it to keep working notes private, or select another location. Git integration, the global notebook and maps extend the ways you can use the records; the CLI does not require them.

## Record what changes the next step

| Record | Purpose | Lifecycle |
|---|---|---|
| Task | Work, dependencies and a progress log | open → active → review → closed |
| Decision | A ruling and its reason | active → superseded or retired |
| Note | Knowledge kept current in place | active → retired |
| Question | An uncertainty with an origin and an eventual answer | open → closed |

A Task log tells the next session where to resume. Decisions distinguish the rules that still apply from the ones you replaced. Questions keep an investigation visible until it has an answer. Notes collect facts and practices worth using again.

Records have readable ids such as `task.parser-accepts-fenced-bodies`. Use those ids to connect the work to its context: create a Question `--from` a Task, cite a Decision in a log entry, or close a Task with its report. The notebook derives a graph from these relationships. Finished records move to an archive that remains searchable.

## Three rules

### The files are the truth

Each record has a small envelope of typed fields followed by a Markdown body. The CLI reads the files directly, without a database, index or daemon. A clone can include both the code and the context behind it. Git is optional: you can also ignore the notebook or store it outside the repository.

Human readability does not require permissive parsing. `anb check` reports invalid files with their locations and repair commands. A malformed record remains visible as a finding. The [format reference](../reference/records.md) specifies the envelope and preservation rules.

### One tool changes them

Agents use commands to change records. The CLI rejects invalid state transitions and dependency cycles. Closing work requires evidence, an explicit waiver, or a reason the work will not happen. Replacing a Decision updates its predecessor too.

These checks apply regardless of which agent runs the command. They verify the recorded outcome; they cannot judge whether a report proves the work or whether a design choice is sound. That judgment stays with the people and agents doing the work.

### Read only what the session needs

`anb status` summarizes active work, rules, ready Tasks and signs of neglected work. Its configurable budget limits how much context the summary consumes. It reports omissions so the agent can request more. `anb show` opens one record, and scoped queries follow one branch of work.

Replies use short text tables and stable refusal codes, with `--json` for programs. The [reply contract](../reference/replies.md) describes the shapes; [Status and Debt](../reference/status.md) explains the budget and its limits.

## Start with the supplied workflow

`anb setup` installs the skills and agent integrations. The [quickstart](quickstart.md) demonstrates their workflow through the CLI. [Wiring agents](../guides/agents.md) explains how to keep your own version of the skills and choose a notebook location.
