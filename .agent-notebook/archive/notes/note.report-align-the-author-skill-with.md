---
id: note.report-align-the-author-skill-with
type: note
state: retired
title: Report: Align the author skill with writing-skills structure
by: Maksim Yaromin
from: task.skill-structure
created: 2026-09-06
updated: 2026-09-06
---

# Align the author skill with writing-skills structure

The main skill now contains the author method: overview, use conditions, core pattern, record-selection table, application, common mistakes and links to heavy references. Planning and domain modeling no longer require separate conceptual reference files. The generated bundle contains the main skill, command reference, worked session and refusal reference. The existing anb name and crates/anb/src/skill/anb.rs source layout are preserved. Guidance remains direct English addressed to the acting agent; record choice and relationships carry their reasons. The docs describe this structure.

## Validation

The existing CLI bundle test first failed because the renderer emitted six files instead of the required four, then passed after consolidation. scripts/check.sh passed: formatting, clippy, workspace tests, doctests, rustdoc and generated-file checks. pnpm docs:check passed with all 20 book pages reachable and links resolving. All four generated YAML frontmatters parsed successfully. The runnable small-change example executed against a scratch notebook and check reported no findings. No new dependency was installed.

## Behavioral samples

The baseline scenario asked a developer to preserve a ticket about reusable verified exports, distinguish delivery and publishing meanings of package, explain ownership, leave git versus database storage unresolved, and prepare a handoff with twenty minutes remaining after earlier sketching. Capture and investigation were authorized; implementation and external contact were excluded. The complete supplied ticket and a minimal CLI contract were available. Each sample used a fresh agent context and produced a command proposal; these samples did not execute mutations.

Five no-guidance samples created no idea Notes, placing the source in an investigation Task. Their explanations included “The open Task preserves the request and its source” and “Each record originates from the ticket Task”. Three omitted via on supporting records and three used guide for a one-off handoff. Those failures concern output structure and missing fields, so the revision uses an affirmative idea-to-work pattern, a via-bearing example and a mistakes table rather than discipline prohibitions.

All five guided samples retained one idea and used via codex on every proposed creation. All preserved both context meanings and provisional ownership in models, kept storage in an open Question and proposed only bounded investigation. Model granularity and the need for a separate spec varied, as the method permits. One sample miscounted its proposed records in its explanatory summary; its command sequence contained eight records. No corrective rule was added for that incidental counting error. No improvement is claimed for domain uncertainty handling, which was already present in the controls. The minimal control contract omitted ID grammar, so invalid control IDs were not counted as evidence of a skill failure.

An additional real CLI exercise captured an agreed button-label change with unchanged behavior and CSV content. It created only an idea and one open Task, both with via codex, connected by origin and mention edges. A subsequent readiness question returned that Task. File-name and content-hash comparisons showed that the reads changed no notebook files. Both checks returned zero findings. These observations cover the sampled scenarios and one agent runtime, not all possible agents or tasks.

## Scope and review

The main skill retains proportional planning, DDD guidance, explicit agreement status, source lineage, one active Task by default, closure with evidence, archiving and user overrides for location and sharing. Brief intake and sustained research remain judgment calls. Heavy command syntax stays in references. A separate rationalization/red-flags section was unnecessary because the observed failures were record-selection errors, not deliberate rule violations. The authoring checklist's publishing step is excluded by the user's no-commit/no-push instructions. No commit, push, release, paid evaluation service or external-system change was performed.
