---
id: note.report-rust-scaffold
type: note
state: retired
title: Report: Rust scaffold
by: Maksim Yaromin
via: claude-code
from: task.rust-scaffold
created: 2026-08-29
updated: 2026-08-29
---

# g1 — Rust scaffold: report

## Delivered

- Cargo virtual workspace (resolver 3, edition 2024): `crates/anb-core` (zero runtime dependencies) + `crates/anb` (clap CLI skeleton).
- Storage seam in `anb-core`: `Storage` trait (list/read/write/remove over strings and relative paths), `StorageError` (NotFound/Io), `MemoryStorage` adapter.
- Test harness: 6 unit tests + 2 proptest properties (byte-exact round-trip; mutation isolation); every test proven able to fail.
- Gates: `scripts/check.sh` — fmt --check, clippy (all+pedantic, -D warnings, unsafe forbidden), tests, doctests; nextest-aware.
- Toolchain pinned: Rust 1.98.0 (latest stable, verified online 2026-08-26); clap 4.6, proptest 1.11, insta 1.48 pinned the same way.

## Decisions from the review

- MSRV tracks the latest stable release until the first public release (comment on `rust-version` in the workspace Cargo.toml).
- insta stays pinned though unused until the first CLI golden test.
- CI tracked-and-deferred: ticket `ci` (Actions running scripts/check.sh verbatim); `a2` and `rel` blocked by it.
