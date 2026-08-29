---
id: task.check-finding-state-vs-residence-mismatc
type: task
state: open
title: Check finding: state vs residence mismatch
by: Maksim Yaromin
via: claude-code
from: task.milestone-cli-complete
tags: cli
blocked-by: task.cli-check-archive-edit-search-overview
priority: 2
created: 2026-08-29
updated: 2026-08-29
---

The archive verb gave every state a canonical residence, so the split became definable corruption: an open task sitting in archive/, or a hand-reopened record left in the archive, passes check clean today while silently vanishing from ready and every Debt clock — the exact silent-loss outcome Check exists to forbid. Ship a new finding code (state-residence-mismatch or better) that fires when a record's state and its live/archive location disagree, in both directions, reported per file. The finding-code catalog is a closed documented set: widening it means the format contract, the error catalog, and the negative corpus move together — corpus cases for both directions are part of done. Born from the independent review of the archive verb; question.should-state-vs-residence-disagreement-b routes here.
