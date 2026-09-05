---
id: decision.expunge-for-records-born-by-mistake
type: decision
state: active
kind: rule
title: Delete removes a record born by mistake
by: Maksim Yaromin
via: claude-code
from: question.does-the-notebook-need-an-expunge-for-re
created: 2026-08-29
updated: 2026-09-05
---

retire ends a Decision or Note that was once true; supersession replaces one; the future archive moves records with their history. A record created in error is a third case: it should leave no trace, and deleting its file by hand bypasses everything derived. The notebook therefore gets an expunge verb: it deletes the record, and it refuses while any inbound reference exists — a mention edge from another body or an origin link on a record the mistake spawned — listing every blocker with its carrier so the repair path is obvious. There is no --force: a flag whose defined outcome is dangling references is better off not existing. Repair the references first; then expunge succeeds.
