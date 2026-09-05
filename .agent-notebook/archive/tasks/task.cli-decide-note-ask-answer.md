---
id: task.cli-decide-note-ask-answer
type: task
state: closed
title: CLI: decide / note / ask / answer
by: Maksim Yaromin
via: claude-code
tags: cli
link: note note.report-cli-decide-note-ask-answer
blocked-by: task.core-record-model-invariants
created: 2026-08-24
updated: 2026-08-29
closed: 2026-08-29
---

Conflicting Decision requires the supersedes flag; routing is the only way a Question closes; Terms supported.

Maintainer's call 2026-08-29: retire lands here — the CLI surface for the standalone, no-successor retirement of a Decision or Note (active -> retired). Core::retire already enforces the transition and idempotent replay; retirement with a successor stays decide/note --supersedes. The command vocabulary in the spec is amended accordingly.

Progress 2026-08-29: work done, independent review done, findings fixed, gate green.

Scope shipped: decide/note/ask/answer/retire CLI commands; Created.may_conflict + conflict_candidates in core (the decide write-time nudge: standing valid Decisions sharing >=2 distinct tags with the draft or cited in its body, oldest first then id, by/via attribution via debt::Cited; skipped when --supersedes declared); Reply::Created{command}/Reply::Answered; may-conflict and routed-to renderings in text+json; answer/create recovery try: lines; Record::is_live as the one home of the liveness fact.

The independent review: 16 findings. Fixed: ruling->decision identifier renames (glossary Avoid-list); created()/sort dedupe into free created+oldest_first; liveness fact unified on Record::is_live (4 sites); guard-first + two named predicates (is_standing_decision, looks_related) in conflict_candidates; pass-through created() helper inlined; AskArgs deleted (Ask(DraftArgs)); nudge doc trimmed to the why, Created doc mirrors Closed; verb-keyed argument retries for add/decide/note/ask (the id-less hole, add's predating hole swept in); text.rs cited shadowing fixed, count from the field; flip_victim extracted from create; Origin help wording per CONTEXT. Added tests: standing-side tag dedupe, same-day id tie-break (both proven red first). Declined with reasons: nudge NOT moved into debt.rs (it is a create-reply consequence, not Debt; the shared fact now lives in Record::is_live); may-conflict stays unbounded (matches the tree's consequence-list dialect; reviewer concurred); Cited::author dash-fallback and answer/retire replay CLI tests skipped as covered elsewhere/low value per reviewer. Named at handoff, opens its own change: deepening answer to Core Notebook::answer(id, &Routing) mirroring close/Proof; --priority moved to last in anb add --help (matters when the generated skill lands, task a2).
