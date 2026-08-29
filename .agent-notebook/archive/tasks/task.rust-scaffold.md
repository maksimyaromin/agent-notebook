---
id: task.rust-scaffold
type: task
state: closed
title: Rust scaffold
by: Maksim Yaromin
via: claude-code
tags: scaffold
link: report .tmp/data/g1/report.md
created: 2026-08-24
updated: 2026-08-26
closed: 2026-08-26
---

Cargo workspace: anb-core (dependency-light library behind the Storage trait, no fs/git/network) + anb CLI crate; test harness; clippy + rustfmt; local check script; Storage trait skeleton.

Progress 2026-08-26: scaffold built and green, in review pause (uncommitted). Review artifact: .tmp/lavish/g1-rust-scaffold.html (Lavish session live).
- Versions verified online 2026-08-26 (owner's rule: modern stack, internet-sourced): Rust 1.98.0 stable (2026-08-20), edition 2024, resolver 3, clap 4.6.6 (no clap 5), proptest 1.11.0, insta 1.48.0, cargo-nextest optional in check script.
- Files: rust-toolchain.toml (1.98.0 + clippy/rustfmt), Cargo.toml (virtual workspace, lints: unsafe forbid, clippy all+pedantic warn, -D warnings in check), crates/anb-core (zero runtime deps; storage.rs = Storage trait + StorageError + MemoryStorage; tests/storage_props.rs = proptest round-trip + isolation), crates/anb (clap derive skeleton), scripts/check.sh, .gitignore +/target/.
- Owner issued the engineering instruction (AGENTS.md "Engineering instruction" -> .tmp/docs/engineering-instruction.md). Scaffold swept against it: restating/false/diary comments removed, user-facing message de-jargoned, all 8 tests proven able to fail (flipped -> 8 reds -> restored). Gate green after sweep.
- Grill round 1, all three DECIDED: Q1 keep rust-version 1.98, policy "MSRV tracks latest stable until the first public release" (comment on rust-version in workspace Cargo.toml). Q2 keep the insta pin. Q3 CI tracked-and-deferred: ticket `ci` added (Actions running scripts/check.sh verbatim), a2 and rel blocked by it so CI cannot be missed.
- Grill frontier is empty. Awaiting the owner's approval to close g1.
