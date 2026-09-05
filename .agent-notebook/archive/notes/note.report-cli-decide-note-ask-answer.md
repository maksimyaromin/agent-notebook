---
id: note.report-cli-decide-note-ask-answer
type: note
state: retired
title: Report: CLI: decide / note / ask / answer
by: Maksim Yaromin
via: claude-code
from: task.cli-decide-note-ask-answer
created: 2026-08-29
updated: 2026-08-29
---

# l2 — CLI: decide / note / ask / answer / retire — report

Closed 2026-08-29. Delivered against the record-model spec (§3 lifecycles, §7 by/via, §9 undeclared conflicts), the interaction spec's output contract, and the maintainer ruling of 2026-08-29 that landed `retire` in this ticket.

## What shipped

**Five commands joined the surface**: `decide`, `note`, `ask`, `answer`, `retire`. The Core already held `create` (all types, kinds, the supersession flip), `route`, `drop_question`, and `retire` from the c-tasks, so the bulk of l2 is the CLI surface plus one new Core computation — the write-time conflict nudge.

- `anb decide "<title>" [--kind rule|shape|drift] [--supersedes <id>]` — creates a Decision; a declared supersession answers `superseded: <victim>` as before. Without `--supersedes`, the reply carries the nudge (below).
- `anb note "<title>" [--kind fact|term|guide] [--supersedes <id>]` — Terms are `--kind term`; a superseded Note retires with the back-pointer.
- `anb ask "<title>" [--from <id>]` — a Question with its Origin; a dangling origin is a `dangling-ref` payload with the create-it `try:` line.
- `anb answer <id> --to <id>` routes (`open→routed`, `routed-to` written in the same move; only a decision or task accepted); `--drop "<reason>"` closes without routing, appending `Dropped <date>: <reason>` to the body. Neither or both flags is an `invalid-argument` payload naming both shapes.
- `anb retire <id>` — `active→retired` for a Decision or Note with no successor; idempotent replay answers `(already)`.

**The write-time nudge** (Core `conflict_candidates`): a Decision created without `--supersedes` is answered with the standing Decisions it may conflict with — those sharing two or more distinct tags with the draft, or cited in its body — oldest first then id, each with its `by`/`via` attribution reusing `debt::Cited`. Candidates are live, non-archived, valid records only (the derived-query exclusion rule). Rendered as `may-conflict[n]: id (by/via), …` in text and a `"may-conflict"` array in JSON (absent `by`/`via` omitted). A consequence in the reply, never a block: the file is written regardless.

**Surface shape**: the create commands share a flattened `DraftArgs` (title, `--id --from --tag --link --body --by --via`); `add` adds `--priority`, `decide`/`note` add `--kind`/`--supersedes` with per-type help, `ask` is `DraftArgs` alone. `Reply::Created` now carries the command word (`ok: decide …`, `"ok":"decide"`); `Reply::Answered` carries the transition plus `routed-to:` when routed. Argument retries became verb-keyed, so the id-less create verbs get command-shaped `try:` lines (a bad `--kind` now answers `try: anb note "<title>" --kind fact`); `add`'s predating gap was swept in the same move.

**Core deepening alongside**: `create` reads the notebook once (`resolve_draft_id`/`id_claims` became free functions over that read); `flip_victim` extracted; `Record::is_live()` became the one home of "the record still binds", replacing four hand-written state checks across notebook.rs and debt.rs; `created`/`oldest_first` are shared by `standing_rules` and the nudge; `Cited::of` replaced debt's private constructor.

**Tests**: core `mod conflict_nudge` (12) specifies the nudge through `Notebook::create` — tag threshold, body citation, set semantics on both sides, declared-supersession silence, liveness/validity/type filters, oldest-first with the id tie-break; CLI `mod knowledge_replies` (12) + 2 JSON tests pin the rendered contract; the command-vocabulary table grew the five verbs. Every new test proved it can fail (expectation flipped red, restored green). A smoke run of the real binary in a scratch repo exercised the full cycle including refusals and `--json`.

## Review (separate agent)

16 findings: 6 should-fix, 6 nits, 2 observations, 2 categories explicitly verified clean (spec fidelity of the nudge clause-by-clause; behavior-preservation of the create refactor). No correctness defect. All should-fixes and most nits fixed:

- Fixed: "ruling" promoted into twelve identifiers against CONTEXT.md's Avoid list → renamed to `decision`; duplicated `created()`/sort lifted to shared free fns; the live-decision fact unified on `Record::is_live`; guard-first plus two named predicates (`is_standing_decision`, `looks_related`) in the nudge filter; the pass-through `created()` helper inlined to match `Reply::Moved`'s dialect; `AskArgs` (zero fields over `DraftArgs`) deleted; the nudge doc trimmed to the why it restated three times; the `cited` shadowing in text.rs with the count read off the field; `flip_victim` extraction; the Origin help wording; the recovery-payload hole on id-less verbs (with the snapshot updated); two coverage gaps (standing-side tag dedupe, same-day id tie-break), both proven red first.
- Positions held, stated: the nudge stays in notebook.rs, not debt.rs — it is a create-reply consequence, not a computed sign of decay; the shared fact the reviewer chased now lives in `Record::is_live`. `may-conflict` stays unbounded, matching the tree's consequence-list dialect (`unblocked`, `open-questions`, `mentions`), the reviewer concurring. The `Cited::author()` dash-fallback and answer/retire replay CLI tests were skipped as covered elsewhere at the interface that owns them.
- Named at handoff, opens its own change: deepening `answer` into Core `Notebook::answer(id, &Routing, today)` mirroring `close`/`Proof`, so the host stops knowing which Core method each flag maps to; `--priority` moved to last in `anb add --help` via the flatten — inert until the generated skill (a2) starts byte-diffing help.
