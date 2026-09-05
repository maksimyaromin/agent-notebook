---
id: task.skill-layout-references-in-their-own
type: task
state: review
title: Skill layout: references in their own directory, SKILL.md in the standard sections
by: Maksim Yaromin
from: task.skills
tags: cli
created: 2026-09-05
updated: 2026-09-05
---

Owner note during the marathon (2026-09-05): the references were dumped at the skill root, and the preferred shape is the one skill writers converge on — SKILL.md at the root with a description that says only when to use the skill, the body in the standard sections (overview, when to use, quick reference, the method, common mistakes), and every reference under references/ with a contents list at the top of any long one. Both skills anb ships follow this layout; anb setup installs and removes the nested files, and an emptied skill directory goes with them.
- 2026-09-05 Maksim Yaromin: Done in code: the three references render under references/ with a Contents list of their sections at the top (verbs, the session's stages, the refusal codes); SKILL.md carries a 'Use when' description and the standard sections — overview, when to use and when not, quick reference, the method, common mistakes; setup writes the nested files and, on --remove, takes out the emptied references/ and skills/anb directories and nothing above or beside them. Gate green; smoke check running.
- 2026-09-05 Maksim Yaromin: Smoke check taken (3 should-fix, 1 nit); owner's second ruling applied: skill prose passes the humanizer, help strings deferred to task.cli-help-and-reply-texts-pass-the. Report at .tmp/docs/report-skill-layout.md.
