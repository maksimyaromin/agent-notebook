---
id: note.report-release-v2026-09-08-3-with-the-packages
type: note
state: retired
title: Report: Release v2026.09.08.3 with the packages at 0.7.0
by: Maksim Yaromin
from: task.release-v2026-09-08-3-with-the-packages
created: 2026-09-08
updated: 2026-09-08
---

# Report: release v2026.09.08.3

The packages moved to 0.7.0 through #81; the tag `v2026.09.08.3` is the day's fourth release and follows the merge commit `18f032f`.

## What shipped

The link edge and the addressee of #80: a `link` to a record id is an edge under the link's own kind, listed by `show` as `linked-by`, drawn by `graph` at document version 4 and reached by `--for` and `--focus`; a `to` field names whom a Task or a Question waits on, written by `add --to`, `submit --to` and `edit --to`, read by `--mine`, `--by` and `scope: mine`, shown by the review table and the questions table and narrowed by `list --to`.

## Evidence

- `scripts/release/check-versions.sh`: every manifest at 0.7.0.
- `scripts/release/changelog-notes.sh v2026.09.08.3` extracted the entry; `check-tag.sh` accepted the tag.
- PR #81 CI green (`check`, `docs`), squash-merged, branch deleted.
- Release workflow run 34268648134 green: five platform builds, the GitHub release with five archives and `SHA256SUMS`, the six packages published.
- The two arm64 packages reached the registry after the publish job, as on the previous releases; from an empty directory `npx -y --prefer-online @supolka/agent-notebook@0.7.0 --version` prints `anb 0.7.0`.
