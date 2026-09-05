---
title: Wiring agents
description: 'What anb setup writes for Claude Code, Codex, Pi and any agent that reads AGENTS.md, and where the skills come from.'
---

One command wires a project. Run it in the project's root, and run it again after an upgrade: it patches in place.

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

## What it writes

| File | What setup puts there | Who reads it |
|---|---|---|
| `AGENTS.md` | one descriptive line between markers: the notebook exists, `anb status` shows its state, `anb --help` the commands | Codex, Pi, OpenCode and any agent that reads `AGENTS.md` |
| `CLAUDE.md` | the same line, unless the file already links `AGENTS.md` or imports it with a bare `@AGENTS.md` line | Claude Code, which reads `CLAUDE.md` and not `AGENTS.md` |
| `.claude/settings.json` | a `hooks.SessionStart` group running `anb status --hook` with a 15-second timeout, beside any group another tool has | Claude Code |
| `.codex/hooks.json` | the same group in the same shape | Codex, after you trust the project and review the hook with `/hooks` |
| `.claude/skills/anb/`, `.agents/skills/anb/` | the `anb` skill: `SKILL.md` and three references | Claude Code from `.claude/skills/`; Codex and Pi from `.agents/skills/` |
| `.claude/skills/anb-atlas/`, `.agents/skills/anb-atlas/` | the atlas skill: drawing the notebook and the intent loop | the same |

Every file is read and judged before the first is written, so a refusal leaves the project as it was: a settings file that is not JSON, or an instruction file with a stray marker, refuses the whole run. A file that is a link to somewhere else is left alone and reported. Each file's line says what happened to it: `written`, `already`, `removed`, `absent`, `a link, left alone`, `yours, left alone`, or for a `CLAUDE.md` that already reaches `AGENTS.md`, `links AGENTS.md` or `imports AGENTS.md`.

`anb setup --remove` takes out what setup put in and nothing else: other tools' lines and hook groups survive, and a skill directory goes only when setup emptied it.

## The hook

The `SessionStart` hook runs `anb status --hook`, which prints the Status wrapped as additional context for the model and fails soft: a notebook root the tool cannot read yields an empty payload and a zero exit. Claude Code runs a project hook as soon as it is in `.claude/settings.json`. Codex runs a project hook only after the project is trusted in its configuration and the hook is reviewed with `/hooks`; setup prints that notice, since the step is yours.

## The skills

The `anb` skill teaches the method: open from Status, resume the active Task or take the next ready one, log as you go, file every friction as a record, close with a proof, archive in the same breath, close Questions when they settle, hold with a reason, leave the notebook clean. Its three references carry the depth an agent opens on demand: every command with its flags, a worked session with every reply as printed, and every refusal code with its repair.

The skill is not written by hand. The binary renders `SKILL.md` and the references from the same definitions that print `--help`, and produces every example by running the command on a scratch notebook, so what agents are taught cannot drift from what the tool does. `anb skill` prints it; `anb skill <dir>` writes it; `anb skill <dir> --check` compares a directory with the rendering and fails on any difference, which is how this repository's committed copy is held in CI.

The atlas skill is written by hand and carried in the binary. It teaches an agent to draw a notebook from `anb graph --json` into one page and to turn the reader's comments on that page into commands; [Drawing the notebook](atlas.md) is the summary.

A skill file you edit becomes yours. Both skills carry `managed-by: anb` in their frontmatter; drop that line and setup reports the file as `yours, left alone`, rewriting nothing and removing nothing from then on. A team that works differently overrides the process this way, or writes its own skill and drives the same commands; the CLI carries no opinion.
