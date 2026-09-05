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

The map is the page. Beside it stands one narrow column: a find box, the legends, and the arrangement switch. The legends are the filters, one row per key with its count, and there is one legend per fact a reader can switch off: kind, settled or live, filed away, relation. Degree is read from size and needs none. Below the map runs one small line: the command that produced the data and the slice it names. That is all the chrome there is. No header of controls, no dashboard of counters, no disclaimer, no second footer; the reader's eye rests on the map.

- The page is one file and uses no library. The data is embedded as JSON; the force layout, the ranking, the label placement and the collision spacing are about two hundred lines of plain JavaScript. Nothing is fetched. The styling is the page's own, never copied from a repository.
- Four facts read at a glance, each on its own channel: the kind of record by hue, how settled it is by fill (live filled, settled outlined), filed away by reduced opacity, how much the notebook leans on a record by radius from degree. Relations are told apart by stroke: solid for waits, dashed for born, dotted for mentions.
- The page opens readable. The opening zoom fits the names of the records the notebook leans on most, so a reader sees words before touching anything; names fade only when the reader zooms out past legibility.
- Web is the arrangement for a notebook; ranked is for a branch. Depth in the chain of what must settle first is small for most records, so a whole notebook ranked becomes a rope of a few rungs. Rank one epic's slice, where the chain is the story.
- Finished work has a picture: closed, superseded and retired records are outlined, archived ones fade, and a hub carries `closed/total` from its `epic` field, whose `next` names what to work on next inside it.
- A record opens beside the map in a side panel, never over it, and opens on what the reader came for: the title, the body, then every relation with a jump to the other end; the envelope comes last, folded. Comments are made in that panel; a drawer that collects them opens only when the reader asks for it.
- A reader can click a record or its name to open it, drag a record to pin it and double-click to release it, drag the ground to pan, scroll to zoom, press `/` to find and `Escape` to close.
- Verify by driving the real browser. A dispatched click bypasses hit-testing and passes on code a real cursor cannot use.

## Common mistakes

| Mistake | What it costs | Instead |
|---|---|---|
| Chrome that outgrows the map: a header of controls, a rail of counters, a footer of provenance, a disclaimer | The reader's eye has nowhere to rest, and the page reads as an instrument panel rather than a map | One narrow column (find, legends, arrangement), one small line under the map (command and slice), nothing else |
| Ranked arrangement on the whole notebook | Most records sit at a depth of one or two, so the notebook ranks into a rope of a few rungs with every name hidden | Rank one branch (`--for <hub>`); the whole notebook is a web |
| Names faded at the opening zoom | The page opens as dots; the reader has to zoom before reading a single word | Fit the opening zoom to the names of the most-leaned-on records; fade names only past legibility |
| `--archive` in a working question | Settled records outnumber live ones several times over, and the work the question was about hides among grey outlines | The slice follows the question; the archive belongs in a picture about history |
| The panel opens on the envelope | Twenty `blocked-by` rows stand between the reader and the body they clicked for | Title, body, relations with jumps; the envelope last and folded |
| Two facts on one channel, colour for kind and for state | A settled Task and a live Decision can wear the same look, and the legend cannot say which | One channel per fact, as the encoding table lays them out |
| Drawing from the plain text of `anb graph` | The plain text is bounded like every listing; a picture of some of the edges is a picture of a notebook that does not exist | `anb --json graph …`, which is never bounded |
