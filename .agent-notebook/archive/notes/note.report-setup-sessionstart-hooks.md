---
id: note.report-setup-sessionstart-hooks
type: note
state: retired
title: Report: setup + SessionStart hooks
by: Maksim Yaromin
from: task.setup-sessionstart-hooks
created: 2026-09-05
updated: 2026-09-05
---

# setup + SessionStart hooks (2026-09-05)

Report for task.setup-sessionstart-hooks: `anb setup` wires the agents' session start to the notebook, in the project, and the hook was verified working in Claude Code and Codex.

## What shipped

`anb setup`, run in the project directory, writes four files and patches them in place on a re-run; `anb setup --remove` takes out only what setup put in and deletes a file it alone had filled.

| File | What setup writes | Who reads it |
|---|---|---|
| `AGENTS.md` | one marker-bounded line: `Project working memory: .agent-notebook/ — anb status shows the current state, anb --help the commands.` | Codex, Pi, OpenCode and any agent that reads AGENTS.md |
| `CLAUDE.md` | the same line, unless the file links `AGENTS.md` (`links AGENTS.md`) or imports it with a bare `@AGENTS.md` line (`imports AGENTS.md`) | Claude Code, which reads `CLAUDE.md` and not `AGENTS.md` — its own docs say so and recommend the import or the link |
| `.claude/settings.json` | a `hooks.SessionStart` group running `anb status --hook` with a 15-second timeout, beside any group another tool already has | Claude Code |
| `.codex/hooks.json` | the same group, the same shape — Codex mirrors Claude Code's hook contract | Codex, after the user trusts the project and reviews the hook with `/hooks`; setup prints that notice |

Rules the implementation keeps: every file is read and judged before the first is written, so a refusal leaves the project exactly as it was; a settings file that is not JSON, or an instruction file with a stray marker, refuses the whole run, since setup patches only what it can read back; a file that is a link to somewhere else is left alone and reported (`a link, left alone`), because writing through it would edit a file the project does not own; setup recognises its own hook by the command alone, so a timeout the user tuned keeps the group setup's; the snippet is descriptive, never imperative, and its markers are HTML comments, which Claude Code strips before injection; `setup --global` is refused with its own reason, because a session starts in a project and the user's notebook needs no hook; setup takes no notebook lock and needs no notebook to exist. The reply's first line counts files and names the verb; each file's line carries what happened to it, so the header cannot promise a write that did not happen.

## Review

One review pass: seven findings, all taken. Two must-fix — the non-JSON refusal came after three files were already written (now every file is planned before any is written), and a `CLAUDE.md` linked to somewhere else was edited through the link (links are now left alone). Should-fix — a stray marker was patched around into a duplicate (now refused); the header said `4 files written` over four `already` lines (now it counts and names the verb); the "moves nothing" test asserted one file (now all four); a deletion comment claimed more than the code guaranteed (rewritten as the rule it is). One nit, a doc comment restating its body, deleted.

## Verified live

Scratch projects with one active Task, `anb setup`, then the real binary on `PATH`:

- **Claude Code** — `claude -p "…reply with the line that begins with active:"` answered `active: task.grammar-parser-accepts-fences "Grammar parser accepts fences"`: the project hook fired, the Status reached the model.
- **Codex** — `codex exec` in an isolated `CODEX_HOME` whose `config.toml` trusts the project, with the hook reviewed (`--dangerously-bypass-hook-trust` stands in for the interactive `/hooks` review): the same line came back from the project `.codex/hooks.json` setup writes. Without the review step Codex answered `NO STATE` — the trust step is real, and the notice setup prints is what a user needs.

Two traps met on the way, recorded for whoever verifies next: `codex exec` blocks forever when its stdin is not a terminal (run it with stdin closed), and the project's trust must stand in `config.toml` — a `-c projects.<dir>.trust_level` override did not load the project hooks. The maintainer's own `~/.codex` was not touched; the isolated home and its copied credential were deleted after the run.

## Tests

Unit tests on the pure text and JSON transformations (append, patch in place, remove, coexist with another tool's group, recognise a tuned command, recognise the import). End-to-end through the binary: a fresh project gets four files and a second run changes nothing; another tool's lines and hook group survive install and removal; a linked `CLAUDE.md` gets one line, not two; an importing `CLAUDE.md` is left alone; a settings file that is not JSON refuses the run and moves nothing; `--global` is refused. The gate is green.

## Left open

The skills setup will install come with the skill generator (the next task); `setup` gains their files then. Pi and OpenCode read the AGENTS.md snippet and need no hook in v1.
