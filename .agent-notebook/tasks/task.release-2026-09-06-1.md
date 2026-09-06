---
id: task.release-2026-09-06-1
type: task
state: active
title: Release v2026.09.06.1 with the packages at 0.2.0
by: Maksim Yaromin
priority: 0
created: 2026-09-06
updated: 2026-09-06
---

The second release of the day: the author method and the new Note kinds ship, so the packages move to 0.2.0. The release tag takes the same-day counter, the first tag of that shape.
- 2026-09-06 Maksim Yaromin: Versions at 0.2.0 in Cargo.toml, Cargo.lock and the six package manifests. scripts/release/check-tag.sh holds the tag shape, vYYYY.MM.DD or vYYYY.MM.DD.N with N from 1, and both publishing jobs of the Release workflow run it first; check-versions.sh no longer equates a tag with a version, a leftover of version tags. The release guide states the same-day rule with v2026.09.06 and v2026.09.06.1 as its example. CHANGELOG.md carries the v2026.09.06.1 entry, which changelog-notes.sh extracts.
