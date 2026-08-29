---
id: decision.close-proofs-are-equals-default-note
type: decision
state: active
kind: rule
title: Close proofs are equals; the default route is a Note
by: Maksim Yaromin
via: claude-code
from: question.where-does-a-report-proof-live-so-a-fres
created: 2026-08-29
updated: 2026-08-29
---

Every proof flag on close is fully functional and none is second-class: --pr, --sha, --report with a path to a file anywhere (machine-local is legitimate), or no proof at all. The default route the docs recommend is new: --note takes a report file and ingests it as a Note born from the task in one motion, so the proof lives inside the notebook and travels with it — committed or relocated, a reader who can reach the task can reach its report. This respects users who move the notebook elsewhere, keep it uncommitted, or skip reports entirely; the tool never forces a location or a commit policy.
