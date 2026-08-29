---
id: question.should-record-import-preserve-historical
type: question
state: open
title: Should record import preserve historical dates?
by: Maksim Yaromin
via: claude-code
from: task.milestone-self-host-switch
created: 2026-08-29
updated: 2026-08-29
---

The self-host migration replayed ten already-finished tasks through open→active→closed, so their created/closed envelope dates read 2026-08-29 while the true dates live only inside the migrated body text. Any team switching to anb from another tracker hits the same wall. Candidate answers: backdating flags on add/close, a dedicated import surface, or a ruling that envelope dates mean notebook time, not project history.
