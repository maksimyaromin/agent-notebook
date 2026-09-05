---
id: note.report-core-record-model-invariants
type: note
state: retired
title: Report: Core: record model + invariants
by: Maksim Yaromin
via: claude-code
from: task.core-record-model-invariants
created: 2026-08-29
updated: 2026-08-29
---

# c2 — Core: record model + invariants — completion report

Status: awaiting the maintainer's review. Working tree only — nothing staged, nothing committed. Local gate green (`./scripts/check.sh`: fmt, clippy `-D warnings` with pedantic, 112 tests, doctests).

## What shipped

- `crates/anb-core/src/finding.rs` — five new codes: `orphan-field` (warning), `broken-routing`, `dangling-ref`, `duplicate-id`, `broken-supersession` (errors); `Finding::located` for findings on fields that may lack a parse line.
- `crates/anb-core/src/grammar.rs` — the S2 envelope keys (`via`, `priority`, `hold`, `hold-until`, `review-by`, `routed-to`) woven into canonical order beside their S1 kin; `Form::NonEmptyText` (title, hold) and `Form::Priority`; the splice mutations `set_field` / `append_field` / `remove_field` / `append_body` per the format spec (values are one line by contract; a spliced line is canonical LF, untouched lines keep their bytes).
- `crates/anb-core/src/record.rs` (new) — `RecordType` with per-type state/kind vocabularies and live states; the Task machine (`open → active → review → closed`, review optional, reopen explicit; an invalid transition answers with the valid commands — US5; `return` has no already-state, so a return on a never-submitted task is refused, not "replayed"); `Record::parse` = grammar + placement + semantic findings (state/kind enums naming the valid set, type-bound fields as `orphan-field`, `kind` on task/question as forward-compatible `unknown-field`, `hold-until` pairing, `routed` without `routed-to` as `broken-routing`).
- `crates/anb-core/src/notebook.rs` (new) — `Notebook` over the Storage seam: `create` (id mint with 40-char slug cap, collision checked against filenames AND `id` fields across live + archive, 2-char base36 suffix retry; the supersession invariant: new record written first, victim gains `superseded-by` and flips in the same move), `start`/`submit`/`close`/`return_task`/`reopen`, `hold`/`unhold` (reason mandatory), `route`/`drop_question` (routing structural; drop appends `Dropped <date>: <reason>`), `retire`, and `check()` (per-record findings + `duplicate-id`, `dangling-ref`, `broken-supersession` from both ends, `broken-routing` for a dangling thread). Every mutation passes one gate (`resolve_live`): a record carrying an error finding — its own or a dangling reference — is never rewritten. Replays answer `already: true` and change no byte.
- Tests: 55 seam tests in `tests/notebook.rs`; `tests/notebook_props.rs` property-tests format-contract §4.3/§4.4 (replay byte-identical, bystander untouched) over generated tasks (CRLF, no final newline, fake-envelope bodies, unknown fields); corpus grew 11 cases and the harness now runs the full record pass on nested cases and judges acceptance by it.

## The independent review (per the new protocol)

The reviewer returned 20 findings; all are addressed. The behavioral defects — each reproduced by a failing test before the fix:

1. **`close` wrote an unvalidated proof** — an empty target bricked the record (`bad-value` freezes it), a newline in the target injected envelope fields. Now guarded before any byte moves; the one-line value contract lives in the grammar.
2. **`return` on an active task answered `already: true`** — one signal for "replayed" and "forbidden". Each action now names the state that proves it already happened; `return` has none.
3. **The supersession victim bypassed the mutation-eligibility gate** the module doc promises — a victim with a dangling `from` was flipped anyway. The victim now passes the same `resolve_live` gate as every verb.
4. **One hand-edited defect produced two identical findings** (`hold-until` on a non-task); the pairing check is now type-aware.
5. **A dangling `routed-to` got two different codes** from `check()` vs the mutation guard; the rule now has one home (`dangling_finding`).
6. **The valid-corpus harness still judged acceptance on the grammar parse** after switching findings to the record pass — a record-layer error under `valid/` slipped through. Proven with a temporary probe case, then fixed.
7. **Write-side id uniqueness trusted filenames** — a misnamed file claiming an id could let `create` mint a duplicate. Collision now checks filenames and `id` fields.

Plus: property tests the spec's testing decision demands (were missing), `# Panics` docs on the grammar mutators, `Record::origin()` (CONTEXT vocabulary; was `from()`), untested `records()` deleted until c3 needs it, `Display` for a terminal state says "no move is valid", the lower-word rule shared instead of copied, spliced lines carry `line: None` instead of a phantom 0, four whole-file test pins narrowed to the fields each situation is about, CRLF splice behavior documented and pinned, comment restatements deleted, the machinery types (`TaskAction`, `Transition`, `into_file`) made crate-private until a second caller exists.

Every new or changed test proved it could fail: two full red runs with deliberately wrong expectations (65+ tests red), restored to green. The red-proof itself caught one real defect (`insertion_index` stopping at a mid-file unknown key).

## Maintainer decisions folded in

- (approved 2026-08-27) A `routed-to` naming a type no answer becomes — a note or a question — is `broken-routing` on every surface. The rule lives in the record's own semantic pass (the target's type is written in its id, no second record needed), so `check()`, the mutation gate, and the corpus all enforce it; test-first, plus corpus case `invalid/questions/question.routed-into-a-note.md`.

## Open questions for the maintainer (not fixed blindly)

1. `close` leaves the record in the live directory; the archive move is l3's `archive` command. The format spec's "state and location can disagree only as a named finding" is therefore not yet enforced — c3/l3 territory, flagged so it is not lost. (Candidate for a Question record once anb self-hosts.)
2. `Notebook::new` takes `&mut Storage` even for reads — accepted friction for v1; revisit when c3's queries land.

## Also in this working tree

`AGENTS.md` — the maintainer-ordered protocol change (2026-08-27): the review phase is never performed by the authoring model; a separate agent reviews every coding task. Stage 3 rewritten; the Conventions bullet no longer duplicates it.
