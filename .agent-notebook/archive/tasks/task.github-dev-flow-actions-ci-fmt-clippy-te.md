---
id: task.github-dev-flow-actions-ci-fmt-clippy-te
type: task
state: closed
title: GitHub dev-flow: Actions CI (fmt + clippy + tests)
by: Maksim Yaromin
via: claude-code
tags: dist
link: note note.report-github-dev-flow-actions-ci-fmt
blocked-by: task.milestone-cli-complete
created: 2026-08-29
updated: 2026-08-31
closed: 2026-08-31
---

GitHub Actions workflow running scripts/check.sh verbatim (fmt --check, clippy -D warnings, tests, doctests) on push/PR. Deferred; tracked here so the skill CI drift check and the release gate cannot land without it.
- 2026-08-31 Maksim Yaromin: CI workflow written: .github/workflows/ci.yml — push to main + PRs, least-privilege permissions, concurrency cancel for superseded PR pushes (main always gets a verdict), actions SHA-pinned (checkout v7.0.1, rust-cache v2.9.2), single step runs scripts/check.sh verbatim. No toolchain action: rust-toolchain.toml is the one source of truth, rustup auto-installs it. No nextest install: it is absent locally, so CI takes the same cargo-test path as the maintainer's gate. actionlint clean, local gate green.
- 2026-08-31 Maksim Yaromin: Maintainer ruling: main stays unprotected for now — GitHub rulesets need a public repo or Pro plan (API answered 403 on a private Free repo). Changes travel through PRs by convention; the ruleset (PRs only, required check 'check', no force-push/deletion) is deferred until the repo goes public or the plan upgrades.
- 2026-08-31 Maksim Yaromin: The independent review returned 4 should-fix + nits; all applied: explicit 'rustup toolchain install' before rust-cache (the action keys on current rustc), concurrency comment deleted (restated the expression), run-step comment rewritten to the one non-derivable fact, dependabot.yml added for the SHA pins. Beside the change: --locked added across check.sh; doctest vacuity (0 examples in both crates) logged on task.close-the-behaviours-the-test-audit. save-if on rust-cache deliberately omitted. Gate green after every change;
- 2026-08-31 Maksim Yaromin: Maintainer returned a style finding: the --locked comment in check.sh violated the comment rules — it restated documented flag semantics ('--locked everywhere: drifted Cargo.lock fails instead of being rewritten'), and 'everywhere' was falsified by the fmt line right under it, which takes no --locked. Deleted; the flag speaks for itself. Ruling recorded: comment rules bind every file, configs and scripts included.
