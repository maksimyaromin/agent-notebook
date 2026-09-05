---
id: task.agents-md-rewritten-for-a-public
type: task
state: closed
title: AGENTS.md rewritten for a public repository, free of local dependencies
by: Maksim Yaromin
via: claude-code
from: task.release-gate-v1
tags: docs
link: note note.report-agents-md-rewritten-for-a-public
blocked-by: task.skills
priority: 2
created: 2026-09-05
updated: 2026-09-05
closed: 2026-09-05
---

Today AGENTS.md is the owner's working protocol: it points into .tmp/, names the engineering instruction and skills that live on the owner's machine, carries dated rulings and the marathon procedure. A stranger cloning the public repository can follow none of it. Before the repository goes public the file is rewritten by the best practices for agent instructions: only what an agent needs to work in this repository from a clean clone — layout, the gate, the notebook protocol through the shipped skill, conventions that bind committed text — and nothing that depends on a path outside the repository. Provenance and the owner's process move to .tmp/ or into the notebook. Acceptance: no path under .tmp/ or the home directory is named; every command in the file runs from a fresh clone; the file reads as documentation, not as a diary.
- 2026-09-05 Maksim Yaromin: AGENTS.md rewritten in the shape of a public instruction file: what the project is, how a change is verified, the notebook protocol through the shipped skill, the layout, a conventions table naming the page that owns each rule, the reading order. No path under .tmp or the home directory. The owner's process (working documents, engineering instruction, task protocol, marathon mode, Lavish and the rest) moved to CLAUDE.local.md, git-ignored and loaded by Claude Code beside AGENTS.md. Fresh-clone command run and smoke check in progress.
- 2026-09-05 Maksim Yaromin: Smoke check: two rules attributed to pages that did not state them (now stated on the development page), the one-Task sentence made honest, two nits. Report at .tmp/docs/report-agents-md.md.
