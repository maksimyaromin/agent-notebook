---
id: decision.review-recipient
type: decision
state: active
kind: rule
title: A review recipient describes the current handoff
by: Maksim Yaromin
via: codex
from: task.shared-memory
supersedes: decision.a-link-is-an-edge-and-to-is-the-smallest
created: 2026-09-12
updated: 2026-09-12
---

Typed links describe named relationships and support incoming reads, graph traversal and subject scope without duplicating IDs in prose. A Task in review or an open Question waits on its to recipient. Resuming a review or reopening a Task clears the old review recipient; a new submission names its current recipient. Closing retains historical attribution without continuing the handoff. The notebook records context; it does not send messages or replace the team's review system.
