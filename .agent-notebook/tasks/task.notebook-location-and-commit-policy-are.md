---
id: task.notebook-location-and-commit-policy-are
type: task
state: open
title: Notebook location and commit policy are the user's call
by: Maksim Yaromin
via: claude-code
from: decision.close-proofs-are-equals-default-note
priority: 3
created: 2026-08-29
updated: 2026-08-29
---

Users must stay free to put the notebook where they want and commit it or not: at the repo root and committed, or relocated (for example into a git-ignored scratch area) and private — the way a personal backlog file works. Today the notebook path is fixed. Give it a configurable location honored by every command, and make sure nothing in the tool assumes the notebook is committed. Acceptance: a relocated notebook passes the full verb cycle; docs state the location and commit policy are configuration, not doctrine.
