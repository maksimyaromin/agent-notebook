---
id: note.report-milestone-self-host-switch
type: note
state: retired
title: Report: Milestone: self-host switch
by: Maksim Yaromin
via: claude-code
from: task.milestone-self-host-switch
created: 2026-08-29
updated: 2026-08-29
---

# m1 — Milestone: self-host switch (2026-08-29)

The backlog moved from tasks-axi to anb by hand, exclusively through the anb CLI — the first field test. Development continues in the notebook; tasks-axi leaves the project.

## What moved

- **22 Tasks**: 10 finished ones (s1–s3, g1, c1–c4, l1, l2) replayed open→active→closed in dependency order, each closing with its report as proof; 12 live ones (the queue as it stood, plus m1 itself, active). Bodies migrated verbatim, progress journals included; titles cleaned of tasks-axi's report-path suffixes (the proof field carries the path now).
- **35 blocked-by edges** — the full historical graph, satisfied edges included, so the notebook reads as if the project ran on anb from the start.
- **1 hold**: the final-CLI-name gate, with its reason.
- The old→new id mapping is logged on `task.milestone-self-host-switch`.

## The binding idea (owner direction during review)

The migrated tasks are not a flat sheet — they all implement one plan, and future ideas will bring their own. `task.anb-v1` (tag `epic`) is now the founding hub: blocked-by all 21 plan tasks (edges to closed tasks are accepted and read as satisfied — verified during this work), so it closes only when the idea is done. What today's anb could NOT do: put `from:<hub>` on the pre-existing children — origin is add-time only, retrofitting waits on the edit surface (evidence logged on the epic-pattern task). **The convention's home is the generated skill (owner ruling during this review): the obligation is logged on `task.skill-from-help-ci-drift-check` — the skill must teach that a task born inside an idea carries `--from` its hub at add time, as the default shape of task creation, not an option.** Two wrong homes were tried and fully reverted along the way: an AGENTS.md paragraph, then a Note. The Note's removal became its own field test: `retire` worked first try (active→retired), but the owner wanted zero trace and the CLI has no expunge — the file was deleted by hand, the gap filed as `question.does-the-notebook-need-an-expunge-for-re`, and the notebook stayed healthy after the vanished file (status clean, `view` of the gone id answers with a plain unknown-id error).

## Findings filed (the test half of the milestone)

- `task.slug-minting-cut-at-a-word-boundary` — minted ids truncate mid-word (four occurrences during this migration).
- `question.how-does-prose-mention-a-record-id-witho` — the mention scan reads id-shaped prose as references; 4 dangling-mention debt rows now on Status with no repair path before the edit surface.
- `question.should-record-import-preserve-historical` — no backdating: replayed history wears today's envelope dates.
- `question.where-does-a-report-proof-live-so-a-fres` — report proofs point into git-ignored `.tmp/`, unreachable from a fresh clone (owner spotted).
- Field evidence commented onto `task.cli-check-archive-edit-search-overview` (edit/archive is the repair path the migration lacked).
- Full experience report: `.tmp/docs/research/16-m1-self-host-field-test.md`.

## Repository changes (in the working tree, uncommitted)

- `.agent-notebook/` — the notebook: 22 tasks, 3 questions, 1 finding task; new, untracked.
- `AGENTS.md` — task protocol rewritten onto the anb CLI (`cargo run --quiet --` until a binary ships), status line updated to self-hosted, `.agent-notebook/` in the layout, tasks-axi operational references removed. The prior-art line naming tasks-axi as an idea source is kept deliberately — it documents provenance, not tool usage; strike it if "no trace" includes it.
- `.gitignore` — the `.tasks.toml` line removed.
- `.tasks.toml` — deleted (was git-ignored, so no diff).
- `.claude/skills/tasks-axi/` — deleted (local, git-ignored).

## Left for close, after owner approval

- Archive `.tmp/backlog.md` to `.tmp/data/m1/backlog-final.md` and delete the original (the last tasks-axi artifact).
- `anb close task.milestone-self-host-switch --report .tmp/data/m1/report.md`.
- Update agent memories that still point at tasks-axi (continue-task protocol, docs-stay-in-tmp).

## Honest limits

- Envelope dates on replayed history read 2026-08-29; true dates live inside the migrated bodies only.
- The ten historical proofs point into `.tmp/` — truthful (the reports live there) but machine-local; the filed proof-reachability question owns the policy.
- Opus 5 review not run: the milestone wrote no code — the mandatory review binds coding tasks.
