<h1 align="center">agent notebooks</h1>

<p align="center">A project's working memory as typed records in plain files, read and written by any coding agent through one CLI.</p>

<p align="center"><a href="https://agent-notebook.supolka.dev"><b>Documentation</b></a></p>

<p align="center">
  <a href="https://github.com/maksimyaromin/agent-notebook/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/maksimyaromin/agent-notebook/actions/workflows/ci.yml/badge.svg" /></a>
  <a href="./LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-667aeb?style=flat-square" /></a>
</p>

An agent working in a repository loses everything between sessions that does not land in code: which task was in flight and where it stopped, the decisions that override the plan, the facts it learned about the project, the doubts it deferred. `anb` keeps that memory in the repository as a notebook: a directory of markdown files, one record each, committed with the code. Four kinds of record, each with a lifecycle the tool enforces: **Tasks** (open → active → review → closed), **Decisions** (active → superseded or retired), **Notes** (active → retired) and **Questions** (open → closed). Every record has an id you can type, such as `task.parser-accepts-fenced-bodies`, and a file a person can read in any editor. Agents never edit the files; they run commands, and every command answers in a few lines of plain text made to be parsed at a glance.

## Install

```sh
npx -y @supolka/agent-notebook --help
```

The package ships a binary for your platform, so there is nothing to install beyond Node. With a Rust toolchain you can build it yourself instead:

```sh
cargo install --git https://github.com/maksimyaromin/agent-notebook anb
```

