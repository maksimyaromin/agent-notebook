---
id: task.notebook-location-and-commit-policy-are
type: task
state: closed
title: Notebook location and commit policy are the user's call
by: Maksim Yaromin
via: claude-code
from: decision.close-proofs-are-equals-default-note
link: sha 2ed0816
priority: 3
created: 2026-08-29
updated: 2026-08-29
closed: 2026-08-29
---

Users must stay free to put the notebook where they want and commit it or not: at the repo root and committed, or relocated (for example into a git-ignored scratch area) and private — the way a personal backlog file works. Today the notebook path is fixed. Give it a configurable location honored by every command, and make sure nothing in the tool assumes the notebook is committed. Acceptance: a relocated notebook passes the full verb cycle; docs state the location and commit policy are configuration, not doctrine.
- 2026-08-29 Maksim Yaromin: Precedence: --notebook > ANB_NOTEBOOK > discovery. Review caught that a relative ANB_NOTEBOOK joined onto the working directory, so one export forked the notebook per directory — a subdirectory silently got its own. It now anchors on the project (the ancestor holding a notebook or .git); the flag stays cwd-relative, since it is typed with a known cwd. Also from review: the flag had no test at any surface, and anb --help still said the notebook lives in the repository.
