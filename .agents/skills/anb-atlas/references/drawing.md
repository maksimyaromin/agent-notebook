---
name: anb atlas drawing
description: What anb graph --json carries, how each fact is encoded, and the layout rules that keep a drawn notebook readable and clickable. Open before writing the page.
metadata:
  managed-by: anb
---

# Drawing the notebook

## What the data carries

`anb --json graph` answers with `v`, the format version; `slice`, every narrowing that made the document (`for`, `type`, `ready`, `focus`, `depth`, `archive`); and `nodes` and `edges`, each a `count` and its `rows`.

A node carries `id`, `type`, `state`, `ready`, `archived`, `degree`, `created` and `title`; `priority` appears only on a Task that has one. A hub also carries `epic` with `id`, `closed`, `total` and `next`, the record to work on next inside it. Under `--full` it carries `fields`, the envelope as ordered name and value pairs, and `body` as `lines` and `head`; with `--all` the head is the whole body, and without it the body is cut like every listing and a `tail` follows the gap. `ready` is three-valued: `true` or `false` for a live Task, and absent where the question does not arise (a Decision never queues, and filed work is done). It travels with the record because it follows from rules the page cannot see, such as which holds stand.

An edge carries `from`, `to` and `kind`. `waits` runs out of the record that must settle first into the one waiting on it; `born` runs out of the origin into the record born from it; both point the way work flows. `mentions` runs the way it was written, out of the record that names another. A pair of records is one edge however many times the notebook states it.

## The encoding

Four facts have to read at a glance, so each gets its own visual channel and none of them shares one.

| Fact | Channel | Why |
|---|---|---|
| Kind of record | Hue, one per kind, the same on every page for one reader | The first thing a reader sorts by |
| How settled it is | Fill: live filled, settled outlined | The eye reads solid as alive |
| Filed away | Reduced opacity | An archived record still explains its edges and should recede |
| How much the notebook leans on a record | Radius from degree | A hub should look like one |
| Kind of relation | Stroke: solid waits, dashed born, dotted mentions | Three kinds on one channel, learned once |

The rail is the legend and the filter: every key turns its own class off, so the reader learns the encoding by using it.

## Layout

- Web is a force layout. Share the pull across a record's edges with the weight `1 / (1 + ln(1 + degree))`, so a hub does not drag its twenty neighbours onto itself.
- Ranked: stack by depth in the chain of `waits` edges, what holds everything up at the top and what waits on it below.
- Space records by the measured width of their names (`getComputedTextLength` in the face they are drawn in), not by the radius of their marks. Place names in order of degree and drop any that still collide. Below a legible rendered size, names fade instead of shrinking into smudges.
- Every mark carries an unpainted circle held at 26 px on screen whatever the zoom, and the name is part of the target: people aim at names.

## The five lessons that cost the most

1. A pointer capture steals the click. Capturing the pointer on `pointerdown` retargets every later event, `click` included, onto the capturing element, so a press on a record arrives at the canvas and the canvas closes the panel. Capture only once a drag has travelled past a dead zone. A press that never travels is a choice; one that travels is a drag.
2. A synthetic click is not a click. Dispatching `click` on an element bypasses hit-testing and pointer retargeting, so it passes on code a real cursor cannot use. Every interaction claim is checked by driving the real browser.
3. Aim is not a detail. A record with nothing waiting on it is drawn a few pixels across, and a mark a hand cannot hit is a record nobody opens. Hence the constant hit circle and the name as target.
4. Labels must be reserved, not repaired. Names overprint because the layout never knew they existed. Measure each name, space records by that width, then place and drop. The invariant is machine-checkable: no two visible label boxes intersect.
5. A hub collapses its own fan. Every edge pulls at full strength, so a record with twenty of them drags all twenty onto itself and the middle becomes a knot. Share the pull as above and the spokes stand apart.

## Before handing over

- No two visible label boxes intersect.
- Every interaction was checked in the real browser, with a real pointer.
- The page opens from a `file://` URL with the network off.
- The slice and the command that produced the data are on the page.
