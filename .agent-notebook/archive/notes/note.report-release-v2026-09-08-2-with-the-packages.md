---
id: note.report-release-v2026-09-08-2-with-the-packages
type: note
state: retired
title: Report: Release v2026.09.08.2 with the packages at 0.6.0
by: Maksim Yaromin
from: task.release-v2026-09-08-2-with-the-packages
created: 2026-09-08
updated: 2026-09-08
---

# Report: release v2026.09.08.2

The packages moved to 0.6.0 through #77; the tag `v2026.09.08.2` is the day's third release and follows the merge commit `da979e5`.

## What shipped

The holder ruling of #76: a Task belongs to who holds it and `mine` is `taken-by`, `add task --taken-by` and `--mine` hand a Task over as it is written, `--untaken` is the pool on `ready`, `list` and `graph`, a narrowed Status counts the pool on its `untaken:` line, and the skill takes from the pool when the user's queue is empty.

## Evidence

- `scripts/release/check-versions.sh`: every manifest at 0.6.0.
- `scripts/release/changelog-notes.sh v2026.09.08.2` extracted the entry; `check-tag.sh` accepted the tag.
- PR #77 CI green (`check`, `docs`), squash-merged, branch deleted.
- Release workflow run 34257102845 green: five platform builds, the GitHub release with five archives and `SHA256SUMS`, the six packages published.
- The two arm64 packages reached the registry about a minute after the publish job, as on the previous release; from an empty directory `npx -y --prefer-online @supolka/agent-notebook@0.6.0 --version` prints `anb 0.6.0`.
