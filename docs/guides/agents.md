---
title: Wiring agents
description: 'Install the notebook instructions, session-start hooks and skills, or adapt them to your workflow.'
---

Run `anb setup` in the project root after installing `anb` on your PATH. Setup installs instructions and skills so agents can use the notebook, plus session-start hooks for Claude Code and Codex. Run it again after an upgrade to refresh the managed files.

```sh
anb setup
```

The reply lists each file and whether it was written, already present, or left alone. Setup does not create the notebook itself; the first `anb add` does that.

## What setup writes

| File | Contents |
|---|---|
| `AGENTS.md` | A marked instruction block pointing to `anb status` and `anb --help` |
| `CLAUDE.md` | The same block, unless the file already links to or imports `AGENTS.md` |
| `.claude/settings.json` | A `SessionStart` hook running `anb status --hook` with a 15-second timeout |
| `.codex/hooks.json` | The corresponding Codex hook |
| `.claude/skills/anb/`, `.agents/skills/anb/` | The workflow skill, command and refusal references, worked session, planning and domain-modeling guidance |
| `.claude/skills/anb-atlas/`, `.agents/skills/anb-atlas/` | The skill for drawing and reviewing the notebook |

Setup adds its hook beside existing hook groups. It validates the files before writing: invalid JSON or an unmatched instruction marker causes a refusal before those changes are applied. Symlinks are reported and left alone, including an existing `CLAUDE.md` link to `AGENTS.md`.

## Enable the session hook

For Codex, trust the project in its configuration and review the installed hook with `/hooks`. Setup prints a reminder. Claude Code uses the hook in `.claude/settings.json`.

The hook gives the model Status as additional context. If the notebook root cannot be read, it returns an empty payload and exits successfully. Hook failure therefore does not prevent the session from starting. To inspect the notebook directly, run `anb status`.

For an agent without a compatible hook, make `anb status` the opening command in its instructions. An agent that reads `AGENTS.md` can find that command there. Point it to the installed `anb` skill for the workflow; the CLI itself needs only shell access.

## The supplied workflow

The `anb` skill teaches agents to capture a new request as an idea, develop it through questions and evidence, and decompose work when its intended result is understood. It also teaches agents to resume work, maintain domain knowledge and close Tasks with proof before archiving. It keeps one Task in flight, groups larger work into epics, and requires a reason for a hold. By default, the agent checks the notebook before stopping and commits it with the code it describes. Maintaining the notebook is part of the agent's work. Planning and domain modeling belong to the main skill, alongside record selection, a worked example and common mistakes. Command syntax stays in the command reference; the main skill explains which record or action to choose and asks agents to identify their tool with `--via` on creation and comments.

The binary generates the workflow skill and references. Command definitions also supply `--help`, and worked examples run against a scratch notebook during generation. CI compares the committed skill with that output. This checks command and example drift; the authored workflow guidance still needs review when behavior changes.

```sh
anb skill                       # print the skill
anb skill path/to/skill          # write it to a directory
anb skill path/to/skill --check  # fail if the directory differs
```

The `anb-atlas` skill is authored separately and bundled with the binary. It uses graph data to build an interactive page and routes review comments back through the CLI. See [Drawing the notebook](atlas.md).

## Customize or remove the integration

[Customizing the workflow](customization.md) gives complete recipes for a private notebook in `.tmp/xxx`, another storage location, custom skills and a review stage. Remove `managed-by: anb` from each skill file you maintain yourself; setup then reports it as `yours, left alone` during updates and removal.

`anb setup --remove` removes managed instructions, hooks and skill files. Other tools' instruction text and hook groups remain, and a skill directory is removed only if it is empty. The notebook records are not removed.
