---
id: note.report-cli-task-cycle
type: note
state: retired
title: Report: CLI: task cycle
by: Maksim Yaromin
via: claude-code
from: task.cli-task-cycle
created: 2026-08-29
updated: 2026-08-29
---

# l1 — CLI: task cycle — report

Closed 2026-08-29. Delivered against the interaction spec (§2–§6), the record-model spec (§3, §7, §8, §10), and the l1 maintainer rulings recorded in the task body.

## What shipped

**The `anb` binary grew from a stub to the full task-cycle surface**: `add start submit close return reopen hold unhold block unblock comment ready list view status`. `return` is included although the ticket's shorthand omitted it — the record-model transition table names it as a command and the invalid-transition payloads suggest it, so its absence would make the tool recommend a command that does not exist.

**Crate shape** — `crates/anb` became lib + thin main:
- `cli.rs` — the clap surface plus `Subject` (verb + id extracted before dispatch, so error payloads can point back at the refused command). Flags with domain semantics stay optional at the clap level and are judged by the Core/shell, so their refusals arrive as recovery payloads, not clap usage errors.
- `reply.rs` — `execute(Command, Storage, git_by, today) -> Reply`; one variant per outcome (`Held`/`Unheld`/`Blocked`/`Unblocked` are separate variants, not string tags). The hook's fail-soft lives here as `Reply::Silence`, which made it testable.
- `text.rs` / `json.rs` — the two renderings of every reply, sharing `Recovery` (code, message, details, computed `try:` lines) and the row bound (20, `--all` lifts it) from one home.
- `fs_storage.rs` — the fs adapter (atomic temp+rename writes, non-UTF-8 read is a named Io error) and `resolve_root`: nearest ancestor with `.agent-notebook`, the walk bounded by the first `.git` — a notebook above the repository is another project's.
- `main.rs` — wiring only: cwd, `jiff` local date, lazy `git config user.name` (forked only for `add`/`comment`), exit codes, trailing-newline termination.

**Output contract**: leading `ok:` with the transition and computed consequences (`unblocked[...]`, `open-questions[...]`, `superseded:`); replay renders `(already)`; flat lists as `count:` + header+rows tables with the one-line truncation hint; `view` as kv in envelope order + `body: |` + `mentions`/`mentioned-by`; `--json` compact everywhere (`preserve_order` so `ok` leads every object; absent fields omitted). Every error is `error[<code>]: message` + detail lines + `try:` commands computed from state (valid transitions, the cycle's own edges, the colliding id, the missing target), stderr, exit 1.

**`anb status`**: `--budget <n>` outranks the config key, `0` = no ceiling on both surfaces; `--json` carries the model; `--hook` emits the Claude Code SessionStart payload with the data-framing line and fails soft — any internal error yields empty output, exit 0.

**Core additions** (`anb-core`): `Notebook::comment` (log convention `- <date> <author>: <text>`, author = via else git identity else `-`; idempotent through the trail's tail; state does not gate the log), `Notebook::list` (live valid records, type-major order), `Notebook::view` (envelope in file order, body, mention blocks; a record never enters its own blocks; `mentioned-by` reads live records only), `RecordFile::fields()`, `encode` module (shared quoting rule + the ready table with its age rule — one home for the row the Status and `anb ready` both print), `grammar::day_number` and `notebook::path_stem` made public for hosts.

## Review (separate agent)

22 findings: 4 must-fix, 11 should-fix, 7 judgment calls. All fixed test-first except three held positions:

- Must-fix fixed: `try:` lines for the invalid-argument/would-cycle/archived/wrong-type/invalid-record classes; the ready row + age duplication moved into Core `encode`; the false `preserve_order` comment; the glossary-forbidden `Dashboard` name replaced with `Status` in identifiers and `--help`.
- Should-fix fixed: hook fail-soft into `execute`; stringly `command` tags split into variants; single `ROW_BOUND`/`shown` home; `Created.superseded` rendered; lazy git identity; split error arms; `path_stem` exported instead of re-derived; doc claims re-derived from the code; the test gaps (budget flag vs config key, hook fail-soft + contrast, `--json` bounding and field omission, block/unblock replies and replays, comment replay, invalid-record/archived/wrong-type payloads).
- Judgment calls acted on: `app.rs`→`reply.rs`, `render.rs`→`text.rs`; root walk bounded at the repository.
- Positions held, stated to the maintainer: CLI goldens keep pinning full payload text including Core message prose (the spec's testing decision makes stdout the behavior under the golden e2e pass); insta inline snapshots kept with hand-derived expectations; the command-vocabulary parse table kept as the one surface-contract guard.

## Verification

- Gate green after every round: `cargo fmt --check`, `clippy` pedantic `-D warnings`, 284 tests, doctests.
- Can-it-fail: 74 expectation flips across the new tests (55 first round, 19 after the review fixes), every one observed red, then restored.
- Binary smoke-tested end to end in a throwaway repo: full cycle, recovery payloads with `try:` lines, hook payload, `--json` surfaces, exit codes.
- Dependency versions verified on crates.io on 2026-08-28: jiff 0.2, serde_json 1.0 (`preserve_order`), tempfile 3.27 (dev), insta 1.48 (dev).

## Decisions recorded along the way

- `retire` gets a CLI surface in l2 (maintainer's call 2026-08-29); the spec's command vocabulary is amended, l2's body carries the ruling.
- Deliberately out of l1, flagged for later: lockfile concurrency (atomic write only for now — no ticket names the lockfile), `check`/`archive`/`search`/`overview` (l3), `decide`/`note`/`ask`/`answer` (l2), `via` auto-detection from host env (flag only; a skill/setup question for a1/a2).
- Known drift left in place: pre-existing committed Core prose and test names still say "dashboard" (glossary Avoid-list) — predates this diff, worth one sweep in a later text pass.
