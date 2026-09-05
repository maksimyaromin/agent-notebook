---
name: anb-atlas
description: Use when asked to see, draw, map or picture a notebook or a part of it (the whole graph, one epic, what blocks what, what can start, how far a hub has come), or to decide about records by pointing at a picture: pick the next Task, triage a queue, review a branch of work, leave comments on records in one page. Not for reading one record or a list; anb show and anb list do that.
metadata:
  managed-by: anb
---

# anb atlas

## Overview

The CLI serves the notebook as data and draws nothing. `anb graph --json` answers with every record and every edge of the slice asked for, and whoever wants a picture builds one from that, in the shape the question needs. A page for "pick the next Task" differs from the page for "show me the whole notebook", and only the asker knows which one they wanted. So the page is generated per question and never shipped frozen, and the picture belongs to whoever asked for it.

Two things follow. The page is one self-contained HTML file: the data is embedded at build time and the command that produced it is printed at the bottom, so a reader can tell what they are looking at and reproduce it. It has no server, no network, no library and no knowledge of where the notebook lives. And the page is where the reader decides, while the notebook changes only through `anb`: what the reader decides on the page comes back to you as comments, and you turn each into a command.

## When to use

- The developer asks to see the notebook, an epic, the queue or the neighbourhood of one record, or asks a question a picture answers faster than a listing: what holds everything up, what is ready, how far a hub has come, where the archive sits.
- The developer wants to decide by pointing: triage a queue, pick what to start, review a branch of work and say what to do with each record.
- Someone asks for a review of the notebook's shape: cycles, hubs, work nobody waits on.

One record is `anb show`, a filtered list is `anb list` or `anb ready`, and the session's opening is `anb status`; none of those needs a page. The page itself changes nothing in the notebook.

## Quick reference

One command is the whole input. Slice flags compose, and every one narrows both the data and anything drawn from it.

| Question | Command |
|---|---|
| The whole notebook, archive included | `anb --json graph --archive --full --all` |
| One epic's branch | `anb --json graph --for <hub> --full --all` |
| What can start now | `anb --json graph --ready --full --all` |
| Around one record | `anb --json graph --focus <id> --depth 2 --full --all` |
| One kind of record | `anb --json graph --type task --full --all` |

`--json` is never bounded, because a drawing made from some of the edges is a picture of a notebook that does not exist. `--full` carries each record's envelope and body, so the page can open a record without a second call. The document names in `slice` every narrowing that made it, and the page shows that slice: a graph that does not say what it left out reads as a notebook that holds nothing else.

Before writing the page, open [drawing](references/drawing.md): what the data carries, how each fact is encoded, the layout rules and the five lessons that cost the most. Before collecting the reader's decisions, open [the intent loop](references/intent-loop.md): how comments are addressed, how they come back and how they become commands.

## The page

- The page is one file and uses no library. The data is embedded as JSON; the force layout, the ranking, the label placement and the collision spacing are about two hundred lines of plain JavaScript, cheaper than a dependency and easier to change per question. Nothing is fetched. The styling is the page's own, never copied from a repository.
- Four facts read at a glance, each on its own channel: the kind of record by hue, how settled it is by fill (live filled, settled outlined), filed away by reduced opacity, how much the notebook leans on a record by radius from degree. Relations are told apart by stroke: solid for waits, dashed for born, dotted for mentions. The legend is the filter, and every key turns its own class off.
- Finished work has a picture. Closed, superseded and retired records are outlined, archived ones fade, and a hub carries `closed/total` from its `epic` field, whose `next` names what to work on next inside it, so progress reads without opening it.
- A record opens beside the map in a side panel, never over it. The reader reviews by pointing and weighs a record against its neighbours, which a covering panel makes impossible. The panel shows the envelope line by line, the body, and every relation with a jump to the other end.
- A reader can click a record or its name to open it, drag a record to pin it and double-click to release it, drag the ground to pan, scroll to zoom, press `/` to search and `Escape` to close, and switch between a force web and a ranked stack ordered by how deep a record sits in the chain of what must settle first.
- Verify by driving the real browser. A dispatched click bypasses hit-testing and passes on code a real cursor cannot use.

## Common mistakes

| Mistake | Instead |
|---|---|
| Drawing from the plain-text `anb graph`, which is bounded | `--json`, which is never bounded |
| A picture of a slice that does not say so | Show `slice` on the page and print the command at the bottom |
| Vendoring a graph library, or copying a theme from the repository | Plain JavaScript and the page's own styling |
| A modal over the map | A side panel beside it |
| A record a few pixels across that a hand cannot hit | A constant on-screen hit circle around every mark; the name is part of the target |
| Names that overprint | Reserve label space in the layout, place by degree, drop what still collides |
| Writing the notebook from the page, or editing files after reading it | An `anb` command per comment, run by you and reported |
| Testing clicks with dispatched events | Drive the real browser |
