---
id: note.report-graph-anb-graph-emits-the
type: note
state: retired
title: Report: Graph: anb graph emits the artifact
by: Maksim Yaromin
from: task.graph-emit-html
created: 2026-08-30
updated: 2026-08-30
---

# Graph: the notebook as a graph

## What shipped

`anb graph` prints the graph as data and writes a page only when asked. Both paths take the same slice flags, so a file never holds a wider graph than the one asked for — the failure Nx has, where `--focus` is silently dropped on `--file=out.html`.

- Data, in the reply family every other verb uses: `nodes[16]{id,state,archived,degree,epic,title}:` and `edges[27]{from,to,kind}:`, `ROW_BOUND`, `quoted_if_delimited`, `--all` to lift. `--json` gives one versioned document, `{"v":1,"slice":…,"nodes":…,"edges":…}`.
- Picture, under `--out <path>`: one self-contained file, no network, SVG drawn from dagre's layout. One `<g data-id="task.x">` per Task and one `<polyline data-source data-target>` per edge, so a reader can point at a task and a review harness can anchor to it.

## The defects the owner found, and what they were

**Names overprinted into smears.** Four things, because the root cause is layout, not drawing: a name's width is measured in the font it is actually drawn in and reserved in dagre's node box; whatever still collides gives way in order of how much the notebook leans on it; a name too small to read fades until the map is zoomed in; the hovered, picked and focused ones are always drawn. Proved in a browser: 0 collisions across ten cases, the worst being 42 nodes with the archive.

**Clicking a task showed nothing.** Two causes, both real. A tile is drawn at the weight of its dependencies, so a task nothing waits on was 4.1px across — a target a hand cannot hit. And names had `pointer-events: none`, so aiming at the name, which is what a reader does, landed on empty ground. Now every tile carries a circle held at 26px on screen at any zoom, and names are aimed at like anything else. Verified end to end: aiming at the name of a 9.8px tile opens its record.

**The fit clipped the outermost names.** `fit` sized the box from the base font, but the legibility floor draws small names wider than that, so they landed past the edge. It now solves for the largest zoom whose drawing fits, measuring positions and floored names together.

**Edges were nearly invisible** at `#454b5e` and .5 opacity — a graph that read as scattered dots. Lifted to `#5f6780`, .8 and .5.

## Removed

The theme apparatus (`Theme`, `ThemeError`, `--theme`, `--eject-theme`, `Reply::ThemeEjected`), `assets/graph.json`, `tools/graph-probe/`, and every layout engine but one: **816,837 bytes of vendored JavaScript down to 48,956** (`@dagrejs/dagre` 3.1.1, MIT). Map-related tests 48 → 43.

## Guarded

`cargo test -p anb-graph --test labels` opens an emitted page in a real browser and settles the two properties a machine can settle: no two names share a patch of screen, and every tile is big enough to point at. Both were mutation-proved — reverting the reach fails at 12.1px against a 16px floor. The test skips when no browser is installed rather than breaking the gate on a machine without one.

## Left open

- The task's own acceptance criteria still describe the shape before the owner's redirection; a comment on the task records what was delivered instead.
- A focused neighbourhood's names bypass the declutter by design, so picking a dense hub can overprint inside that neighbourhood.
- Headless and desktop Chrome give different font metrics, so the two show different name counts. The invariant holds in both.

## Fixed along the way

`crates/anb-core/src/notebook/query.rs.bak` was committed by accident in `ace46b8`; removed, and `.gitignore` now refuses `*.bak` and `*.orig`.
