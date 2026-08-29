---
id: decision.envelope-dates-mean-notebook-time
type: decision
state: active
kind: rule
title: Envelope dates mean notebook time, not project history
by: Maksim Yaromin
via: claude-code
from: question.should-record-import-preserve-historical
created: 2026-08-29
updated: 2026-08-29
---

created, updated, and closed on the envelope answer when the notebook learned it, never when it happened in the project; project history lives in the body, where a migration writes it. No backdating flags on ordinary verbs — envelope dates stay something the notebook vouches for. A dedicated import surface, where backdating is explicit and fenced, is a future concern for teams migrating from another tracker. One-time exception, owner-approved: the ten records imported by the self-host migration get their envelopes hand-fixed to the true historical dates, because their notebook-time dates all read as the single migration day and carry no information.
