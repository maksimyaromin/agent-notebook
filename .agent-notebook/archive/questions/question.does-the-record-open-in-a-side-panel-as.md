---
id: question.does-the-record-open-in-a-side-panel-as
type: question
state: closed
title: Does the record open in a side panel, as shipped, or in the modal the design names?
by: Maksim Yaromin
from: task.graph-emit-html
resolved-by: task.claude-code-skill-the-notebook-drawn-and
created: 2026-08-30
updated: 2026-08-31
---

The settled design for the map says a click opens the full record in a modal. What shipped opens it in a side panel instead: an aside pinned to one edge, with the graph keeping the rest of the viewport and the tile staying visible beside the record it stands for. The panel was chosen because a modal covers the very graph the reader is pointing at, and review by pointing is the map's whole job — a reader picks tiles, reads what each one says, and carries a batch of instructions back to the CLI, which is work they cannot do while the record they opened hides the neighbours they were weighing it against. Two ways to settle it. One, keep the panel and correct the design to name a side panel, on the reasoning above. Two, restore the modal as designed, accepting that reading one record hides the map behind it, and drop the panel.
