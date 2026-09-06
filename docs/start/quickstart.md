---
title: Quickstart
description: 'Install anb and use the supplied workflow, from the first Task to the next session.'
---

This guide uses the defaults: a notebook in your repository, the supplied skills, and memory committed with the code. The npm installation below needs Node.js 20 or later. Your coding agent needs shell access to run `anb`.

## Install and set up

Install the CLI:

```sh
npm install -g @supolka/agent-notebook
```

In the root of the repository where you want to work, run:

```sh
anb setup
```

Setup adds notebook instructions to `AGENTS.md` and `CLAUDE.md`, installs the `anb` and `anb-atlas` skills, and configures session-start hooks for Claude Code and Codex. It reports each file it writes or leaves alone. [Wiring agents](../guides/agents.md) lists the files.

For Codex, trust the project and review the hook with `/hooks`. Start a new agent session in this repository so it can load the installed instructions and skills. With a compatible hook, Status arrives at session start. For another agent, ask it to read the installed `anb` skill and begin with `anb status`.

Setup creates the integration files, but no records. You do not need to initialize the notebook or create a configuration file.

## Give the agent work

For example:

> Use the anb skill to add support for fenced parser bodies. Keep the work in the notebook.

The first record creates `.agent-notebook/` at the repository root. Each Task, Decision, Note and Question is a Markdown file under that directory. The agent changes records through `anb`; you can read them in your editor or inspect their diffs.

The supplied skill defines this working cycle:

| During the work | What the agent does |
|---|---|
| Starting | Resumes the active Task, or selects ready work when none is active. It keeps one Task in flight. |
| Understanding a new request | Captures an idea with its source, clarifies the intended outcome, and records questions and evidence. Creates a spec or domain model when the work needs one. |
| Planning delivery | Creates a hub Task from the idea, child Tasks from the hub, and dependencies where one result needs another. |
| Making progress | Logs what it established and what comes next. |
| Learning | Records rulings as Decisions, reusable knowledge as Notes, and uncertainties as Questions with their origin. |
| Resolving or pausing | Closes answered Questions. Holds a paused Task with a reason. |
| Finishing | Closes the Task with proof, normally a report imported as a Note, then archives the Task and report. |
| Ending the session | Leaves `anb check` clean and commits the notebook with the code it describes. |

These are instructions to the agent. The CLI validates record changes; it does not implement your feature, assess the report, or run git commits itself. Your instructions and the agent's permissions still govern the work.

## Pick it up next session

Ask the agent to continue the Task or epic. It reads Status and the active Task's latest log entry to find where work stopped. If the Task is finished, it selects the next ready one. Dependencies keep blocked work out of that queue.

Closed work stays in `.agent-notebook/archive/`, including its reports. `anb search` and `anb show` can still read it. With the notebook committed, another collaborator or another clone has the same records. An agent in a new checkout also needs `anb` installed and the project's skills and hook enabled.

For a visual review, ask:

> Use anb-atlas to show this epic and what is blocking it.

The agent builds an HTML map. Open a record beside it, leave comments, and return them to the agent to apply through notebook commands. [Drawing the notebook](../guides/atlas.md) explains the review loop.

## Change the defaults when you need to

The default workflow is ready to use without customization. If you prefer private notes, another storage location, or a different working method, [follow the customization recipes](../guides/customization.md). The global notebook is separate and optional; use it for [personal knowledge across projects](../guides/your-own-notebook.md).

## Other installation options

To try the CLI without installing globally:

```sh
npx -y @supolka/agent-notebook --help
```

For subsequent commands, replace `anb` with `npx -y @supolka/agent-notebook`. To use the session hooks as installed by setup, install `anb` on your PATH.

[GitHub releases](https://github.com/maksimyaromin/agent-notebook/releases) also provide native binaries for macOS and Linux on x64 and arm64, and Windows on x64, with a `SHA256SUMS` file. Extract the archive for your platform, verify its checksum, and put `anb` on your PATH. This installation does not require Node.js.

With a Rust toolchain, you can install from source instead:

```sh
cargo install --git https://github.com/maksimyaromin/agent-notebook anb
```

## The same workflow through the CLI

The agent uses these commands to maintain the notebook. You can run this example yourself to inspect the records and replies; you do not need to run it before giving your agent work.

Create a Task, start it, and log enough detail for someone else to continue:

```
$ anb add task "Parser accepts fenced bodies" --tag parser
ok: add task.parser-accepts-fenced-bodies — tasks/task.parser-accepts-fenced-bodies.md

$ anb start task.parser-accepts-fenced-bodies
ok: start task.parser-accepts-fenced-bodies — open→active

$ anb comment task.parser-accepts-fenced-bodies "fences parse; the indented-body case is next"
ok: comment task.parser-accepts-fenced-bodies — logged
```

Suppose the implementation raises a question about nested fences. Record it with the Task as its origin. Once you decide the rule, record the Decision and close the Question against it:

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

Run Status to see what another session would receive. The date and author in your output reflect your own run:

```
$ anb status
ok: notebook — 1 task, 1 decision, 0 notes, 0 questions
active: task.parser-accepts-fenced-bodies "Parser accepts fenced bodies"
log: "- 2026-09-05 Alex: fences parse; the indented-body case is next"
rules[1]:
  decision.fences-never-nest: "Fences never nest"
budget: ~82/1500 tokens
```

When the work is finished, write a report of the result and its verification. This example uses a short report; a real one should contain enough evidence to assess the work. `--note` imports it into the notebook, and archiving the Task archives its report too:

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

`count: 0` means the check found no problems. The supplied workflow commits `.agent-notebook/` with the code so collaborators and future clones share its history. For larger work, [group Tasks into an epic](../guides/tasks.md#hubs-and-epics); for subsequent sessions, [resume from Status](../guides/session.md).
