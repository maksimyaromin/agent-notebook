---
id: note.report-why-rust-dossier-release-gate
type: note
state: retired
title: Report: Why-Rust dossier (release gate)
by: Maksim Yaromin
from: task.why-rust-dossier-release-gate
created: 2026-09-05
updated: 2026-09-05
---

# Why Rust: the release gate answered (2026-09-05)

Report for task.why-rust-dossier-release-gate. ADR 0006 carried a gate since 2026-08-25: before the first public release, three to five clear answers to "why Rust" that stand on technical or product merit, with "for fun" and "the owner said so" ruled out. The owner asked for the answers to be drafted from what the codebase shows, to be accepted or reopened at the final review.

## What the codebase shows

| Measure | Value | Where it comes from |
|---|---|---|
| Release binary | 1.7 MB, linking only the system library | `cargo build --release`, `otool -L` |
| Startup | under a millisecond once cached; 0.39 s on the first cold run | `time anb status --budget 0`, three runs |
| Runtime dependencies | 4 (the Core, clap, jiff, serde_json); 77 packages in the lock file | `Cargo.toml`, `Cargo.lock` |
| Safety and lint policy | `unsafe_code = "forbid"`; clippy `all` and `pedantic` as warnings, promoted to errors by the gate | workspace lints, `scripts/check.sh` |
| Tests | 666, unit and integration, with property tests over generated notebooks | the test suites |
| Source | about 14,000 lines of code and 15,000 of tests | `wc -l` |

## The five answers

Written into ADR 0006 under "Why Rust: the gate answered". In one line each: the tool runs inside every agent session, so it must cost nothing to start and nothing to install; the record model is invariants and the compiler holds them through exhaustive enums and typed results; the markdown file is the source of truth and byte-exact round-tripping is Rust's default rather than a discipline; one Core serves many hosts, including a `wasm32` build, with no runtime to carry; a small auditable surface for a tool with write access to every repository it serves. The ADR says plainly which answers Go would share (the first and the last) and where Rust specifically earns the choice (the middle three).

## Left to the owner

Acceptance at the final review. If an answer does not convince, the ADR's own rule applies: the decision is reopened before the release, not after.
