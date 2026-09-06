---
id: decision.setup-wires-only-the-agents-named
type: decision
state: active
kind: shape
title: setup wires only the agents named
by: Maksim Yaromin
via: claude-code
created: 2026-09-06
updated: 2026-09-06
---

anb setup takes --agent <name>, repeatable, and writes the files of those agents alone: claude-code, codex, or agents-md for any tool that reads AGENTS.md and .agents/skills. Without the flag it refuses and names the three, on a terminal as anywhere else. Alternatives weighed: asking on a terminal, rejected because the reply contract has no interactive surface and the agents that run the tool never have a terminal, so one shape serves both; writing every host and documenting the deletion, rejected because a re-run after an upgrade brings the files back and the user fights the tool on every upgrade. The deciding constraint is the footprint a new adopter reads in git status: a file for a tool the project does not run reads as a commitment. The same rule reaches removal: --remove takes the flag and takes out the named agents' files, a file two agents read goes only when every agent that reads it is named, and a host directory holding nothing but setup's files goes with them.
