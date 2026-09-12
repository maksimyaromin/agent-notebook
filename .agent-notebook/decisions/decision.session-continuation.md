---
id: decision.session-continuation
type: decision
state: active
kind: rule
title: Separate record validity from local session coordination
by: Maksim Yaromin
via: codex
from: task.shared-memory
supersedes: decision.the-cli-keeps-a-record-s-invariants-not
created: 2026-09-12
updated: 2026-09-12
---

The Core enforces record states, relationships, references and outcomes. A person may have several active Tasks. The host remembers one focus per local session, performs next-work selection and claiming under one lock, and requires an explicit join before two local sessions share a Task. Switching focus does not hold other work. Missing or ambiguous focus requires an explicit choice, not a guess from timestamps. Local session claims are not distributed locks across disconnected clones.
