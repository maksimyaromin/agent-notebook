---
id: note.report-core-status-budget
type: note
state: retired
title: Report: Core: Status + Budget
by: Maksim Yaromin
via: claude-code
from: task.core-status-budget
created: 2026-08-29
updated: 2026-08-29
---

# c4 — Core: Status + Budget (2026-08-28)

Delivered in anb-core, all at the Notebook seam: `status(today, budget)` and `config()`.

## Maintainer rulings folded in (2026-08-28)

- Debt in Status is the full set, mention scan included; record-model §8's "never during Status assembly" superseded (spec amended, with org-roam's scan-cost caution kept as the falsification trigger).
- Status carries a rules section (US12): live Decisions of kind `rule`, oldest first.
- `ready` stays oldest-first within a priority; the interaction §2 worked example was the wrong side and is re-sorted (closes the c3 open question).
- Budget resolution: CLI flag > config `budget` > default 1500; `0` = no ceiling on both surfaces; the budget line always prints and carries the restore command. The Budget bounds dashboard degradation only — reading a whole Task log is `view`'s surface (l1), never Status.
- `anb status` (with `--budget`, `--json`, `--hook`) recorded on ticket l1.

## What shipped

- `status.rs` — the gated dashboard: quiet one-liner without signal (in-flight ∨ review ∨ ready ∨ debt); full composite otherwise: counts, in-flight lines (+ last log line), review-waiting list, rules, ready top-5 with truncation hint, debt lines, budget line. Degradation ladder as a `Collapse` enum (out-of-order states unrepresentable): ready rows → debt→count → rules→count → log dropped + review→count → floor (counts + in-flight + budget line; the floor ships even over the ceiling, reported honestly). The budget line's spent number converges by a bounded fixed-point loop.
- `tokens.rs` — deterministic token estimate `ceil((6·ascii + 7·non_ascii)/21)` (≈ bytes/3.5 ASCII, /3.0 non-ASCII). Calibrated 2026-08-28 against o200k_base via gpt-tokenizer: status-shaped fixtures land +3–6% above the true count, never under on the calibration set; long English prose overshoots up to +50% (safe side); CJK can undercount — documented limit. Calibration constants live in the unit tests as measured truth with tolerance bands.
- `debt.rs` — every signal computed at read from envelope dates, nothing stored: task-stale (7d), question-age origin-keyed (14d free-standing / 7d task-born), origin-closed (immediate), hold-quiet (14d, wins over task-stale on held tasks), review-wait (7d), review-due (`review-by` ≤ today, any record), dangling-mention and undeclared-pair (mention scan, by/via attributions printed, oldest first), invalid (error findings, excluded from clocks). Archive never ages.
- `mention.rs` — hand-rolled id-token scan (no regex dependency): word boundaries both sides (`subtask.x`, `task.fooBar` cite nothing; `(task.x)`, sentence-final `task.x.` cite), id grammar enforced, dedupe, UTF-8-safe.
- `config.rs` — flat `key: value` in the envelope line grammar: `format`, `budget`, five `debt-*` thresholds; unknown-field / bad-value / duplicate-field / bad-envelope-line findings through `check`; every key defaults, fail-soft.
- `grammar.rs` — `day_number` (Hinnant days-from-civil); `parse_field_line` shared with config.
- `notebook.rs` — `status()`, `config()`, `check()` covers the config file.

## Tests

Gate green: fmt, clippy -D warnings (pedantic), 198 tests. 46 new behavior tests at the Notebook seam (status_dashboard, debt_signals, budget_ladder, notebook_config), a budget property (any notebook fits any ceiling or stands on the floor), estimator calibration tests with measured truth constants, and the regression fixture of the spec's budget test: a 60-task notebook's full dashboard pinned ≤ 350 estimated tokens (measured truth 212 + added sections + calibrated overshoot), fitting the default 1500 with nothing cut. Every new test proven able to fail: 53-flip expectation sweep, all red, all restored.

## Decisions taken here (not fixed by spec) — flagged for review

- Rules alone do not open the gate — a standing rule is not work in motion.
- Counts count live record files by their `type` field; a file too broken to name its type is visible as an invalid debt line, not in counts.
- Config creation on first write ("config is created on first write with `format` only", format §9) deferred to the CLI ticket l1 — no Core mutation currently creates the notebook root.
- Exclusion from derived queries has two adapters over one rule: `notebook::exclusion_errors` probes storage (mutation path reads one record), `debt::is_excluded` answers from the stem map (full-notebook passes) — same contract, each documented at its seam.

## Calibration harness

`gpt-tokenizer` o200k_base via node. Measured bands recorded in `tokens.rs` tests. Re-run recipe: encode the test strings with `gpt-tokenizer/esm/encoding/o200k_base` and compare with `estimate_tokens`.

## Review

The mandatory review pass returned 28 findings: 7 must-fix defects, 4 spec questions (3 resolved by maintainer ruling, 1 by the spec itself), comment-law and dialect items, and test gaps. All acted on:

- **Defects fixed (test-first, each red before green):** the estimator's calibration claim rewritten to what the code delivers (+6% composites / +20% short lines / +50% prose) and the harvested per-test bands replaced by two policy constants; the exclusion rule's two adapters reconciled — every derived query (`ready`, `close` consequences, Status, Debt, `check` refs) now answers from one `resolvable_by_id` map with the mutation gate's canonical-path semantics, so a record in the wrong directory resolves nothing anywhere; the dead `DebtThresholds::default` deleted (config's key table is the one home for defaults); `stem` deduplicated onto `path_stem`; the cut note no longer names a log line that never existed and now names the collapsed review list; an invalid Decision can no longer enter an undeclared pair.
- **Maintainer rulings (2026-08-28, folded into the interaction spec):** review Tasks ratified as the gate's fourth signal; mention-borne Debt classes bounded at five lines with a one-line hint; the restore command prints only when something was cut.
- **Spec-compliance fix:** every Debt clock now ticks only on records in a live state (§6 "currently open records") — a closed Task with a leftover `hold:` or a superseded Decision with a past `review-by` is history, not Debt.
- **Also per review:** `Status`/`Config` derive `Debug`/`PartialEq`/`Eq`; the in-flight title is JSON-escaped like every quoted value; `status` reads the notebook once; pairs rank by the older member's `created`; day arithmetic rejects a malformed timestamp clock; the config integer overflow case is named as out-of-range; `(1 error)` pluralizes; borrowed-key sorts. Mention-scanner boundary tests added at the Notebook seam (c3 precedent) rather than in-module as suggested.
- **Left as is, stated:** `spent` is the sum of per-part estimates — documented, and pinned by a test to land within one token above the whole-text estimate, never under; the fixed-point budget-line loop caps at three passes with the last measure winning, stated in its doc.

Gate after all fixes: fmt, clippy -D warnings, 215 tests; every new test proven able to fail (53 + 11 expectation flips, all red, all restored).
