# AGENTS.md

Guidance for coding agents (Claude Code, Codex, Pi, and others) working in this repository.
`CLAUDE.md` is a symlink to this file; edit `AGENTS.md` only.

## Project

**agent-notebook** (anb) stores a project's working memory in the repository as typed records with lifecycles — Tasks, Decisions, Notes, Questions under `.agent-notebook/` (CLI named `anb`, directory the full word — owner's call, 2026-08-25) — read and mutated by any agent through a Rust CLI (owner's call, 2026-08-25 — for fun; ADR 0006 supersedes 0003 Go). The backlog is the deepest-worked pattern of the notebook, not the whole of it.

Status: no code yet. The concept spec is grilled and frozen (2026-08-24); tickets are cut and live in the tasks-axi backlog; work starts on the owner's signal.

Goals, in priority order:

1. Any agent can read and mutate the notebook through a small CLI with a stable, token-cheap output.
2. The markdown file stays the source of truth and stays readable by a human.
3. The task/backlog workflow remains a concrete, well-supported pattern, not an afterthought.

Prior art to borrow ideas from, not code: `tasks-axi` (markdown backlog CLI, byte-exact round-trip, derived `ready` queue, structured holds), `beads` (dependency graph model).

## Repository layout

- `.tmp/` — git-ignored. Scratch space AND the standard home of all working documents at this stage: nothing under it may be moved or copied elsewhere in the repo. Never use `/tmp`.
- `.claude/` — local agent config and skills, git-ignored.
- `.agents/` — reserved for skills and agent config that must be committed and shared.
- `.tasks.toml` — points tasks-axi at `.tmp/backlog.md`.

## Working documents (all under `.tmp/`, deliberately uncommitted)

- `.tmp/docs/spec-anb-concept.md` — the grilled concept spec: problem, 32 user stories, implementation and testing decisions, Definition of Done.
- `.tmp/docs/CONTEXT.md` — the domain glossary; use its canonical terms (Record, Task, Status, Check, ...) in every language.
- `.tmp/docs/adr/` — decisions: 0001 own format, 0002 cross-project as ecosystem horizon, 0003 Go for the CLI (superseded), 0004 record file format, 0005 record model, 0006 Rust for the CLI (with the why-Rust release gate).
- `.tmp/docs/research-method.md` — the scientific search method: novelty questions get a systematic sweep, findings classified tried-vs-theory (theory weighs more); research subagents are Sonnet 5 only.
- `.tmp/docs/research/` — the evidence base (tasks-axi dissection, codemode-executor, prior art, kody).
- `.tmp/backlog.md` — the tasks-axi backlog (19 tickets, dependency graph); mutate it only through `npx -y tasks-axi`.

## Engineering instruction

Before writing any code: load `.tmp/docs/engineering-instruction.md` and every skill it points to — load, understand, and apply, not skim. It defines the engineer working here (engineering + codebase-design skills), the testing mindset (BDD; tests survive refactoring, test behavior and business requirements, never lines), the code/naming principles (the-art-of-code), and the law for every text in or about code (comment-rules + text-quality-pass). The skills are not installed — reach them at their paths; a skill referencing another skill is found near the entry point.

## Task protocol

When the owner says **"continue the task"** (in any wording, any language), it means exactly this:

1. Run `npx -y tasks-axi` (dashboard). A task **in flight** is THE task — there is never more than one. If none is in flight, take the first task from `npx -y tasks-axi ready` and `start` it.
2. Read the task with `show <id> --full`, read the docs it references under `.tmp/docs/`, and resume from the progress notes in its body — not from scratch.

Every task moves through these stages:

1. **Dispatch** — `tasks-axi start <id>`.
2. **Work** — execute against the body's acceptance criteria; append progress notes to the task body (`tasks-axi update`) so any later session can resume mid-task. Deliverables are written under `.tmp/`.
3. **Review pause (mandatory, never skipped)** — when the work is done, STOP. Leave every produced or changed file in the working tree — **uncommitted and unstaged** (no `git add`). Report what is ready and where, then wait for the owner to review.
4. **Close** — only after the owner's explicit approval: `tasks-axi done <id> --report <path>` (or `--pr`), and commit/push only if the owner asks.

No formal task closure, no commit, and no push ever happens before the review pause in stage 3.

## Conventions

- Review pauses run the **`grill-with-docs` skill** (`~/dev/skills/skills/engineering/grill-with-docs`) over a Lavish artifact — the skill defines how the interview works; follow it, don't improvise the format. The artifact must be fully self-contained: digested proofs with their numbers, worked examples (mock records, CLI replies), and diagrams in place — never pointers into `.tmp/docs/research/` or cross-references between sections. The owner reads only the artifact; research reports are the agent's own working material.
- Language: code, comments, commit messages, and docs in English.
- Commit messages follow Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`).
- Commit and push only when asked.
- Keep this file short and current: update it when a convention or command changes, remove anything that stops being true.

## Commands

None yet. Add build, test, and lint commands here as soon as they exist.

## Markdown authoring

- Never insert artificial line breaks inside markdown prose: one paragraph or list item is one physical line, however long. Editors soft-wrap; hard wraps corrupt diffs and editing.
