---
id: note.report-status-a-held-task-is-not-in
type: note
state: retired
title: Report: Status: a held Task is not in flight
by: Maksim Yaromin
from: task.status-a-held-task-is-not-in-flight
created: 2026-09-05
updated: 2026-09-05
---

# Status: a held Task is not in flight (2026-09-05)

Report for task.status-a-held-task-is-not-in-flight, a finding met in the marathon: holding the active Task and starting the next one left Status printing two `active:` lines, the held one first.

## What shipped

A hold is a pause somebody chose, so the paused Task is not the one a session resumes from. Status now:

- keeps a held Task out of its `active:` lines, whatever state the hold froze it in;
- names every held Task in its own bounded section, `held[N]{id,reason,until}`, so a session can see what waits and for what, and whether that reason has lifted;
- collapses that section to a count at the same rung as review — both are work standing still — and drops it at the floor, which keeps the counts, the first active line and the budget line as before;
- treats a hold as no signal: a notebook whose only work is on hold stays quiet, and a hold gone stale is Debt's to raise, as it already was.

The JSON rendering carries the same `held` section.

## The second question, decided

The task asked whether `start` should refuse while another Task is active and unheld, since the protocol says there is never more than one in flight. It does not, and no verb will: decision.the-cli-keeps-a-record-s-invariants-not (recorded from this task) states the rule — the Core enforces what makes a notebook readable by any team, and one Task in flight at a time is a working protocol the skill teaches and Status shows, not a rule of the record model.

## Tests

Core: a held active Task is not an active line and waits in the held section with its reason and date beside a held open one; a notebook whose only work is held is quiet; the floor drops the held section. CLI: both renderings of a notebook with one active and one held Task. The gate is green.
