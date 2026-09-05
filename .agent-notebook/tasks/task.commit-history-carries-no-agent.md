---
id: task.commit-history-carries-no-agent
type: task
state: open
title: Commit history carries no agent attribution before the repository goes public
by: Maksim Yaromin
via: claude-code
from: task.release-gate-v1
tags: dist
blocked-by: task.skills
priority: 2
created: 2026-09-05
updated: 2026-09-05
---

The owner's ruling: this is their code and their ownership, and no commit in the public history mentions an agent as co-author or generator. Today 84 lines across 74 commits carry Co-Authored-By or generated-with trailers. From 2026-09-05 no new commit or pull request body carries one. For the existing history: a rewrite script that strips exactly those trailer lines and nothing else, verified on a scratch clone by diffing every tree against the original, with the exact force-push the owner runs. Rewriting main is the owner's hand: the marathon leaves the script, the verification and the instruction.
