---
id: task.skills
type: task
state: open
title: Skills: everything the tool teaches an agent (hub)
by: Maksim Yaromin
from: task.anb-v1
tags: epic
blocked-by: task.setup-sessionstart-hooks
blocked-by: task.skill-from-help-ci-drift-check
blocked-by: task.global-skill-notes-docs
blocked-by: task.claude-code-skill-the-notebook-drawn-and
created: 2026-08-31
updated: 2026-08-31
---

Epic hub uniting every skill the tool ships in one form or another: setup installing the AGENTS.md snippet and SessionStart hooks; the skill generated from the same source as CLI help, CI-checked against drift; the Claude Code skill that draws the notebook and takes decisions in the page; the global skills-as-notes convention. Done when an agent that has never seen this repository learns the tool from the installed skills alone. Carries the remainder of the closed Global Notebook epic: a global Decision and a global Note recorded from one repo are usable from another by one instruction to an agent. Ordered after task.improvements; the pre-release and release activities wait on this hub.
