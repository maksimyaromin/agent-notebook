---
id: task.graph-emit-html
type: task
state: open
title: Graph: anb graph emits the artifact
by: Maksim Yaromin
via: claude-code
from: task.task-graph-visualization-for-fun
blocked-by: task.release-gate-v1
created: 2026-08-29
updated: 2026-08-29
---

anb graph [--for <hub>] [--ready] writes one self-contained HTML artifact, data embedded: task-centric map, id-only tiles colored by state, full record in a modal, blocked-by (solid) and origin (dashed) edges unlabeled, mentions only inside the modal. Slices: full map, epic branch, ready lens. No server; the CLI stays the single write path.
