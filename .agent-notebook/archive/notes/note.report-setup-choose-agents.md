---
id: note.report-setup-choose-agents
type: note
state: retired
title: Report: setup chooses which agents to wire
by: Maksim Yaromin
from: task.setup-choose-agents
created: 2026-09-06
updated: 2026-09-06
---



# setup chooses which agents to wire

`anb setup` wired every host it knew in one run: the snippet in `AGENTS.md` and `CLAUDE.md`, a hook for Claude Code and one for Codex, and both skills into both skills directories. In a repository where only Claude Code runs that was eighteen files, a `.codex/` directory that did not exist before, and a second copy of each skill. The footprint is the first thing a new adopter reads in `git status`, and deleting the files by hand did not hold: the next `setup` after an upgrade wrote them again, and `--remove` took out everything.

## What changed

- `anb setup --agent <name>`, repeatable, wires the named agents and no other: `claude-code` writes `CLAUDE.md`, or `AGENTS.md` when `CLAUDE.md` links or imports it, the Claude Code hook and `.claude/skills`; `codex` writes `AGENTS.md`, the Codex hook and `.agents/skills`; `agents-md` is any tool that follows the agents.md convention and writes `AGENTS.md` and `.agents/skills` with no hook. A file two agents share is planned once. The reply names the agents it skipped, so the choice is on record in the log.
- Without `--agent`, and with a name setup does not know, setup refuses before any byte moves and offers one `try:` line per agent. It refuses on a terminal as anywhere else: the reply contract has no interactive surface, and the agents that run the tool never have a terminal, so one shape serves both.
- `--remove` takes the same flag and takes out the named agents' files; a file two agents read goes only when every agent that reads it is named, and the reply says which it kept. A host directory that held nothing but setup's files goes with them: `.codex/`, `.agents/` and `.claude/` are gone after a removal on a project that had none of them, while a directory holding anything else stays. This reverses an earlier promise that removal never climbed above the skill directory; the old promise left `.agents/skills/` and `.codex/` standing empty, which is the footprint the issue is about.
- The agents guide names the three agents in one table, the quickstart and the README show `--agent claude-code`, the customization guide says where each agent's skill copy lives. The ruling and its alternatives are recorded as `decision.setup-wires-only-the-agents-named`.

## Evidence

- End-to-end through the binary: `--agent claude-code` alone writes nine files and no `AGENTS.md`, `.codex` or `.agents`, and names the two skipped; a `CLAUDE.md` importing `AGENTS.md` gets the line there for Claude Code alone; bare `setup` and `--agent pi` refuse with the three try lines and write nothing; `--agent codex --remove` after both were set up takes out Codex's files and directories and leaves Claude Code's; a removal takes out the host directories it emptied and leaves one holding the user's own skill. The two tests that set up both agents keep their eighteen files with the skipped line added. Each new test was shown red once.
- `scripts/check.sh` green, the regenerated skill and reference pages included; `pnpm docs:check` green.
- Smoke check: one must-fix, taken: removal of one agent took out a file another agent still read, since sharing was judged from the agents named in that call alone; a shared file now goes only when every agent that reads it is named, the reply says which it kept, three end-to-end tests pin it and the guide states the rule. Should-fix, taken: the text renderer's doc comment now names the skipped line. Nits taken: a dead fallback in the host directory helper; the shared-resource test the check asked for is the one above.

Pull request: https://github.com/maksimyaromin/agent-notebook/pull/54, squash-merged on a green CI check.
