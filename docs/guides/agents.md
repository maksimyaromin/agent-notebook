---
title: Wiring agents
description: 'Install the notebook instructions, session-start hooks and skills, or adapt them to your workflow.'
---

Run `anb setup --agent <name>` from anywhere in the project after installing `anb` on your PATH. Name each agent that works in the repository. Setup installs the instructions and skills that agent reads, and its session-start hook where the host runs one. It uses the nearest parent containing `.agent-notebook` or `.git` as the project root, or the current directory if neither exists. Run it again after an upgrade to refresh the managed files.

```sh
anb setup --agent claude-code
anb setup --agent claude-code --agent codex
```

The reply lists each file and whether it was written, already present, or left alone, and names the agents it skipped. Setup requires `--agent` and writes files for the named agents only. It does not create the notebook itself; the first `anb add` does that.

## What setup writes

| Agent | Instructions | Hook | Skills |
|---|---|---|---|
| `claude-code` | `CLAUDE.md`, or `AGENTS.md` when `CLAUDE.md` links or imports it | `.claude/settings.json`: a `SessionStart` hook running `anb hook` with a 15-second timeout | `.claude/skills/anb/`, `.claude/skills/anb-atlas/` |
| `codex` | `AGENTS.md` | `.codex/hooks.json`: the corresponding Codex hook | `.agents/skills/anb/`, `.agents/skills/anb-atlas/` |
| `agents-md` | `AGENTS.md` | none | `.agents/skills/anb/`, `.agents/skills/anb-atlas/` |

`agents-md` stands for any tool that follows the agents.md convention and reads skills from `.agents/skills`; a file two agents share is written once. The instructions are one marked paragraph identifying the memory folder and explaining that its Markdown can be read without the CLI or skill. It also points to `anb recall`, `anb --help` and the optional project workflow extension. The `anb` skill holds the workflow, the command and refusal references, the worked session and the planning and domain-modeling guidance; `anb-atlas` is the skill for drawing and reviewing the notebook.

Setup adds its hook beside existing hook groups. An exact legacy `anb status --hook` command is upgraded in place to `anb hook`; group settings and a hand-tuned timeout remain unchanged. Commands with custom arguments or shell operators belong to the project and are neither rewritten nor removed.

Setup validates files before writing: invalid JSON, wrong hook container types, an unmatched instruction marker or a linked parent directory causes a refusal before integration changes are applied. Individual file symlinks are reported and left alone, including an existing `CLAUDE.md` link to `AGENTS.md`. Setup never follows a linked integration directory outside the project.

Setup also writes `.anb-setup.json` at the project root. This receipt records which instruction files setup has handled; it contains no notebook records or personal preferences. Keep it with the integration files so updates can distinguish a first installation from instructions your project has replaced.

## Enable the session hook

For Codex, trust the project in its configuration and review the installed hook with `/hooks`. Setup prints a reminder. Claude Code uses the hook in `.claude/settings.json`.

The native hook gives the model Recall as structured additional context: current work, relevant project knowledge and scoped personal practices. It reads the host's `SessionStart` JSON from standard input, including `session_id`. An explicit `--session`, `ANB_SESSION` or `CODEX_THREAD_ID` takes precedence over that input. When Claude Code provides `CLAUDE_ENV_FILE`, the hook appends an `ANB_SESSION` export for subsequent Bash calls, so the conversation keeps the same focus after its opening hook.

A hook read failure produces diagnostic context and exits successfully, allowing the host session to continue. Failure is not reported as empty memory. Run `anb recall` to inspect the same memory directly; ordinary CLI reads retain their nonzero failure status.

For an agent without a compatible hook, make `anb recall` the opening command in its instructions. An agent that reads `AGENTS.md` can find that command there. Point it to the installed `anb` skill for the workflow; the CLI itself needs only shell access. Use `--session <name>` or `ANB_SESSION` when you want separate conversations to remember separate Tasks.

## The supplied workflow

The `anb` skill teaches agents to resume from a useful cue and record only information worth carrying into another session. It distinguishes a Task's progress from reusable knowledge, shared meaning from personal practices, and a local next step from an external system's source of truth. Planning and domain modeling are available when the work needs them; a small change does not require an idea record or a specification.

Each conversation can remember its own Task, while one person or a team works on several Tasks in parallel. A handoff states the current result, remaining uncertainty and next action. Completion records a short outcome with relevant evidence on the Task; a separate Note is for knowledge that should outlive it. The agent checks the notebook before stopping and commits records only with the user's authorization. Command syntax stays in the generated reference; the main skill explains which record or action to choose and asks agents to identify their tool with `--via` on creation and comments.

The binary generates the workflow skill and references. Command definitions also supply `--help`, and worked examples run against a scratch notebook during generation. CI compares the committed skill with that output. This checks command and example drift; the authored workflow guidance still needs review when behavior changes.

```sh
anb skill                       # print the skill
anb skill path/to/skill          # write it to a directory
anb skill path/to/skill --check  # fail if the directory differs
```

The `anb-atlas` skill is authored separately and bundled with the binary. It uses graph data to build an interactive page and routes review comments back through the CLI. See [Drawing the notebook](atlas.md).

## Customize or remove the integration

Write team-specific workflow rules in `.agents/anb.md`. The supplied skill reads that file in both host directories, and setup never creates, changes or removes it. Keep personal preferences in [your own notebook](your-own-notebook.md). [Customizing the workflow](customization.md) covers these boundaries, private storage and a review stage.

You can replace the marked block in `AGENTS.md` or `CLAUDE.md` with your own instructions, or delete the file. Setup remembers the installation and reports the unmarked file as `yours, left alone`; later upgrades do not append the block again. Existing `anb` skills also identify an installation made without a receipt. To let setup manage an instruction block again, restore its `<!-- anb:begin -->` and `<!-- anb:end -->` markers around the notebook instructions.

For a complete skill replacement, remove `managed-by: anb` from each file you maintain yourself. Setup preserves those files during updates and removal, while references that retain the marker continue receiving updates.

`anb setup --agent <name> --remove` removes the managed instructions, hooks and skill files of the named agents. A file two agents read goes only when every agent that reads it is named, and the reply says which it kept: `AGENTS.md` and `.agents/skills` need `codex` and `agents-md` named together, and `claude-code` too when `CLAUDE.md` links or imports `AGENTS.md`. Other tools' instruction text and hook groups remain, and setup removes only empty skill and host directories. The receipt remains while it protects shared files or project-owned instructions; otherwise setup removes it. Notebook records and `.agents/anb.md` remain untouched.
