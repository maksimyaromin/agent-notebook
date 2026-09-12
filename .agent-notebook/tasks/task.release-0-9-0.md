---
id: task.release-0-9-0
type: task
state: active
title: Release shared working memory as 0.9.0
by: Maksim Yaromin
via: codex
taken-by: Maksim Yaromin
from: task.shared-memory
created: 2026-09-12
updated: 2026-09-12
---

Package the shared-memory implementation in reviewable commits, replace same-day release counters with the complete package version, preserve published releases, and publish the maintainer-requested minor release through a pull request after CI succeeds. Verify GitHub artifacts and every npm package independently.
- 2026-09-12 Maksim Yaromin/codex: Prepared four commit boundaries: the shared-memory implementation and guides, reusable evaluations, release naming, and the versioned release. Cargo and all six npm manifests agree at 0.9.0; the registry has no 0.9.0 package yet. The complete local code and documentation gates pass, including release-tag acceptance and mismatch refusals. Existing changelog entries are byte-identical. Next: submit the branch, wait for CI, merge while preserving commits, then publish v2026.09.12.0.9.0 and verify GitHub and npm independently.
