---
id: note.report-ground-the-public-documentation
type: note
state: retired
title: Report: Ground the public documentation in the product and its working method
by: Maksim Yaromin
from: task.public-docs-review
created: 2026-09-06
updated: 2026-09-06
---

# Public documentation review

The documentation now explains the purpose of the supplied method, the record rules that make it dependable, and how development of agent-notebook uses its own notebook to find the next improvement. The design page includes the concrete reason report Notes became the default proof: a report in an uncommitted directory cannot be read from another clone. It preserves the equal standing of other proof choices and of private notebooks.

The contribution guide addresses developers who already know their craft. It describes the Core/Storage/CLI division, the difference between a record invariant and a workflow choice, multi-file concurrency, byte preservation, bounded replies and archive-aware performance checks. It invites evidence from actual sessions and custom skills. Generic submission and formatting instructions were removed from the public guide; repository conventions remain in AGENTS.md.

The release guide follows the actual workflow: GitHub release creation and npm publication are independent jobs, publication uses Trusted Publishing, and a partially published npm version cannot be resumed by blindly rerunning the package loop. The quickstart includes native release archives alongside npm and source installation. GitHub's public API confirmed repository visibility and the v2026.09.06 assets. The npm registry confirmed 0.1.1 and its GitHub Actions Trusted Publisher. The documentation site returned successfully.

Source verification corrected the Status reference: lost-proof reports external commit and file links in the working set; a missing report Note is a notebook reference finding. The Task guide explains incremental repairs. The skill source and generated copy now agree with the implemented origin edit, Status row limits and retry templates. No runtime behavior changed.

Verification passed: scripts/check.sh, including generated skill and reference drift checks; pnpm docs:check with no warnings, 18 book pages reachable and all checked links resolving; the 11 literal quickstart command replies against a scratch notebook; the private-notebook and custom-skill recipes against a scratch repository; release version agreement and changelog extraction; README site routes against the local build; git diff --check; anb check. No commit, push, deployment or publication was performed.
