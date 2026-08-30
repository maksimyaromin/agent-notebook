---
id: decision.concurrent-access-serializes-on-a
type: decision
state: active
title: Concurrent access serializes on a notebook lock, readers included
by: Maksim Yaromin
tags: storage, concurrency
supersedes: decision.concurrent-mutations-serialize-on-a
created: 2026-08-30
updated: 2026-08-30
---

Two agents on one notebook are serialized by the host, not by the Core. A mutating command takes an exclusive advisory lock on a single lock file in the notebook root for its whole read-modify-write window; a read-only command takes a shared one on the same file, and readers never block each other.

The superseded Decision held that readers take nothing. That was wrong for a measurable reason: a settling verb moves several files in turn — archive writes the destination then removes the source, expunge removes a record and its reports, a supersession writes the superseder then flips the victim — and a reader landing between two of them reports the half-finished state as corruption. One process archiving 60 closed Tasks while another read in a loop produced 7 spurious failures in 80 reads: check naming a duplicate-id on a record merely mid-move, list failing outright with not found. Per-file atomic writes cannot close a window that spans files.

A reader takes its claim only if the lock file is already there, and nothing on the read path creates one: a notebook no writer has ever touched has no one to wait for, and a notebook mounted read-only stays readable. The lock is released by the kernel when the process ends, so a crash leaves nothing to clean up and no verb grows a retry story. The Core keeps no lock and knows of none; concurrency is a property of the host that hands it Storage.
