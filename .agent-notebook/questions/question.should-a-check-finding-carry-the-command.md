---
id: question.should-a-check-finding-carry-the-command
type: question
state: open
title: Should a Check finding carry the command that repairs it?
by: Maksim Yaromin
from: task.erase-an-optional-envelope-field-through
tags: cli
created: 2026-08-30
updated: 2026-08-30
---

check names a file, a line, a severity, a code and a message, and never what to do about it. An agent reading a finding has to infer the repair: an origin cycle is cleared by edit --clear from, a corrupted edge by unblock, a missing title by edit --title, and a bad kind by nothing the CLI offers.

The case for: the reader of a finding is an agent, and the repair is the only thing it will do next. The case against: a hint on every row costs tokens on the one reply that is already the longest, several findings have no repair at all, and a hint that goes stale is worse than none. A third shape exists — the repair belongs to the code, not the row: check could name the codes that are repairable and the reply carry one line for the whole report.

Deciding this also decides the sibling question the same task left open: whether every optional envelope line should be repairable through the CLI at all. The machine-written ones — kind, by, via, link, supersedes, superseded-by, routed-to, updated, closed — can only be corrupted by a hand edit, and edit can write none of them, so clearing one would be a one-way door.
