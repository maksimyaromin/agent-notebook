---
id: note.report-restore-verb-bring-a-record-back
type: note
state: retired
title: Report: Restore verb: bring a record back out of the archive
by: Maksim Yaromin
from: task.restore-verb-bring-a-record-back-out-of
created: 2026-08-31
updated: 2026-08-31
---

# Report: Restore verb — bring a record back out of the archive

Task: task.restore-verb-bring-a-record-back-out-of. Gate green (`./scripts/check.sh`: fmt, clippy -D warnings, all tests, doctests), smoked live on a scratch notebook.

## What shipped

`anb restore <id>` is the inverse of `archive`: it moves `archive/<dir>/<id>.md` back to `<dir>/<id>.md` — same filename, same bytes, the record alone. The reports the archive move carried are retired history and stay in the archive.

The contract, clause by clause:

- **The bytes travel unjudged past one bar, readability.** No verb that corrects a record resolves an archived id (`expunge` reaches the archive only to delete), so restore is what makes a finding on an archived record repairable at all. A record carrying error findings moves back verbatim; a NotUtf8 file refuses on either side of the move, because the verb promises residence and cannot vouch it over bytes no parse can read.
- **A live file under the id is never overwritten.** The live directory is the only editable home, so what stands there is the record's current truth. When both homes hold the id — the leftover of a move interrupted in either direction — restore finishes the move by removing the leftover, provided both files answer for the record: byte-identical (an untouched interrupted copy, however broken), or the standing file parses clean (a canonical live path admits no clean record but the id's own, and `add` refuses an id the archive claims) *and* the leftover's own bytes declare the id. Anything else refuses `duplicate-id`.
- **Write before remove**, so a crash between the two leaves a loud duplicate, never a lost record — mirroring archive's doctrine, and pinned by a test with a failing-remove adapter.
- **Replay answers `already`** when the id lives and the archive holds nothing; the promise is residence, not validity.
- **check names the move**: `archived-live-record` now carries `repair: anb restore <id>` — the one finding restore erases; other findings on archived files still name nothing (their erasers refuse to run until the record is back). The `error[archived]` refusal offers `try: anb view <id>` then `try: anb restore <id>` — view first, because restore on a correctly filed record manufactures an `unarchived-settled-record` warning.

## Mandatory review (Opus 5, separate agent)

Verdict: 0 defects, 9 should-fixes, 10 nits. All 9 should-fixes and 8 of 10 nits are fixed; the notable ones:

1. **"restore is the only verb reaching the archive" was false** — `expunge` resolves both homes. Swept out of five homes (restore's doc, check's two comments, the user-facing finding message — reverted to the original true sentence — and a test doc).
2. **Stale rationale in two more homes**: `finding.rs`'s severity-split doc and `debt.rs`'s "no verb reaches into the archive" — both restated as the reasons that still hold.
3. **The gate's header claimed "every write passes through here"** — restore now writes outside it by design; the header names the exception and why.
4. **A real hole in the resume test**: the both-homes branch removed an archived leftover it never read, so foreign bytes under this filename (a hand-move's only copy) could be destroyed and reported as a move. The resume test now also requires the leftover's own bytes to declare the id; a new test pins the refusal.
5. **The 64-line function was buried under its I/O** — reshaped into `held_at` (one home's answer as data: `Absent | Bytes | Unreadable`) and a flat seven-arm truth table, with the resume test as a named free function carrying its two tiers and their reasons.
6. **Two missing behaviors specified**: write-before-remove (RemoveFails adapter, promoted to the shared test harness beside BinaryHolding), and the headline promise end to end — an archived record with a stray `priority: 9` restores, `check` names `edit --clear priority`, the eraser runs, `check` is clean.

Not applied, deliberately: a separate reply signal for "leftover dropped" vs "bytes moved" (no caller branches on the difference today; the reply's outcome — the id lives only live — is the same), and splitting one two-assert test further. Correction to an earlier log line: the CLI suite gained three tests, not two.

## Tests

19 core behaviors in `crates/anb-core/tests/notebook/restore.rs` plus 3 CLI tests and 5 updated ones (lock table, scope table, repair round-trip, recovery snapshot, check-repair naming). Every new test proved it can fail by deliberately wrong expectations before counting.

## For the owner to confirm

The live Decision `decision.a-repair-is-progress-not-perfection` still says "a record in the archive … has no move at all, whatever is wrong inside it — and a row that named one anyway would send an agent in a circle." Restore falsifies the clause: the archived canonical path now has exactly one move, and check names it on the residence finding. Proposed: supersede with the same decision amended — repair-is-progress and read-off-the-line stand unchanged; the reachability clause becomes "every correcting verb resolves an id to its one live path; the archived path is reached only to be moved back or deleted, so the one repair ever named on an archived file is `restore`, on the residence finding it erases." Not written to the notebook — awaiting confirmation.
