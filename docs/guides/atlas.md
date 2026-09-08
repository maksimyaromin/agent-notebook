---
title: Drawing the notebook
description: 'Explore dependencies and project history in an interactive map, then send comments back to the agent.'
---

Ask your agent to draw the notebook when a list no longer answers the question. A map can show what blocks a release, how an epic divides into work, or where a Decision came from. The `anb-atlas` skill installed by `anb setup` teaches the agent to build that view from the CLI's graph data.

## Ask for the view you need

"Show what is blocking the parser epic" and "show the whole notebook, including finished work" need different amounts of context. Name the question when asking for a map. The agent selects the graph slice and builds an interactive HTML file with the data embedded.

You can open a record beside the map to read its body and relationships. Filters narrow the view, and completed and archived work remain visually distinct. The page identifies the command that produced its data, so you can reproduce the view or ask for a fresh one.

## Review on the map

Leave comments on individual records or on the view as a whole, then return the batch to the agent. It rereads the affected records in the original notebook, translates the comments into commands under the main working method, and reports the result of each. A comment on the whole view applies to its captured records, not records added after the map was drawn. For example, a comment asking to pause a Task needs a reason before it can become `anb hold`.

The page never writes record files. It is a snapshot you can keep or share; changes go through `anb`, with the same checks as any other command. Ask for a new map after changes if you need the updated state.

## Get the graph directly

`anb graph --json` supplies data for your own renderer too:

| View | Command |
|---|---|
| Whole notebook, including the archive | `anb graph --json --archive --full --all` |
| One epic | `anb graph --json --for <hub> --full --all` |
| One person's work | `anb graph --json --mine --full --all` |
| Neighbors of one record | `anb graph --json --focus <id> --depth 2 --full --all` |
| Tasks | `anb graph --json --type task --full --all` |

`graph` takes the same narrowing flags as `list`: `--for`, `--tag`, `--match`, `--by`, `--mine`, `--team`, `--type`, `--kind` and `--archive`, composable. Graph JSON includes every node and edge in the selected slice. `--full` adds record envelopes and bodies; `--all` removes their text and field limits. Omit `--full` when ids, titles and state are enough.

The document contains `v` for the format version, `slice` for the query, `nodes` and `edges`. Nodes include identity, type, state, readiness, archive status, degree, creation date and title. Tasks may include priority, and hubs include epic progress. A page about what can start now reads readiness off each node rather than asking for a narrower slice: a ready Task waits on nothing live, so a slice of ready Tasks alone would have no edges to draw.

| Edge | Direction |
|---|---|
| `waits` | The prerequisite points to the Task waiting on it |
| `born` | The origin points to the record created from it |
| `mentions` | The citing record points to the cited record |
| any other word | A `link` line whose target is a record, under the link's own kind: the record carrying the link points to the record it names |

The CLI defines the graph; the skill defines the presentation and review workflow. Its installed files contain the layout guidance for agents building a page.
