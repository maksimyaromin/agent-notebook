---
id: task.main-ruleset-when-plan-allows
type: task
state: closed
title: Ruleset on main: PRs only, required check, once the plan allows it
by: Maksim Yaromin
via: claude-code
from: task.github-dev-flow-actions-ci-fmt-clippy-te
tags: dist
link: note note.report-ruleset-on-main-prs-only-required
priority: 3
created: 2026-08-31
updated: 2026-09-06
closed: 2026-09-06
---

GitHub refuses rulesets on a private Free-plan repo (HTTP 403: upgrade to Pro or make the repository public). Until then main is guarded by convention only: pull requests with a green 'check', no direct push. When the repo goes public or the plan upgrades, create the branch ruleset on main: require a pull request (0 approvals — a solo repo cannot self-approve), require the status check 'check' (GitHub Actions, integration 15368), block force-push and deletion. The exact API call is logged on the origin task.
- 2026-09-06 Maksim Yaromin: Repository public since 2026-09-06; ruleset main created on the default branch: deletion and force-push refused, pull request required with zero approvals, status checks check and docs required, no bypass actor. Verified through the branch rules endpoint.
