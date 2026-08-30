---
id: task.serialize-concurrent-mutations-on-a
type: task
state: open
title: Serialize concurrent mutations on a notebook lock
by: Maksim Yaromin
via: claude-code
from: decision.concurrent-mutations-serialize-on-a
priority: 2
created: 2026-08-30
updated: 2026-08-30
---

The host takes an exclusive advisory lock for the whole read-modify-write window of every mutating command, so two agents can never lose one another's write.

Shape: a lock guard in the CLI crate over a lock file in the notebook root, taken with std::fs::File::lock before the command runs and dropped when it ends. Read-only commands take nothing. The Core is untouched: it holds no lock and its Storage seam gains no method.

Acceptance: a mutating command holds the lock for its whole window, including the read that precedes the write; twenty concurrent comments on one Task all land; a read-only command run against a locked notebook answers immediately; the lock file is never a record to any listing, query, or check, and a notebook committed to git does not carry it; the gate is green.
