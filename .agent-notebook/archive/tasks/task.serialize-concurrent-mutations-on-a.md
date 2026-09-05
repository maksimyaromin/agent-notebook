---
id: task.serialize-concurrent-mutations-on-a
type: task
state: closed
title: Serialize concurrent mutations on a notebook lock
by: Maksim Yaromin
via: claude-code
from: decision.concurrent-mutations-serialize-on-a
link: note note.report-serialize-concurrent-mutations-on
priority: 2
created: 2026-08-30
updated: 2026-08-30
closed: 2026-08-30
---

The host takes an exclusive advisory lock for the whole read-modify-write window of every mutating command, so two agents can never lose one another's write.

Shape: a lock guard in the CLI crate over a lock file in the notebook root, taken with std::fs::File::lock before the command runs and dropped when it ends. Read-only commands take nothing. The Core is untouched: it holds no lock and its Storage seam gains no method.

Acceptance: a mutating command holds the lock for its whole window, including the read that precedes the write; twenty concurrent comments on one Task all land; a read-only command run against a locked notebook answers immediately; the lock file is never a record to any listing, query, or check, and a notebook committed to git does not carry it; the gate is green.
- 2026-08-30 Maksim Yaromin: Done. Exclusive claim for writers, shared for readers, on <root>/.lock; classification in lock::writes, one exhaustive match. Measured: 20 concurrent comments left 7 before, 20 after; 20 concurrent adds left 7 files before, 20 after; a reader loop against a running archive cascade went from 7 spurious failures in 80 reads to 0. Overhead below noise (comment 10.6 ms both, list 42.1 ms both, at 1000 live records). The notebook root gains a .gitignore naming the lock and the temp file, written with create_new.
