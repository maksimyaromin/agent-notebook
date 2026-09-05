---
id: task.skills
type: task
state: closed
title: Skills: everything the tool teaches an agent (hub)
by: Maksim Yaromin
from: task.anb-v1
tags: epic
blocked-by: task.setup-sessionstart-hooks
blocked-by: task.skill-from-help-ci-drift-check
blocked-by: task.global-skill-notes-docs
blocked-by: task.claude-code-skill-the-notebook-drawn-and
blocked-by: task.skill-layout-references-in-their-own
blocked-by: task.cli-help-and-reply-texts-pass-the
created: 2026-08-31
updated: 2026-09-05
closed: 2026-09-05
---

Epic hub uniting every skill the tool ships in one form or another: setup installing the AGENTS.md snippet and SessionStart hooks; the skill generated from the same source as CLI help, CI-checked against drift; the Claude Code skill that draws the notebook and takes decisions in the page; the global skills-as-notes convention. Done when an agent that has never seen this repository learns the tool from the installed skills alone. Carries the remainder of the closed Global Notebook epic: a global Decision and a global Note recorded from one repo are usable from another by one instruction to an agent. Ordered after task.improvements; the pre-release and release activities wait on this hub.
- 2026-09-05 Maksim Yaromin: Owner ruling 2026-09-05 on distribution: both skills are committed under .agents/skills/<name>/SKILL.md; a new anb skill subcommand prints the generated one, which is what CI diffs against; anb setup installs the skills into the target project beside the AGENTS.md snippet and the hooks, into each agent's own skills directory. The workflow the skills teach aims at the least oversight from the developer: hygiene is the agent's job, and anyone who disagrees overrides the skill rather than the CLI.
- 2026-09-05 Maksim Yaromin: Every child closed: setup with hooks, the generated skill and its layout, the atlas skill, the global-notebook text, the help-string sweep. The hub itself has nothing to show beyond its children's report Notes.
