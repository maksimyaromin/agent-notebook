---
id: note.report-serialize-concurrent-mutations-on
type: note
state: retired
title: Report: Serialize concurrent mutations on a notebook lock
by: Maksim Yaromin
from: task.serialize-concurrent-mutations-on-a
created: 2026-08-30
updated: 2026-08-30
---

# Serializing concurrent mutations on a notebook lock

## The hazard, measured

Against a build without the lock, in a fresh notebook, twenty processes launched at once:

| what ran | expected | landed | exit codes |
|---|---|---|---|
| `anb comment task.probe "note-N"` ×20 | 20 log entries | 7 | all 0 |
| `anb add "Parallel item"` ×20 | 20 records | 7 files | all 0, each reporting a distinct id |

Every verb reads the records it needs, splices them, and writes the whole file back. Two runs that overlap on that window read the same bytes, and the second write erases the first. The atomic write the adapter already does — temp file, rename — prevents a torn file and nothing else; it makes the loss clean rather than visible. The `add` case is worse than the `comment` case: each caller was told `ok: add task.parallel-item-03`, and the file at that name held another record entirely.

A second, subtler hazard has the same root. A settling verb moves several files in turn — `archive` writes the destination and then removes the source, `expunge` removes a record and its reports, a supersession writes the superseder and then flips the victim — and a reader landing between two of those files reports the half-finished state as corruption. Measured with one process archiving 60 closed Tasks and another running `check` and `list` in a loop: **7 spurious reader failures in 80 reads** — `check` reporting `duplicate-id` on a record that was merely mid-move, and `list` failing outright with `not found`.

## The shape

`crates/anb/src/lock.rs`. A writer takes an exclusive advisory lock (`std::fs::File::lock`, stable since Rust 1.89) on `<root>/.lock` before the command runs; a reader takes a shared one. The claim is released when the process ends, however it ends, because the kernel holds it — so a crash leaves no stale file for the next run to reason about, and no verb grows a retry story.

- `lock::writes` classifies every verb in one exhaustive match, so a new verb cannot compile until it is classified.
- The reader's claim is taken only if the lock file is already there, and nothing on the read path creates it: a notebook no writer has ever touched has no one to wait for, and a notebook mounted read-only stays readable.
- A writer that cannot take its claim reports a storage failure rather than writing anyway. A medium that reports no error and still fails to serialize — some network filesystems — is beyond what any caller can detect, and the doc says so instead of promising otherwise.

Why the kernel's lock rather than a compare-and-swap write through the Storage seam: it costs the Core nothing. A compare-and-swap would push a conflict outcome through the seam, and every adapter would then have to implement it. The Core keeps no lock and knows of none.

Why blocking rather than a timeout or a polling retry: nothing but `anb` ever takes this lock, and only for one mutation window, so the wait is bounded by the writers ahead of you. A timeout would buy a failure mode instead of removing one.

## Cost

At 1 000 live records, median of 15 runs, release build:

| verb | without the lock | with it |
|---|---|---|
| `comment` (exclusive) | 10.6 ms | 10.6 ms |
| `add` (exclusive) | 50.6 ms | 51.5 ms |
| `list` (shared) | 42.1 ms | 42.1 ms |
| `status` (shared) | 43.4 ms | 43.6 ms |

One `open` and one `flock` on a directory the command is about to read anyway. It does not show above the noise.

## The leavings, and git

The lock file carries no `.md` and sits in the root rather than in a type directory, so no listing, query, or check can meet it. A notebook committed with its project would still carry it, and would also carry the temp file an interrupted write leaves behind (`.<name>.<pid>.tmp`, in the target's own directory because a rename must not cross filesystems). So the first writer creates a `.gitignore` in the notebook root naming both — with `create_new`, so a notebook that already states its own rules keeps them and no window exists in which two writers race for the file. A root that refuses the file works exactly as well; only git sees the difference.

## What it costs elsewhere

A refused first command now materializes the notebook: the root and its two hidden files appear before the verb is judged, so `anb start task.absent` in a fresh directory leaves an empty notebook behind. Where no ancestor holds a notebook or a `.git`, that directory then anchors every later default-rooted command. The alternative — removing the root when the command fails — would unlink the lock file out from under a waiting process and break the one property the whole design rests on, so the leavings stay and `lock::taken` documents them.

## Durability: deliberately not fsynced

Measured on this machine (APFS), 200 writes of a 2 KB record:

| recipe | per write |
|---|---|
| write temp + rename (what anb does) | 0.50 ms |
| + `sync_all()` before the rename | 4.55 ms |
| + directory fsync after the rename | 8.04 ms |

`sync_all` on macOS issues `F_FULLFSYNC`, and a mutation writes one to three files, so full durability would cost 4–14 ms on a command whose whole budget is 11 ms. What it buys is surviving a power loss in the second after a write. The notebook is markdown under git; losing the most recent comment to a power cut is recoverable, and the guarantee that matters — a reader never sees half a record — is what the rename already gives. The adapter's doc claims exactly that and no more.

## Verification

- 30 concurrent comments → 30 entries; 30 concurrent `add`s → 30 records, `check` clean; 50 of each, likewise. The reader loop against a running archive cascade: 0 spurious failures in 80 reads, against 7 without the lock.
- Nine process-level tests in `crates/anb/tests/concurrency.rs`, including the classification asked of clap's own subcommand list rather than of the author, so a verb added and left unclassified fails there.
- A 28-command mutation script over four seed notebooks — empty, 100 live records, 500 live records, and this project's own — run against a build without the lock and this one: replies and resulting trees byte-identical, the two new hidden files excepted.

## The Decision this replaces

`decision.concurrent-mutations-serialize-on-a` held that read-only commands take nothing and are never blocked. The reader failures above are the evidence against it; the superseding Decision gives readers a shared claim.
