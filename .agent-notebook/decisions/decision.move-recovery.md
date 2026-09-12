---
id: decision.move-recovery
type: decision
state: active
kind: rule
title: A repeated move never discards a divergent copy
by: Maksim Yaromin
via: codex
from: task.shared-memory
supersedes: decision.an-interrupted-move-is-recognised-by
created: 2026-09-12
updated: 2026-09-12
---

Archive and restore can finish an interrupted move when the two copies have identical bytes. A shared id does not prove that one copy contains every contribution. If the copies differ, preserve both and refuse the move with their paths and reconciliation guidance. Compare and merge the contributions while retaining originals before retrying. Do not use delete to repair this condition: it removes the record, including both copies.
