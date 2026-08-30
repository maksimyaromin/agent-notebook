---
id: question.should-two-agents-writing-one-record-be
type: question
state: routed
title: Should two agents writing one record be serialized?
by: Maksim Yaromin
via: claude-code
routed-to: decision.concurrent-mutations-serialize-on-a
created: 2026-08-30
updated: 2026-08-30
---

The filesystem adapter replaces a record atomically — temp file, rename — so no reader sees half a write. What it cannot do is make a read-modify-write cycle safe: comment reads the record, appends its log line, and writes the whole file back, so two agents commenting on the same Task concurrently produce one file, one comment lost, and exit 0 on both. The project's first goal is that any agent can read and mutate the notebook, and plural agents is the design point, which makes this the seam's gap rather than the Core's — the Core has no lock to reach for. Closing it means a Storage extension: a lock file taken for the mutation window, or a compare-and-swap write that refuses when the bytes moved under it. The second invents nothing and needs no cleanup after a crash, but every mutation verb grows a retry story. Nothing has reported losing a comment yet.
