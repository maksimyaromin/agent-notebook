---
id: decision.a-repair-is-progress-not-perfection
type: decision
state: active
kind: rule
title: A repair is progress, not perfection
by: Maksim Yaromin
via: claude-code
from: decision.a-check-finding-names-the-move-that
tags: cli
created: 2026-08-30
updated: 2026-08-30
---

Refines decision.a-check-finding-names-the-move-that, which is archived and cannot be superseded in place. Two clauses were wrong there and one was missing.

A repairing verb — edit, unblock, unhold — is judged on the record it produces: it must leave fewer of the record's own error findings than it read, and none the record did not already carry. Not zero. Demanding zero made a record broken two ways unrepairable through the CLI at all: each clear was refused because the other error still stood, which is the deadlock the repair column was built to end. Progress is the bar; check names what remains.

A repair is named only on a file a verb can reach. Every mutation resolves an id to the one live path its type dictates, so a record in the archive, in the wrong directory, or under a filename that is no id has no move at all, whatever is wrong inside it — and a row that named one anyway would send an agent in a circle.

unhold erases the hold pair by name, so it repairs as well as resumes, and a finding on a hold or hold-until line names it. That was the last line the CLI writes with no way to take back.

What the earlier Decision got right stands: the repair is read off the line the finding sits on, it is not a function of the finding's code, and the CLI gains no eraser for a field only a hand edit can break.
