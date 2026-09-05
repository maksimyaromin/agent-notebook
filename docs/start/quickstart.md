---
title: Quickstart
description: 'Install anb, wire a project, and run one session from the first record to a clean check.'
---

## Install

```sh
npx -y @supolka/agent-notebook --help
```

The package ships a binary for your platform. With a Rust toolchain you can build it instead:

```sh
cargo install --git https://github.com/maksimyaromin/agent-notebook anb
```

Either way the command is `anb`.

## Wire the project

In the project's root:

```
$ anb setup
ok: setup — 18 files
  AGENTS.md: written
  CLAUDE.md: written
  .claude/settings.json: written
  .codex/hooks.json: written
  .claude/skills/anb/SKILL.md: written
  ...
notice: Codex runs a project hook after you review it: run /hooks in Codex from this directory
```

Setup writes one line into `AGENTS.md` and `CLAUDE.md`, a `SessionStart` hook for Claude Code and Codex that runs `anb status --hook`, and the two skills where each agent looks for them. Re-running patches in place; `anb setup --remove` takes out only what setup put in. [Wiring agents](../guides/agents.md) explains each file. You can skip this step: the notebook appears on the first record you add.

## One session

A Task, started and logged:

```
$ anb add task "Parser accepts fenced bodies" --tag parser
ok: add task.parser-accepts-fenced-bodies — tasks/task.parser-accepts-fenced-bodies.md

$ anb start task.parser-accepts-fenced-bodies
ok: start task.parser-accepts-fenced-bodies — open→active

$ anb comment task.parser-accepts-fenced-bodies "fences parse; the indented-body case is next"
ok: comment task.parser-accepts-fenced-bodies — logged
```

A doubt met on the way, filed with its origin, and the ruling that settles it:

```
$ anb add question "Do fences nest?" --from task.parser-accepts-fenced-bodies
ok: add question.do-fences-nest — questions/question.do-fences-nest.md

$ anb add decision "Fences never nest" --kind rule --body "A fence closes at the first closing marker."
ok: add decision.fences-never-nest — decisions/decision.fences-never-nest.md

$ anb close question.do-fences-nest --resolved-by decision.fences-never-nest
ok: close question.do-fences-nest — open→closed
resolved-by: decision.fences-never-nest

$ anb archive question.do-fences-nest
ok: archive question.do-fences-nest — questions/question.do-fences-nest.md→archive/questions/question.do-fences-nest.md
```

What the next session opens with:

```
$ anb status
ok: notebook — 1 tasks, 1 decisions, 0 notes, 0 questions
active: task.parser-accepts-fenced-bodies "Parser accepts fenced bodies"
log: "- 2026-09-05 Alex: fences parse; the indented-body case is next"
rules[1]:
  decision.fences-never-nest: "Fences never nest"
budget: ~83/1500 tokens
```

The work closes with its report ingested as a Note, and the archive carries the Note along:

```
$ echo "The parser accepts fenced bodies; the indented case is covered by the corpus." > report.md

$ anb close task.parser-accepts-fenced-bodies --note report.md
ok: close task.parser-accepts-fenced-bodies — active→closed
report: note.report-parser-accepts-fenced-bodies

$ anb archive task.parser-accepts-fenced-bodies
ok: archive task.parser-accepts-fenced-bodies — tasks/task.parser-accepts-fenced-bodies.md→archive/tasks/task.parser-accepts-fenced-bodies.md
carried[1]: note.report-parser-accepts-fenced-bodies

$ anb check
count: 0
```

That is the whole loop: [the session](../guides/session.md), [tasks](../guides/tasks.md) and [knowledge](../guides/knowledge.md) each take one part of it further.
