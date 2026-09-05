---
id: task.main-ruleset-when-plan-allows
type: task
state: open
title: Ruleset on main: PRs only, required check, once the plan allows it
by: Maksim Yaromin
via: claude-code
from: task.github-dev-flow-actions-ci-fmt-clippy-te
tags: dist
priority: 3
hold: waits for the repository to go public, which the owner does after README, the docs site and the history scrub are in place; the marathon leaves the ruleset script ready to run
created: 2026-08-31
updated: 2026-09-05
---

GitHub refuses rulesets on a private Free-plan repo (HTTP 403: upgrade to Pro or make the repository public). Until then main is guarded by convention only: pull requests with a green 'check', no direct push. When the repo goes public or the plan upgrades, create the branch ruleset on main: require a pull request (0 approvals — a solo repo cannot self-approve), require the status check 'check' (GitHub Actions, integration 15368), block force-push and deletion. The exact API call is logged on the origin task.
