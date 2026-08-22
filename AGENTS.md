# AGENTS.md

Guidance for coding agents (Claude Code, Codex, Pi, and others) working in this repository.
`CLAUDE.md` is a symlink to this file; edit `AGENTS.md` only.

## Project

**agent-notebook** is a portable, structured notebook for coding agents: one hand-editable
markdown file that any agent reads at the start of a session and updates at the end.
The backlog (queued → in flight → done) is the first-class workflow pattern inside it.

Status: greenfield. No code yet; design and scope are being decided.

Goals, in priority order:

1. Any agent can read and mutate the notebook through a small CLI with a stable, token-cheap output.
2. The markdown file stays the source of truth and stays readable by a human.
3. The task/backlog workflow remains a concrete, well-supported pattern, not an afterthought.

Prior art to borrow ideas from, not code: `tasks-axi` (markdown backlog CLI, byte-exact round-trip,
derived `ready` queue, structured holds), `beads` (dependency graph model).

## Repository layout

- `.tmp/` — scratch space, git-ignored. Put throwaway files here, never in `/tmp`.
- `.claude/` — local agent config and skills, git-ignored.
- `.agents/` — reserved for skills and agent config that must be committed and shared.

## Conventions

- Language: code, comments, commit messages, and docs in English.
- Commit messages follow Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`).
- Commit and push only when asked.
- Keep this file short and current: update it when a convention or command changes,
  remove anything that stops being true.

## Commands

None yet. Add build, test, and lint commands here as soon as they exist.
