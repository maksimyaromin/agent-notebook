---
id: task.skill-from-help-ci-drift-check
type: task
state: open
title: Skill from help + CI drift check
by: Maksim Yaromin
via: claude-code
tags: cli
blocked-by: task.setup-sessionstart-hooks
blocked-by: task.github-dev-flow-actions-ci-fmt-clippy-te
created: 2026-08-29
updated: 2026-08-30
---

Skill generated from the same source as CLI help; CI-checked against drift.
- 2026-08-29 claude-code: Owner ruling (m1 review): the epic/hub convention lives in the generated skill, nowhere else. The skill must teach: tasks are almost never a flat sheet — an idea gets a hub Task (tag epic), every task born inside it carries --from <hub> at add time (origin is add-time only; no retrofit before the edit surface), the hub is blocked by its children and closes when the last one does. The founding hub is task.anb-v1. Teach this as the default shape of task creation, not an option.
- 2026-08-29 Maksim Yaromin: Carried from the epic-pattern task, since the generated skill does not exist yet and this recipe would otherwise evaporate with .tmp/. The skill must teach: resolve continue <epic> by matching the hub id slug, else search over titles and tags; then take the active Task inside the hub's scope, else the top of ready --for <hub>; then start it. Scope is what the epic waits on plus what was born inside it, both followed transitively. And Tasks born inside an idea carry --from <hub> at add time: retrofitting Origin onto existing children needs an edit surface that does not exist, which is why the founding hub had to be assembled from blocked-by alone.
- 2026-08-30 Maksim Yaromin: Carried from task.global-skill-notes-docs, which waits on this one and whose deliverable is text this generator will emit. The skill must teach the Global Notebook: --global names the user's own notebook in the home directory, sharing the flag rung with --notebook so it outranks ANB_NOTEBOOK and naming both is refused; it holds knowledge that outlives one repository, so the verbs that create or move a task or a question refuse it, and every read and every knowledge write behaves the same in either scope. And the convention the scope exists for: a practice is recorded as a global Note tagged skill and addressed by name, so 'use my global skill X' resolves through anb instead of through a pasted absolute path. Also worth teaching: a project rule that stands against one of the user's says so by citing its id in its body, which is what makes the shadow pair surface in Status.
