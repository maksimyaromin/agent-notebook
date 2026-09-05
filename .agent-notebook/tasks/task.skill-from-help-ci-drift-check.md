---
id: task.skill-from-help-ci-drift-check
type: task
state: review
title: Skill from help + CI drift check
by: Maksim Yaromin
via: claude-code
tags: cli
blocked-by: task.setup-sessionstart-hooks
blocked-by: task.github-dev-flow-actions-ci-fmt-clippy-te
created: 2026-08-29
updated: 2026-09-05
---

Skill generated from the same source as CLI help; CI-checked against drift.
- 2026-08-29 claude-code: Owner ruling (m1 review): the epic/hub convention lives in the generated skill, nowhere else. The skill must teach: tasks are almost never a flat sheet — an idea gets a hub Task (tag epic), every task born inside it carries --from <hub> at add time (origin is add-time only; no retrofit before the edit surface), the hub is blocked by its children and closes when the last one does. The founding hub is task.anb-v1. Teach this as the default shape of task creation, not an option.
- 2026-08-29 Maksim Yaromin: Carried from the epic-pattern task, since the generated skill does not exist yet and this recipe would otherwise evaporate with .tmp/. The skill must teach: resolve continue <epic> by matching the hub id slug, else search over titles and tags; then take the active Task inside the hub's scope, else the top of ready --for <hub>; then start it. Scope is what the epic waits on plus what was born inside it, both followed transitively. And Tasks born inside an idea carry --from <hub> at add time: retrofitting Origin onto existing children needs an edit surface that does not exist, which is why the founding hub had to be assembled from blocked-by alone.
- 2026-08-30 Maksim Yaromin: Carried from task.global-skill-notes-docs, which waits on this one and whose deliverable is text this generator will emit. The skill must teach the Global Notebook: --global names the user's own notebook in the home directory, sharing the flag rung with --notebook so it outranks ANB_NOTEBOOK and naming both is refused; it holds knowledge that outlives one repository, so the verbs that create or move a task or a question refuse it, and every read and every knowledge write behaves the same in either scope. And the convention the scope exists for: a practice is recorded as a global Note tagged skill and addressed by name, so 'use my global skill X' resolves through anb instead of through a pasted absolute path. Also worth teaching: a project rule that stands against one of the user's says so by citing its id in its body, which is what makes the shadow pair surface in Status.
- 2026-09-05 Maksim Yaromin: Owner ruling (2026-09-05): the workflow the generated skill teaches aims at maximum efficiency, a clean notebook, and the least possible oversight from the developer. Hygiene the developer would otherwise have to police by hand is the skill's job: closed records archived in the same move as the close, reports ingested as Notes, Questions routed, holds carrying their reason, no unarchived leftovers for the owner to find — the day the owner must check cleanliness after every session is the day the experience is bad. Users who disagree with this opinionated process simply do not install the skill, or override it with their own and drive the CLI as they like; the CLI stays neutral, the skill carries the opinion.
- 2026-09-05 Maksim Yaromin: Generator done: anb skill [DIR] [--check] renders SKILL.md plus three references (commands.md from the clap tree, session.md and refusals.md by running every example on an in-memory notebook on a fixed day); anb setup installs them into .claude/skills/anb and .agents/skills/anb, rewriting only files still carrying the generated mark; scripts/check.sh diffs the committed copy. Owner note mid-work: one 590-line reference is not progressive disclosure — split into one file per need, each named in SKILL.md with a consult-before-acting trigger. Gate green; smoke check next.
