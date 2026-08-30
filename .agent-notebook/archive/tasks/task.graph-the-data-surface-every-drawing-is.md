---
id: task.graph-the-data-surface-every-drawing-is
type: task
state: closed
title: Graph: the data surface every drawing is built from
by: Maksim Yaromin
link: note note.report-graph-the-data-surface-every
priority: 1
created: 2026-08-30
updated: 2026-08-30
closed: 2026-08-30
---
- 2026-08-30 Maksim Yaromin: The picture left the CLI: crates/anb-graph deleted whole — dagre, map.js, map.css, the SVG renderer and --out. Owner's call, 2026-08-30: a CLI has no business shipping a browser runtime, and the page a reader wants is the one their own agent builds for the question they asked.
- 2026-08-30 Maksim Yaromin: The defect that made this urgent: --json truncated to ROW_BOUND, handing an agent 20 of 42 nodes and 20 of 96 edges. A drawing made from that is not a smaller picture of the notebook but a picture of one that does not exist. Structural blocks are now never bounded; prose inside a record still obeys --all.
- 2026-08-30 Maksim Yaromin: The graph became the notebook's, not the queue's: every kind of record is a node (16 tasks, 14 decisions, 1 note, 5 questions live), --type slices by kind, mentions joined waits and born as a third edge, and nodes carry type, ready, priority and created so one call answers which task to take next.
- 2026-08-30 Maksim Yaromin: Review by a separate Opus 5 agent returned 16 findings, all real. Four reproduced on live data before fixing: 12 duplicate edges, --type absent from the echoed slice, --type with --focus answering empty at exit 0, and --help still promising an HTML file. Kinds became RecordType instead of strings, which removed the validation loop and a fourth hand-written copy of the kind list.
- 2026-08-30 Maksim Yaromin: Proved by drawing it: the whole notebook as one page, built from a single anb --json graph --archive --full --all. Owner's verdict — almost ideal. Kept with its principles in .tmp/atlas/.
