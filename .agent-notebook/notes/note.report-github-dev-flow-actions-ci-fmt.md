---
id: note.report-github-dev-flow-actions-ci-fmt
type: note
state: active
title: Report: GitHub dev-flow: Actions CI (fmt + clippy + tests)
by: Maksim Yaromin
from: task.github-dev-flow-actions-ci-fmt-clippy-te
created: 2026-08-31
updated: 2026-08-31
---

# GitHub dev-flow: Actions CI (fmt + clippy + tests)

Task: task.github-dev-flow-actions-ci-fmt-clippy-te. Done 2026-08-31.

## What shipped

- `.github/workflows/ci.yml` — one job, one working step: `./scripts/check.sh` verbatim, so the commands live only in the script and CI cannot drift from the local gate. Triggers: push to main (post-merge verdict, primes the rust-cache entry PR branches restore from) and every pull request; the two never double-fire for one commit because push is scoped to main. `permissions: contents: read`; concurrency cancels superseded PR runs while every main push keeps its verdict; `timeout-minutes: 15`; actions pinned to commit SHAs (checkout v7.0.1, Swatinem/rust-cache v2.9.2). No toolchain action: `rust-toolchain.toml` is the single source of truth, and an explicit `rustup toolchain install` step materializes it before rust-cache keys on the rustc version. No nextest install: nextest is absent on the owner's machine, so CI proves the same `cargo test` path the local gate runs.
- `.github/dependabot.yml` — weekly `github-actions` updates, the companion to SHA-pinning: Dependabot rewrites both the SHA and the trailing version comment, so the pins do not fossilize.
- `scripts/check.sh` — `--locked` on every cargo command that resolves dependencies (clippy, nextest/test, doc-tests, rustdoc), so a `Cargo.lock` drifted from `Cargo.toml` fails the gate instead of being silently rewritten mid-run. Found during review, fixed beside the change.
- `AGENTS.md` — two lines: the PR-only convention for main (owner's call, 2026-08-31), and the Commands entry now says CI runs the same script.

## Branch protection (owner ruling, 2026-08-31)

GitHub refused the ruleset: private repo on the Free plan (HTTP 403, "Upgrade to GitHub Pro or make this repository public"). Owner chose to keep the repo private and unprotected for now — PRs by convention, server enforcement deferred. Filed as task.main-ruleset-when-plan-allows with the exact ruleset shape (require PR with 0 approvals, required check `check`, block force-push and deletion).

## Review (Opus 5, mandatory stage 3)

Findings, all applied:

1. rust-cache ran before any toolchain step, against the action's documented contract (it keys on the current rustc); toolchain auto-install would have happened as a side effect inside the cache step. Fixed: explicit `rustup toolchain install` (no-arg form installs the pinned toolchain) before rust-cache.
2. The concurrency comment restated the expression beneath it. Deleted.
3. The run-step comment claimed "CI fails exactly when a developer's local run fails" — false while check.sh branches on nextest presence. Replaced with the one non-derivable fact: commands live only in check.sh so CI and the gate cannot drift.
4. SHA pins without an update channel are pin-and-abandon. Fixed: dependabot.yml.
5. (nit, deliberately not applied) `save-if` on rust-cache to stop PR runs from writing cache entries — optimizing against an unevidenced constraint for a two-crate workspace; recorded here so the omission is a decision, not an oversight.
6. Out of the diff, filed instead of fixed: both crates carry zero doctests, so the gate's doctest step passes vacuously — logged as a comment on task.close-the-behaviours-the-test-audit. Missing `--locked` in check.sh — fixed here (above) since the gate is exactly what this task wires into CI.

Reviewer verifications worth keeping: the pinned rust-cache SHA is the commit behind the annotated tag v2.9.2, not the tag object; concurrency groups for PR refs (`refs/pull/N/merge`) and main are disjoint namespaces; actionlint validates the cancel-in-progress expression (probed with a deliberately broken field); the run step propagates check.sh's exit code (mode 100755 in the index, `set -euo pipefail`).

## State

actionlint clean; full gate green after every change (fmt, clippy, 143 tests, doctests, rustdoc, all `--locked`). Working tree left uncommitted for owner review. First landing must go through a PR per the new convention — and the `pull_request` trigger fires for the PR that introduces the workflow itself.
