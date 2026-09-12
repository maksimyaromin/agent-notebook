---
id: task.release-0-9-0
type: task
state: closed
title: Release shared working memory as 0.9.0
by: Maksim Yaromin
via: codex
taken-by: Maksim Yaromin
from: task.shared-memory
created: 2026-09-12
updated: 2026-09-12
closed: 2026-09-12
---

Package the shared-memory implementation in reviewable commits, replace same-day release counters with the complete package version, preserve published releases, and publish the maintainer-requested minor release through a pull request after CI succeeds. Verify GitHub artifacts and every npm package independently.
- 2026-09-12 Maksim Yaromin/codex: Prepared four commit boundaries: the shared-memory implementation and guides, reusable evaluations, release naming, and the versioned release. Cargo and all six npm manifests agree at 0.9.0; the registry has no 0.9.0 package yet. The complete local code and documentation gates pass, including release-tag acceptance and mismatch refusals. Existing changelog entries are byte-identical. Next: submit the branch, wait for CI, merge while preserving commits, then publish v2026.09.12.0.9.0 and verify GitHub and npm independently.
- 2026-09-12 Maksim Yaromin/codex: Published package version 0.9.0 and GitHub release v2026.09.12.0.9.0 from the reviewed merge commit. PR https://github.com/maksimyaromin/agent-notebook/pull/88 preserved the four implementation, evaluation, release-policy and version commits after green code and documentation checks. Release workflow https://github.com/maksimyaromin/agent-notebook/actions/runs/34715568746 succeeded for all five platforms and both publication jobs. Downloaded every GitHub archive and verified SHA256SUMS; all six npm packages expose 0.9.0 with integrity metadata. The global npm installation reports anb 0.9.0 and passed an isolated create, start, comment, close, archive and check sequence. All nine previously published releases retained their names and asset identities; historical changelog entries remained byte-identical. Release: https://github.com/maksimyaromin/agent-notebook/releases/tag/v2026.09.12.0.9.0.
