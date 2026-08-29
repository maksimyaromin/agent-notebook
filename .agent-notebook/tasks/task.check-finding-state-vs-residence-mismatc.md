---
id: task.check-finding-state-vs-residence-mismatc
type: task
state: closed
title: Check finding: state vs residence mismatch
by: Maksim Yaromin
via: claude-code
from: task.milestone-cli-complete
tags: cli
link: sha 9b5a693
blocked-by: task.cli-check-archive-edit-search-overview
priority: 2
created: 2026-08-29
updated: 2026-08-29
closed: 2026-08-29
---

The archive verb gave every state a canonical residence, so the split became definable corruption: an open task sitting in archive/, or a hand-reopened record left in the archive, passes check clean today while silently vanishing from ready and every Debt clock — the exact silent-loss outcome Check exists to forbid. Ship a new finding code (state-residence-mismatch or better) that fires when a record's state and its live/archive location disagree, in both directions, reported per file. The finding-code catalog is a closed documented set: widening it means the format contract, the error catalog, and the negative corpus move together — corpus cases for both directions are part of done. Born from the independent review of the archive verb; question.should-state-vs-residence-disagreement-b routes here.
- 2026-08-29 Maksim Yaromin: Shipped two codes, not one: archived-live-record (error) for a record that still binds from inside the archive, unarchived-settled-record (warning) for a settled record still in the working set. One code could not carry both, since severity is a property of the code and the two outcomes differ in kind — the first is invisible to every derived query, the second only unfiled. Corpus: two error cases (task, decision) under invalid/archive/, one warning case under valid/tasks/. Routing cases moved under archive/ so each isolates its class.
- 2026-08-29 Maksim Yaromin: Correction to the earlier log entry: the corpus routing cases did NOT move under archive/ — they stayed in questions/ and gained unarchived-settled-record in their sidecars, so the corpus proves the two codes land together on one file. What moved is the record.rs unit-test question fixture, which sits in the archive so a routing finding stands alone. Three review passes: pass 1 found the finding message falsely claimed no derived query reads the archive (search does; ready treats an archived open task as a blocker). Pass 2 found the reworded message still false (view reaches it) and two defects the fix had introduced. Pass 3 found the archive-filter test still unpinned and, worse, that the new warning pushed real errors out of the default check under its 20-row bound — finding_order now keys on severity first, so an error can never be crowded out by a warning the tool's own happy path produces.
