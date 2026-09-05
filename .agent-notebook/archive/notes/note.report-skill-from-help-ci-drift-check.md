---
id: note.report-skill-from-help-ci-drift-check
type: note
state: retired
title: Report: Skill from help + CI drift check
by: Maksim Yaromin
from: task.skill-from-help-ci-drift-check
created: 2026-09-05
updated: 2026-09-05
---

# The skill, generated from the binary (2026-09-05)

Report for task.skill-from-help-ci-drift-check: the agent skill is rendered by the CLI itself, installed by `anb setup` where each agent looks for skills, and diffed against the committed copy in CI, so it cannot drift from the tool it teaches.

## What shipped

`anb skill` prints the skill; `anb skill <dir>` writes it; `anb skill <dir> --check` compares the directory with the rendering byte for byte and fails the exit on any difference or missing file, naming each and printing the command that repairs it. `scripts/check.sh` runs that check on `.agents/skills/anb`, the committed copy Codex and Pi read. `anb setup` now installs the same files into `.claude/skills/anb` (Claude Code) and `.agents/skills/anb` (Codex, Pi) of the target project, beside the snippet and the hooks, and reports each file's outcome; a skill file whose frontmatter no longer carries `generated: anb` is the user's from then on — reported `yours, left alone`, never rewritten, never removed.

Four files, one read once and three opened on demand:

| File | Carries | Rendered from | Open when |
|---|---|---|---|
| `SKILL.md` | the method: the session, the Task loop, the shape of work (hub Tasks, `--from` at add time), knowledge, the user's notebook and the skills-as-notes convention, the reply contract, check before stopping | hand-written text in the generator | every session |
| `commands.md` | every verb with its arguments and flags | the clap tree the binary parses with | before a verb not used yet |
| `session.md` | one notebook from empty to archived work, every reply as printed | executing the steps on an in-memory notebook, day fixed to 2026-01-15, author `Ada` | to see what a reply looks like before parsing one |
| `refusals.md` | every refusal code with its cause and `try:` repair, and the check findings by severity | executing the refusing commands the same way; the finding codes from the Core | when a `try:` line is not enough |

The method the skill teaches is the owner's ruling made text: hygiene is the agent's job, not the developer's — archive in the same move as the close, reports ingested as Notes, Questions closed the moment they settle, holds with reasons, `anb check` green before stopping. Users who work differently do not install the skill or override it; the CLI carries no opinion.

## A decision made mid-work

The first rendering was one `reference.md` of 590 lines. The owner asked whether that is efficient, and the skill-writing guidance answers: disclosure is the central architectural decision, and a reference is named in prose with a consult-before-acting trigger and a preview of its contents. One flat file forces a session that needs a flag table to load the whole refusal catalog too. The reference is now three files, each for one need, each named in `SKILL.md` with the moment to open it. `SKILL.md` itself stays under a thousand words.

## Tests

Unit tests in the generator: the rendering is deterministic; every file carries the mark and a file without it is not generated; every verb but help has a commands entry; no worked-session step was refused and the refusals hold a refusal. End to end through the binary: the skill is written, the check passes, an edited and a missing file are drift with a failing exit and the repairing `try:` line; setup installs the skill files and a second run changes nothing; `setup --remove` takes them out with the directory they alone filled; a skill file the user made theirs is left alone by install and by removal. The verb is in the vocabulary table, the global-scope table (skill admits `--global` — it names no notebook) and the lock table (skill takes no lock). The gate is green, drift check included.

## Smoke check

Sonnet 5, once, verifying the claims against the code and the running binary. Eight findings, all taken. Two must-fix, both in text an agent would act on: `SKILL.md` said a hub "closes when the last child does" — false, the last close only makes the hub ready, so the sentence now says so and tells the agent to close and archive the hub like any Task; the `skill` help named `reference.md`, a file that no longer exists. Should-fix: an optional positional rendered as required in the commands reference (now `[DIR]`); the setup module doc still counted two mechanisms (now three); "`anb archive` in the same move as the close" read as if archive were automatic (now "right after the close"); `--json` on the skill verb and `--check` without a directory were real and untested (both tested now); `FindingCode::ALL` was a hand-kept list nothing guarded (an exhaustive match in a test now fails to compile when a variant is added without its entry). One nit, two doc comments restating their signatures, deleted. The check also recorded that `skill --global` is admitted and does nothing, which is the intended reading — the verb touches no notebook — and that every reply in the worked session matched the real binary byte for byte.

## Left open

The skill teaches the method; whether agents follow it well is measured by using it. Findings from the first sessions on the installed skill are follow-up tasks. `task.global-skill-notes-docs` asked for exactly the text the "user's own notebook" section now carries and closes on this work.
