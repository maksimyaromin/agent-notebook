---
id: task.global-skill-notes-docs
type: task
state: closed
title: Global skills: docs and skill teach the phrase
by: Maksim Yaromin
via: claude-code
from: task.global-notebook
link: sha 1e90c72e442da84323499b3d4b430b8a9d0329d7
blocked-by: task.global-root-and-flags
blocked-by: task.skill-from-help-ci-drift-check
created: 2026-08-29
updated: 2026-09-05
closed: 2026-09-05
---

The generated skill and help teach the Global Notebook and the skills-as-notes convention: a global Note tagged skill is addressed by name, so 'use the global skill X' resolves through anb without a file path.
- 2026-08-30 Maksim Yaromin: Not started, and not startable inside this marathon's scope: it waits on task.skill-from-help-ci-drift-check, which is Track A and the maintainer reserved Track A and B for themselves. Its deliverable is the generated skill's text, and there is no generator yet. The help half shipped with task.global-root-and-flags: --global's own help now carries its precedence and its restriction, and AGENTS.md names the third way a notebook root is chosen. What is still owed is the skills-as-notes convention itself — a global Note tagged skill, addressed by name, so 'use my global skill X' resolves through anb with no path pasted — which nothing today teaches. Logged onto the skill task so the recipe does not evaporate with this session.
- 2026-09-05 Maksim Yaromin: Fulfilled by the generated skill: its section 'The user's own notebook' teaches --global as the user's notebook in the home directory, the refusal of Tasks and Questions there, the skills-as-notes convention addressed by name through anb search/show --global, and the shadow pair a project rule surfaces by citing the user's id. The help half shipped earlier with the global root. Proof is the merge that shipped the skill text.
