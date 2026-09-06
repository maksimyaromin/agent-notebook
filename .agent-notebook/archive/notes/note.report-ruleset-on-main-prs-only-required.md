---
id: note.report-ruleset-on-main-prs-only-required
type: note
state: retired
title: Report: Ruleset on main: PRs only, required check, once the plan allows it
by: Maksim Yaromin
from: task.main-ruleset-when-plan-allows
created: 2026-09-06
updated: 2026-09-06
---

# Ruleset on main

main moves only by pull request with a green CI check. The branch ruleset `main`, active on the default branch, holds four rules: deletion refused, force-push refused, a pull request required with zero approvals (a solo repository cannot approve itself), and the status checks `check` and `docs` from GitHub Actions required before merge. No bypass actor is listed, so the rule binds the maintainer too. The extra approval GitHub adds by default for unattributed changes is switched off; it would demand an approval nobody can give.

The rules in force are listed by `gh api repos/maksimyaromin/agent-notebook/rules/branches/main`: deletion, non_fast_forward, pull_request with required_approving_review_count 0, required_status_checks with check and docs. The ruleset became possible on 2026-09-06 when the repository went public; GitHub refuses rulesets on a private repository without a paid plan.
