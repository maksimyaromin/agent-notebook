---
id: note.report-skill-layout-references-in-their
type: note
state: retired
title: Report: Skill layout: references in their own directory, SKILL.md in the standard sections
by: Maksim Yaromin
from: task.skill-layout-references-in-their-own
created: 2026-09-05
updated: 2026-09-05
---

# Skill layout: references in their own directory (2026-09-05)

Report for task.skill-layout-references-in-their-own, born from the maintainer's note during the marathon: the references were dumped at the skill root, and the shape skill writers converge on is `SKILL.md` at the root, the body in standard sections, the depth under `references/`.

## What changed

```
.agents/skills/anb/
  SKILL.md                  the method, read once per session
  references/
    commands.md             every verb with its flags, from the clap tree
    session.md              one notebook from empty to archived work, every reply as printed
    refusals.md             every refusal code with cause and repair, and the check findings
```

`SKILL.md` now opens with a `description` that says only when to use the skill — "Use when working in a repository that has an .agent-notebook directory …" — because that field is what an agent reads to decide whether to load it; a summary of the workflow there is noise at decision time. The body follows the standard sections: overview, when to use and when not, a quick reference table from situation to command, the method (the session, the Task loop, the shape of work, knowledge, the user's own notebook, before you stop), and a common-mistakes table that teaches by contrast. Each reference link names the moment to open it.

Every reference opens with a `## Contents` list of its sections at the shallowest heading level, so a partial read still shows the scope: the verbs in the commands reference, the session's stages (the hub and the work born inside it, the queue, a session at work, decisions and notes, closing a Question, status, closing a Task, ending without work and pausing, reading back and the gate), and the refusal codes, each now a heading over its example. The list is derived from the rendered body, never kept by hand.

`anb setup` writes the nested files and, on `--remove`, deletes them and the directories it emptied, `references/` and then the skill's own directory, and stops there: a directory holding anything else stays, and nothing above the skill's directory is touched. The deletion carries the directory it may empty, so the rule is not tied to the name `skills`.

## Tests

Unit: every reference opens with a contents list that is exactly its sections at the shallowest level, in order. End to end through the binary, unchanged in intent and moved to the nested paths: the skill is written, checked, found drifted by relative path; setup installs, re-runs idle, removes with the emptied directories; a stray file under the skill keeps its directory and removal never climbs above the skill's directory; a file the user made theirs is left alone, a nested reference included. The gate is green, drift check included.

## Review

One review pass, against the code and the running binary: no must-fix. Three should-fix, all taken: the module doc still said the references sit beside `SKILL.md`; the directory-safety rules held on the binary but had no test (three tests now: a stray file keeps its directory, removal never climbs above the skill directory, a nested reference made the user's is left alone); the contents-list test checked only that listed sections exist, and a mutation dropping a heading from the list stayed green (the test now demands the list equal the body's sections). One nit taken too: the climb stopped at a directory literally named `skills`, coupling the rule to the layout; the deletion now carries the directory it may empty. The check confirmed every command in the quick reference and common mistakes tables against `--help`, and the rendered replies byte for byte.
