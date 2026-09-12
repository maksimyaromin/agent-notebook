---
id: decision.an-interrupted-move-is-recognised-by
type: decision
state: superseded
kind: rule
title: An interrupted move is recognised by identity, never by bytes
by: Maksim Yaromin
superseded-by: decision.move-recovery
created: 2026-08-30
updated: 2026-09-12
---

When a move into the archive finds a file already at its destination, identity decides whether the move may go on — never whether it must write.

The file being moved from is the authoritative one. It can carry a correction made since the interrupted run, so a move that trusted what it found would delete that correction unread. Bytes cannot serve as the test in either direction: filing a report restamps it, so a sound resume never matches; and a record corrected since its interrupted move stops matching a twin that is its own.

Both destinations answer to one guard. A record is its own when it answers for the id it sits under; a report is when it was born inside this origin and wears the retired state the move gave it. Anything else — a foreign record, bytes no parse can read — refuses the move and keeps what it holds.

Before this, the record's own destination admitted by byte equality. An archive interrupted between its write and its remove, followed by any edit of the live copy, wedged the id permanently: every later archive refused with duplicate-id and no verb could clear it.
