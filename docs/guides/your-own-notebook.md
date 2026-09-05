---
title: Your own notebook
description: 'Keep personal rules and practices across repositories with the global notebook.'
---

Some practices belong to you across projects: how you review a change, how you name things, or what a good handoff includes. `--global` stores these in `.agent-notebook/` in your home directory. It uses the same record format and CLI as a project notebook.

```text
$ anb add note "Review in two passes" --kind guide --tag skill --global --body "First the diff against the tests, then the tests against the promise."
ok: add note.review-in-two-passes — notes/note.review-in-two-passes.md
```

## Keep knowledge here, work in the project

The global notebook accepts Decisions and Notes. Tasks and Questions belong to the project where the work happens:

```text
$ anb add task "Tidy the desk" --global
error[invalid-argument]: add: the user's notebook holds decisions and notes — work stays in the project
try: anb add task "<title>"
```

Knowledge commands work in either scope. Add `--global` to create, read, correct or retire a personal record.

## Find a practice by name

Tag a guide Note with `skill` when you want agents to find it as a reusable practice:

```text
$ anb search review --global
matches[1]{id,state,priority,title}:
  note.review-in-two-passes,active,-,Review in two passes
```

Read it with `anb show note.review-in-two-passes --global`. Tell your agent to use the named practice; the supplied skill teaches it to resolve that request through the global notebook. The Note is not automatically loaded into every session.

## Make project exceptions explicit

When a project Decision overrides a global Decision, cite the global id in the project's record. Status reads the global notebook and reports the pair as `shadow` Debt. The agent can then see the project exception and the personal rule together. The tool detects the citation, not a semantic disagreement, and leaves the global Decision unchanged.

## Choose a notebook location

Without an override, `anb` walks up from the working directory to the nearest existing `.agent-notebook/` or repository root. A new notebook is created at that root on the first record write, so commands work from project subdirectories too. Outside a repository, with no existing notebook above it, the working directory becomes the root.

`ANB_NOTEBOOK` selects a notebook for the shell; relative values are anchored to the project. `--notebook <path>` selects one for a single call. Both `--notebook` and `--global` take precedence over the environment variable, and the two flags cannot be combined.

You can commit a project notebook, ignore it in git, or store it elsewhere. The global notebook stays in your home directory unless you arrange to share its files yourself.

For a project notebook that stays private, follow [the `.tmp/xxx` recipe](customization.md#keep-project-memory-private). It keeps Tasks and Questions in project scope while excluding their files from git.
