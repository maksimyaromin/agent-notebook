---
id: task.github-dev-flow-actions-ci-fmt-clippy-te
type: task
state: open
title: GitHub dev-flow: Actions CI (fmt + clippy + tests)
by: Maksim Yaromin
via: claude-code
tags: dist
blocked-by: task.milestone-cli-complete
created: 2026-08-29
updated: 2026-08-29
---

GitHub Actions workflow running scripts/check.sh verbatim (fmt --check, clippy -D warnings, tests, doctests) on push/PR. Deferred; tracked here so the skill CI drift check and the release gate cannot land without it.
