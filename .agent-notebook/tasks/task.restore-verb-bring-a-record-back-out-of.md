---
id: task.restore-verb-bring-a-record-back-out-of
type: task
state: open
title: Restore verb: bring a record back out of the archive
by: Maksim Yaromin
from: task.check-finding-state-vs-residence-mismatc
tags: cli
priority: 2
created: 2026-08-29
updated: 2026-08-29
---

A record whose state still binds while its file sits in the archive is now a named error (archived-live-record), and every surface agrees on it: check reports it, Status Debt carries it, and archive refuses to call it an already-done move. Nothing can repair it. Every mutation verb resolves an id against the live directory first and refuses an archived one before it ever reads the file, so reopen, edit and close are all closed to it, and hand-editing the notebook is forbidden. Ship the inverse of archive — the same move back, same filename, same bytes, refusing when the live destination is taken — so the one corruption the tool can name is also one the tool can undo. Born from the independent review of the residence findings.
