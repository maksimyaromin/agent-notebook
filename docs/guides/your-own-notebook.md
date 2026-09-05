---
title: Your own notebook
description: 'The global notebook in your home directory: knowledge that outlives one repository, and practices found by name.'
---

Some knowledge is yours rather than the project's: how you like a review run, which tools you trust, the rule you apply in every repository. `--global` names a notebook for it, `.agent-notebook/` in your home directory, served by the same CLI.

```
$ anb add note "Review in two passes" --kind guide --tag skill --global --body "First the diff against the tests, then the tests against the promise."
ok: add note.review-in-two-passes — notes/note.review-in-two-passes.md
```

## What lives there

Decisions and Notes. Tasks and Questions are refused, because work stays in the project it belongs to:

```
$ anb add task "Tidy the desk" --global
error[invalid-argument]: add: the user's notebook holds decisions and notes — work stays in the project
try: anb add task "<title>"
```

Every read and every knowledge write behaves the same in either scope. `--global` shares a rung with `--notebook <path>`, so both outrank the `ANB_NOTEBOOK` variable and naming both in one call is refused.

## Practices as Notes

A practice you want every agent to know is a global Note tagged `skill`, addressed by name:

```
$ anb search review --global
matches[1]{id,state,priority,title}:
  note.review-in-two-passes,active,-,Review in two passes
```

Then `anb show note.review-in-two-passes --global`. "Use my review skill" resolves through anb, never through a pasted path.

## When a project rule stands against yours

A project Decision that overrides one of your global ones says so by citing the global id in its body. The tool reads both notebooks and names the pair in Status as a `shadow`, so the agent follows the project's ruling knowingly rather than by accident.

## Where a notebook is found

Without a flag, the nearest `.agent-notebook/` at or above the working directory is the notebook, so a command works from any subdirectory of a project. `ANB_NOTEBOOK` names one for a shell, anchored on the project so one export means one notebook whatever the current directory. `--notebook <path>` names one for a call. A notebook may sit beside the code and be committed, hide in a git-ignored corner, or live outside the repository: where it sits is configuration, not something the tool requires.
