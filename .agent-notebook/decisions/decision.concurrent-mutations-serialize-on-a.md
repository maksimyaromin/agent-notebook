---
id: decision.concurrent-mutations-serialize-on-a
type: decision
state: active
kind: rule
title: Concurrent mutations serialize on a notebook lock
by: Maksim Yaromin
via: claude-code
from: question.should-two-agents-writing-one-record-be
tags: storage, concurrency
created: 2026-08-30
updated: 2026-08-30
---

Two agents mutating one notebook are serialized by the host, not by the Core. Every mutating command takes an exclusive advisory lock on a single lock file in the notebook root for the whole read-modify-write window; read-only commands take nothing and are never blocked.

The hazard is real and silent: every verb reads a record, splices it, and writes the whole file back, so two concurrent comments on one Task produce one file, one lost comment, and exit 0 on both. An atomic write — temp file, rename — prevents a torn file and nothing else.

The lock wins over a compare-and-swap write because it costs the Core nothing. An advisory lock is released by the kernel when the process ends, so a crash leaves no stale state to clean up and no verb grows a retry story; the alternative would push a conflict outcome through the Storage seam, which every adapter would then have to implement. The Core keeps no lock and knows of none: concurrency is a property of the host that hands it Storage.

A mutation window is milliseconds, so serializing every writer costs a waiting process nothing worth measuring. The lock file is not a record: it carries no .md suffix and sits in the root rather than in a type directory, so no listing, query, or check ever sees it.
