---
id: note.report-release-v2026-09-08-4-with-the-packages
type: note
state: retired
title: Report: Release v2026.09.08.4 with the packages at 0.8.0
by: Maksim Yaromin
from: task.release-v2026-09-08-4-with-the-packages
created: 2026-09-08
updated: 2026-09-08
---

# Report: release v2026.09.08.4

The packages moved to 0.8.0 through #84; the tag `v2026.09.08.4` is the day's fifth release and follows the merge commit `e75d069`. A minor by the versioning rule the releasing guide now states: a line joined three replies and a field joined two JSON documents.

## What shipped

The whose line of #83: a `list`, `ready` or `graph` narrowed to one identity opens with `by: <name> — anb <verb> … --team`, its JSON carries `by`, and the skill carries the table of whose question takes which flag. The releasing guide states the semantic-versioning rule.

## Evidence

- `scripts/release/check-versions.sh`: every manifest at 0.8.0.
- `scripts/release/changelog-notes.sh v2026.09.08.4` extracted the entry; `check-tag.sh` accepted the tag.
- PR #84 CI green (`check`, `docs`), squash-merged, branch deleted.
- Release workflow run 34274831386 green: five platform builds, the GitHub release with five archives and `SHA256SUMS`, the six packages published.
- From an empty directory `npx -y --prefer-online @supolka/agent-notebook@0.8.0 --version` prints `anb 0.8.0`.