Either way the command is `anb`. Run `anb setup` once in a project to wire the agents to it (see [Wiring agents](#wiring-agents)); or skip that and let the notebook appear on the first record you add.

## A session

The notebook appears on first write. A Task is created with its title; the id is minted from it.

```
$ anb add task "Parser accepts fenced bodies" --tag parser
ok: add task.parser-accepts-fenced-bodies — tasks/task.parser-accepts-fenced-bodies.md
```

Every reply starts with `ok:`, the verb, the id and what changed. State moves are commands, never edits:

```
$ anb start task.parser-accepts-fenced-bodies
ok: start task.parser-accepts-fenced-bodies — open→active

$ anb comment task.parser-accepts-fenced-bodies "fences parse; the indented-body case is next"
ok: comment task.parser-accepts-fenced-bodies — logged
```

The comment is the Task's log, and it is where the next session resumes. A doubt met on the way is filed with its origin instead of widening the Task; a ruling is a Decision with a kind, `rule`, `shape` or `drift`:

```
$ anb add question "Do fences nest?" --from task.parser-accepts-fenced-bodies
ok: add question.do-fences-nest — questions/question.do-fences-nest.md

$ anb add decision "Fences never nest" --kind rule --body "A fence closes at the first closing marker."
ok: add decision.fences-never-nest — decisions/decision.fences-never-nest.md
```

A Question closes into the record that settled it, and a settled record is archived so the working set stays what is in play:

```
$ anb close question.do-fences-nest --resolved-by decision.fences-never-nest
ok: close question.do-fences-nest — open→closed
resolved-by: decision.fences-never-nest

$ anb archive question.do-fences-nest
ok: archive question.do-fences-nest — questions/question.do-fences-nest.md→archive/questions/question.do-fences-nest.md
```

Status is what a session opens with: the active Task with its last log line, the standing rules, and a token budget it stays under.

```
$ anb status
ok: notebook — 1 task, 1 decision, 0 notes, 0 questions
active: task.parser-accepts-fenced-bodies "Parser accepts fenced bodies"
log: "- 2026-09-05 Alex: fences parse; the indented-body case is next"
rules[1]:
  decision.fences-never-nest: "Fences never nest"
budget: ~83/1500 tokens
```

Work closes with its proof. The default proof is a report ingested as a Note, because a Note travels with the notebook; a pull request, a commit or a file left in place are its equals, and `--no-proof` says there is nothing to show. Write the report first, then close with it:

```
$ echo "The parser accepts fenced bodies; the indented case is covered by the corpus." > report.md

$ anb close task.parser-accepts-fenced-bodies --note report.md
ok: close task.parser-accepts-fenced-bodies — active→closed
report: note.report-parser-accepts-fenced-bodies

$ anb archive task.parser-accepts-fenced-bodies
ok: archive task.parser-accepts-fenced-bodies — tasks/task.parser-accepts-fenced-bodies.md→archive/tasks/task.parser-accepts-fenced-bodies.md
carried[1]: note.report-parser-accepts-fenced-bodies
```

The archive carried the report along. `anb check` verifies every file and names what would repair each finding; a clean notebook answers with a count.

```
$ anb check
count: 0
```

A refusal names the code, the fact, and a command that runs as printed:

```
$ anb start task.nope
error[unknown-id]: no record `task.nope`
try: anb list
```

## The files

Each record is one markdown file: a fenced envelope of `key: value` lines the tool owns, then a body the tool never parses. The archived Task from the session above:

```markdown
---
id: task.parser-accepts-fenced-bodies
type: task
state: closed
title: Parser accepts fenced bodies
by: Alex
tags: parser
link: note note.report-parser-accepts-fenced-bodies
created: 2026-09-05
updated: 2026-09-05
closed: 2026-09-05
---
- 2026-09-05 Alex: fences parse; the indented-body case is next
```

`by` comes from the git identity of whoever ran the command. The layout under `.agent-notebook/` is `tasks/`, `decisions/`, `notes/`, `questions/`, and `archive/` with the same four below it. No lifecycle move deletes a file or frees an id; the archive keeps both. Only `anb delete`, for a record born by mistake, removes a record, and it refuses while anything cites it. You can edit a file by hand; `anb check` reads every file and names each line it cannot accept, so a hand edit never silently drops a record.

## The shape of work

Work is rarely a flat list. An idea gets a hub Task tagged `epic`; each Task born inside it is created `--from` the hub, and the hub is blocked by its children, so it becomes ready only when the last child closes.

```
$ anb add task "Ship the parser" --tag epic
ok: add task.ship-the-parser — tasks/task.ship-the-parser.md

$ anb add task "Negative corpus wired into CI" --from task.ship-the-parser
ok: add task.negative-corpus-wired-into-ci — tasks/task.negative-corpus-wired-into-ci.md

$ anb block task.ship-the-parser task.negative-corpus-wired-into-ci
ok: block task.ship-the-parser — waits on task.negative-corpus-wired-into-ci

$ anb ready
ready[1]{id,priority,age,title}:
  task.negative-corpus-wired-into-ci,-,0d,Negative corpus wired into CI
```

`ready` is the dispatch queue: open, unblocked, unheld, most urgent first. A cycle in `block` edges is refused when it is written, so the queue cannot silently empty forever. A pause carries its reason, and a paused Task leaves the queue and the active line:

```
$ anb hold task.negative-corpus-wired-into-ci --reason "waits for the corpus license"
ok: hold task.negative-corpus-wired-into-ci — held

$ anb status
ok: notebook quiet — 2 tasks, 1 decision, 0 notes, 0 questions. anb --help when needed.
```

Tables like `ready[1]{id,priority,age,title}:` name their columns once and print comma rows. Listings are bounded; `--all` lifts the bound, and `--json` on any command gives the same data as compact JSON.

## Wiring agents

One command wires a project for Claude Code, Codex, Pi and any agent that reads `AGENTS.md`:

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

| Written | What it does |
|---|---|
| one line in `AGENTS.md` and `CLAUDE.md`, between markers | tells the agent the notebook exists and how to read it |
| a `SessionStart` hook in `.claude/settings.json` and `.codex/hooks.json` | runs `anb status --hook` when a session starts, so the agent begins from the active Task |
| the `anb` skill under `.claude/skills/` and `.agents/skills/` | the method: resume, work, record, close with proof, archive, leave the notebook clean |
| the `anb-atlas` skill beside it | how to draw the notebook from `anb graph --json` into a page and turn the reader's comments into commands |

Re-running patches in place, and `anb setup --remove` takes out only what setup put in. A skill file you edit becomes yours: drop the `managed-by: anb` line from its frontmatter and setup leaves it alone from then on.

The `anb` skill is not written by hand. The binary renders it from the same definitions that print `--help` and by running every example on a scratch notebook, so what agents are taught cannot drift from what the tool does; `anb skill` prints it, and this repository's committed copy is checked against the rendering in CI.

## Your own notebook

`--global` names a notebook in your home directory for knowledge that outlives one repository. Decisions and Notes live there; Tasks and Questions are refused, because work stays in the project. A practice you want every agent to know is a global Note tagged `skill`, found by name with `anb search <name> --global`.

## Development

```sh
./scripts/check.sh
```

The gate runs formatting, clippy with warnings as errors, every test, the doctests, the rustdoc build with warnings as errors, and the skill drift check; CI runs the same script on every pull request. The workspace has two crates: `anb-core`, the record model, grammar and notebook behind a storage trait with no filesystem or git of its own, and `anb`, the command line around it. The project's own backlog lives in `.agent-notebook/` at the root and is read and written only through the tool.

## License

[MIT](./LICENSE)
