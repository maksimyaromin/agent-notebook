---
id: task.setup-choose-agents
type: task
state: review
title: setup chooses which agents to wire
by: Maksim Yaromin
via: claude-code
from: task.release-0-3-0
link: issue https://github.com/maksimyaromin/agent-notebook/issues/42
priority: 1
created: 2026-09-06
updated: 2026-09-06
---

anb setup --agent <name>, repeatable, chooses the hosts before the first write; without the flag setup refuses with try lines naming it; --remove takes the same flag. The reply lists the hosts it skipped. The open mapping question is which agents read .agents/skills. Evidence: setup tests per host; the agents guide, the quickstart and the README show the flag; the commands reference regenerated.
- 2026-09-06 claude-code: --agent <name>, repeatable, chooses the hosts: claude-code, codex, agents-md for the agents.md convention; a file two share is planned once; bare setup and an unknown name refuse with try lines naming the three; --remove takes the flag and a host directory setup emptied goes with its files (one existing test's promise reversed, stated in the PR); five new end-to-end tests shown red once; agents guide, quickstart, README and customization updated; the ruling recorded as decision.setup-wires-only-the-agents-named; gate and docs check green; smoke check next
- 2026-09-06 claude-code: smoke check: one must-fix taken (a shared file goes only when every agent that reads it is named; three tests pin it), one should-fix and two nits taken; gate and docs check green; submitting
