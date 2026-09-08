---
id: note.report-release-v2026-09-08-with-the-packages-at
type: note
state: retired
title: Report: Release v2026.09.08 with the packages at 0.4.0
by: Maksim Yaromin
from: task.release-v2026-09-08-with-the-packages-at
created: 2026-09-08
updated: 2026-09-08
---

# Report: release v2026.09.08

The packages moved to 0.4.0 through #68; the tag `v2026.09.08` is the day's first release and follows the merge commit `97f5a04`.

## What shipped

The identity and standing-rules work of #67: `taken-by` on Tasks written by `start` and the `taken` refusal, `--mine` and `--by` on `ready` and `list`, the `taken-by` column, `search` over people, `by` and `taken-by` in JSON rows, the `by/via` log signature, `ANB_BY`, Status leading with the caller's work, and a standing rule opening Status.

## Evidence

- `scripts/release/check-versions.sh`: every manifest at 0.4.0.
- `scripts/release/changelog-notes.sh v2026.09.08` extracted the entry; `check-tag.sh` accepted the tag.
- PR #68 CI green (`check`, `docs`), squash-merged, branch deleted.
- Release workflow green: five platform archives and `SHA256SUMS` on the GitHub release, the six packages published; `npx -y @supolka/agent-notebook@0.4.0 --version` from an empty directory prints `anb 0.4.0`.
