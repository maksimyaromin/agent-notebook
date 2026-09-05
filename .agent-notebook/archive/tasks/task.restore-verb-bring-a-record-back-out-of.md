---
id: task.restore-verb-bring-a-record-back-out-of
type: task
state: closed
title: Restore verb: bring a record back out of the archive
by: Maksim Yaromin
from: task.check-finding-state-vs-residence-mismatc
tags: cli
link: note note.report-restore-verb-bring-a-record-back
priority: 2
created: 2026-08-29
updated: 2026-08-31
closed: 2026-08-31
---

A record whose state still binds while its file sits in the archive is now a named error (archived-live-record), and every surface agrees on it: check reports it, Status Debt carries it, and archive refuses to call it an already-done move. Nothing can repair it. Every mutation verb resolves an id against the live directory first and refuses an archived one before it ever reads the file, so reopen, edit and close are all closed to it, and hand-editing the notebook is forbidden. Ship the inverse of archive — the same move back, same filename, same bytes, refusing when the live destination is taken — so the one corruption the tool can name is also one the tool can undo. Born from the independent review of the residence findings.
- 2026-08-30 Maksim Yaromin: check now names the move that erases each finding, and archived-live-record is the one error with no move to name: a live record inside the archive is unreachable by every verb. restore is its repair, so this task closes the last gap in the repair column.
- 2026-08-30 Maksim Yaromin: Correction to the note above: archived-live-record is not the only error with no move — a bad-value on a state line has none either, and no finding on an archived record has one at all, since every verb resolves an id to its live path. restore is still this task's point: it is the move that brings such a record back where a verb can reach it.
- 2026-08-31 Maksim Yaromin: Shipped and green: anb restore <id> is the inverse of archive — same filename, same bytes, the record alone (carried reports are retired history and stay put). The bytes travel unjudged, so a broken archived record can come back to where the repairing verbs are. The live directory is the only editable home, so a standing live copy is never overwritten: a resume finishes an interrupted move by removing the archived leftover when the copy is provably this record's own (byte-identical, or clean — add refuses an id the archive claims), and refuses duplicate-id otherwise. Replay answers already on residence; NotUtf8 refuses on either side of the move. check now names anb restore on archived-live-record — the one finding restore erases — and the archived error's try line offers it. 14 core tests + 2 CLI tests, each proven able to fail; gate green; smoked live on a scratch notebook. One decision clause is now stale (a-repair-is-progress-not-perfection: 'a record in the archive has no move at all') — supersession drafted for the maintainer's confirmation in the report.
- 2026-08-31 Maksim Yaromin: The independent review returned 0 defects, 9 should-fixes, 10 nits; all should-fixes and 8 nits applied. The substantial ones: the 'restore is the only verb reaching the archive' claim was false (expunge resolves both homes) and was swept out of five homes; the resume test had a real hole — a foreign leftover could be removed unread — closed by requiring the leftover's own bytes to declare the id; the verb was reshaped into held_at + a flat truth table; write-before-remove and the end-to-end repair promise gained tests. Correction to the previous entry: three CLI tests, not two. gate green.
