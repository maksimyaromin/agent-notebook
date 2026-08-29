# AGENTS.md

Guidance for coding agents (Claude Code, Codex, Pi, and others) working in this repository.
`CLAUDE.md` is a symlink to this file; edit `AGENTS.md` only.

## Project

**agent-notebook** (anb) stores a project's working memory in the repository as typed records with lifecycles — Tasks, Decisions, Notes, Questions under `.agent-notebook/` (CLI named `anb`, directory the full word — owner's call, 2026-08-25) — read and mutated by any agent through a Rust CLI (owner's call, 2026-08-25 — for fun; ADR 0006 supersedes 0003 Go). The backlog is the deepest-worked pattern of the notebook, not the whole of it.

Status: self-hosted. The backlog lives in the notebook itself (`.agent-notebook/`), read and mutated only through the anb CLI; the concept spec is grilled and frozen (2026-08-24).

Goals, in priority order:

1. Any agent can read and mutate the notebook through a small CLI with a stable, token-cheap output.
2. The markdown file stays the source of truth and stays readable by a human.
3. The task/backlog workflow remains a concrete, well-supported pattern, not an afterthought.

Prior art to borrow ideas from, not code: `tasks-axi` (markdown backlog CLI, byte-exact round-trip, derived `ready` queue, structured holds), `beads` (dependency graph model).

## Repository layout

- `.tmp/` — git-ignored. Scratch space AND the standard home of all working documents at this stage: nothing under it may be moved or copied elsewhere in the repo. Never use `/tmp`.
- `.claude/` — local agent config and skills, git-ignored.
- `.agents/` — reserved for skills and agent config that must be committed and shared.
- `.agent-notebook/` — the notebook: the committed source of truth for Tasks, Decisions, Notes, Questions. Mutate it only through the anb CLI, never by hand-editing the files.

## Working documents (all under `.tmp/`, deliberately uncommitted)

- `.tmp/docs/spec-anb-concept.md` — the grilled concept spec: problem, 32 user stories, implementation and testing decisions, Definition of Done.
- `.tmp/docs/CONTEXT.md` — the domain glossary; use its canonical terms (Record, Task, Status, Check, ...) in every language.
- `.tmp/docs/adr/` — decisions: 0001 own format, 0002 cross-project as ecosystem horizon, 0003 Go for the CLI (superseded), 0004 record file format, 0005 record model, 0006 Rust for the CLI (with the why-Rust release gate).
- `.tmp/docs/research-method.md` — the scientific search method: novelty questions get a systematic sweep, findings classified tried-vs-theory (theory weighs more); research subagents are Sonnet 5 only.
- `.tmp/docs/research/` — the evidence base (tasks-axi dissection, codemode-executor, prior art, kody, field tests).

## Engineering instruction

Before writing any code: load `.tmp/docs/engineering-instruction.md` and every skill it points to. Loading is not the point — the skills bind every line: apply them while planning a function, while writing it and its tests; the review stage holds the diff against them once more. Code that merely follows after a reading of the skills was rejected (2026-08-26). It defines the engineer working here (engineering + codebase-design skills), the testing mindset (BDD; tests survive refactoring, test behavior and business requirements, never lines), the code/naming principles (the-art-of-code), and the law for every text in or about code (comment-rules + text-quality-pass). The skills are not installed — reach them at their paths; a skill referencing another skill is found near the entry point.

## Task protocol

The project dogfoods its own tool: the backlog is the notebook, and every task-state change goes through the anb CLI, invoked from the repo root as `cargo run --quiet -- <command>` until a binary ships. Every CLI failure, bug, or friction met on the way is a finding — file it into the notebook (a comment on the task it burdens, or a new Task/Question born `--from` the current one) instead of working around it silently.

When the owner says **"continue the task"** (in any wording, any language), it means exactly this:

1. Run `cargo run --quiet -- status`. The **in-flight** (active) task is THE task — there is never more than one. If none is active, take the top of `cargo run --quiet -- ready` and `start` it.
2. Read the task with `view <id>`, read the docs it references under `.tmp/docs/`, and resume from its log — not from scratch.

Every task moves through these stages:

1. **Dispatch** — `anb start <id>`.
2. **Work** — execute against the body's acceptance criteria; append progress notes with `anb comment <id> "<text>"` so any later session can resume mid-task. Deliverables are written under `.tmp/`.
3. **Code review by Opus 5 (mandatory on coding tasks; owner's call, 2026-08-27)** — NEVER reviewed by the authoring model: spawn a separate agent on Opus 5 that loads the engineering instruction and its skills, re-reads the whole diff, code and tests, holding every line against them (story, naming, comments, test behavior), and returns findings with file:line. Fix what it finds and report the findings honestly. A review that finds nothing was not performed.
4. **Review pause (mandatory, never skipped)** — when the work is done, `anb submit <id>` and STOP. Leave every produced or changed file in the working tree — **uncommitted and unstaged** (no `git add`). Report what is ready and where, then wait for the owner to review.
5. **Close** — only after the owner's explicit approval: `anb close <id> --report <path>` (or `--pr`, `--sha`), and commit/push only if the owner asks.

No formal task closure, no commit, and no push ever happens before the review pause in stage 4.

## Conventions

- Lavish is never part of a coding task's review (owner's call, 2026-08-26; who reviews is stage 3's rule). Lavish is for brainstorming and design discussions only — there it runs through the **`grill-with-docs` skill** (`~/dev/skills/skills/engineering/grill-with-docs`), which defines the interview format; the artifact must be fully self-contained: digested proofs with their numbers, worked examples, and diagrams in place — never pointers into `.tmp/docs/research/`.
- Language: code, comments, commit messages, and docs in English.
- Committed text is self-contained (owner's call, 2026-08-28): comments, docs, and test data never cite what only `.tmp/` holds — no spec §, research-report, ADR, user-story, or owner-ruling pointers. State the constraint itself; provenance stays in `.tmp/` reports.
- Commit messages follow Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`).
- Commit and push only when asked.
- Keep this file short and current: update it when a convention or command changes, remove anything that stops being true.

## Commands

- `./scripts/check.sh` — the local gate: `cargo fmt --check`, `cargo clippy -D warnings`, all tests, doctests.

## Markdown authoring

- Never insert artificial line breaks inside markdown prose: one paragraph or list item is one physical line, however long. Editors soft-wrap; hard wraps corrupt diffs and editing.
