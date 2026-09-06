---
title: Wiring agents
description: 'Install the notebook instructions, session-start hooks and skills, or adapt them to your workflow.'
---

Run `anb setup --agent <name>` in the project root after installing `anb` on your PATH, naming each agent that works in the repository. Setup installs the instructions and skills that agent reads, and its session-start hook where the host runs one. Run it again after an upgrade to refresh the managed files.

```sh
anb setup --agent claude-code
anb setup --agent claude-code --agent codex
```

The reply lists each file and whether it was written, already present, or left alone, and names the agents it skipped. Without `--agent`, setup refuses and names the agents it knows: files for a tool the project does not run read as noise at best and as a commitment at worst. Setup does not create the notebook itself; the first `anb add` does that.

## What setup writes

| Agent | Instructions | Hook | Skills |
|---|---|---|---|
| `claude-code` | `CLAUDE.md`, or `AGENTS.md` when `CLAUDE.md` links or imports it | `.claude/settings.json`: a `SessionStart` hook running `anb status --hook` with a 15-second timeout | `.claude/skills/anb/`, `.claude/skills/anb-atlas/` |
| `codex` | `AGENTS.md` | `.codex/hooks.json`: the corresponding Codex hook | `.agents/skills/anb/`, `.agents/skills/anb-atlas/` |
| `agents-md` | `AGENTS.md` | none | `.agents/skills/anb/`, `.agents/skills/anb-atlas/` |

`agents-md` stands for any tool that follows the agents.md convention and reads skills from `.agents/skills`; a file two agents share is written once. The instructions are one marked block pointing to `anb status` and `anb --help`, appended as a paragraph of its own. The `anb` skill holds the workflow, the command and refusal references, the worked session and the planning and domain-modeling guidance; `anb-atlas` is the skill for drawing and reviewing the notebook.

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

`anb setup --agent <name> --remove` removes the managed instructions, hooks and skill files of the named agents. A file two agents read goes only when every agent that reads it is named, and the reply says which it kept: `AGENTS.md` and `.agents/skills` need `codex` and `agents-md` named together, and `claude-code` too when `CLAUDE.md` links or imports `AGENTS.md`. Other tools' instruction text and hook groups remain, a skill directory is removed only if it is empty, and a host directory that held nothing but setup's files goes with them. The notebook records are not removed.
