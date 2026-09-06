---
id: task.setup-choose-agents
type: task
state: open
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
