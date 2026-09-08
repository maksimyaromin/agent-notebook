---
id: note.report-release-v2026-09-08-1
type: note
state: retired
title: Report: Release v2026.09.08.1 with the packages at 0.5.0
by: Maksim Yaromin
from: task.release-v2026-09-08-1
created: 2026-09-08
updated: 2026-09-08
---

# Report: release v2026.09.08.1

The packages moved to 0.5.0 through #73; the tag `v2026.09.08.1` is the day's second release and follows the merge commit `9669594`.

## What shipped

The read-side revision of #72: one narrowing shared by `list`, `ready` and `graph`, `search` and `overview` folded into `list`, `graph --ready` gone, Status as the work with `--mine`, `--by`, `--team` and the `scope` config key, the open Questions on the dashboard, `anb debt`, and the graph document at version 3.

## Evidence

- `scripts/release/check-versions.sh`: every manifest at 0.5.0.
- `scripts/release/changelog-notes.sh v2026.09.08.1` extracted the entry; `check-tag.sh` accepted the tag.
- PR #73 CI green (`check`, `docs`), squash-merged, branch deleted.
- Release workflow green: five platform archives and `SHA256SUMS` on the GitHub release, the six packages published with provenance.
- The registry served the two arm64 packages at 0.5.0 about three minutes after the publish job reported them, so an install made inside that window pinned the launcher without its platform package; from an empty directory `npx -y --prefer-online @supolka/agent-notebook@0.5.0 --version` prints `anb 0.5.0`.
