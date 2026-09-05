---
id: task.commit-history-carries-no-agent
type: task
state: closed
title: Commit history carries no agent attribution before the repository goes public
by: Maksim Yaromin
via: claude-code
from: task.release-gate-v1
tags: dist
link: note note.report-commit-history-carries-no-agent
blocked-by: task.skills
priority: 2
created: 2026-09-05
updated: 2026-09-05
closed: 2026-09-05
---

The owner's ruling: this is their code and their ownership, and no commit in the public history mentions an agent as co-author or generator. Today 84 lines across 74 commits carry Co-Authored-By or generated-with trailers. From 2026-09-05 no new commit or pull request body carries one. For the existing history: a rewrite script that strips exactly those trailer lines and nothing else, verified on a scratch clone by diffing every tree against the original, with the exact force-push the owner runs. Rewriting main is the owner's hand: the marathon leaves the script, the verification and the instruction.
- 2026-09-05 Maksim Yaromin: Scripts under .tmp/release: scrub-history.sh (filter-repo or filter-branch, four line shapes deleted, nothing else) and verify-scrub.sh (trees, authors, dates, messages minus the trailers, none left). Verified on a fresh clone of origin: 86 commits, 80 lines gone, main 94a83b0 -> a32c29c. PR bodies #5 and #6 edited, originals kept. README.md carries the exact force-push with the lease; the push is the owner's. Smoke check running.
- 2026-09-05 Maksim Yaromin: Smoke check took the scripts apart (both backends, bash vs sh, the dropped remote, refs/original, the final-newline byte); one Perl transform now serves the rewrite and the verifier, both backends give main 4b1f041 from origin 71f2199 and the strict verifier passes on each. README.md carries the run and the exact push; the push is the owner's.
