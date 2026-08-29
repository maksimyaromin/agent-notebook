---
id: decision.backticked-id-is-a-quotation
type: decision
state: active
kind: rule
title: A backticked id is a quotation; a bare id is a reference
by: Maksim Yaromin
via: claude-code
from: question.how-does-prose-mention-a-record-id-witho
created: 2026-08-29
updated: 2026-08-29
---

Mentions are derived, never registered: the scan re-reads every body and treats every id-shaped token as a live reference, which is what keeps the markdown the source of truth. Prose that only talks about an id therefore needs an opt-out. The rule: an id inside a markdown code span is a quotation and creates no mention edge; a bare id is a reference. Alongside it, a write-time nudge: any mutating command whose text introduces a bare id that does not resolve warns in its own output — backtick it to quote, or create the record — so the writer fixes it in the same breath. Strict rejection was considered and refused: it would kill legitimate forward references and still could not touch hand edits or later removals. Dangling-mention debt remains the net for everything the CLI never saw.
