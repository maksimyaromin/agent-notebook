---
id: note.report-release-v2026-09-06-1-with-the
type: note
state: retired
title: Report: Release v2026.09.06.1 with the packages at 0.2.0
by: Maksim Yaromin
from: task.release-2026-09-06-1
created: 2026-09-06
updated: 2026-09-06
---

# Release v2026.09.06.1 with the packages at 0.2.0

The second release of 2026-09-06, and the first tag named by the same-day counter. The first release of a day is tagged `vYYYY.MM.DD`; each further release that day appends `.N`, counting from 1, so `v2026.09.06` was the day's first release and `v2026.09.06.1` its second. The rule is a Decision in the notebook, `scripts/release/check-tag.sh` holds the shape, and both publishing jobs of the Release workflow run it right after checkout, so a tag of another shape fails before a release is created or a package published. `check-versions.sh` lost the branch that compared a tag to a version, unused since tags stopped being versions. The release guide states the rule with the two tags as its example, and the changelog entry for the tag is what the workflow took as the release notes.

The release ships the author method and the new Note kinds from #39, the project's mark from #34 and #38, and the documentation build that warns about nothing from #36. The packages move from 0.1.1 to 0.2.0 because an older binary refuses the `idea`, `model` and `spec` kinds.

## Evidence

- Pull requests #39 (the prepared documentation, skill and Note kind changes) and #40 (the rule, the checks, the versions and the changelog entry) merged by squash with green CI; the tag `v2026.09.06.1` points at the merge of #40.
- The Release run for the tag: five builds, the GitHub release job and the npm publish job, every one a success.
- The GitHub release `anb v2026.09.06.1` is marked latest and carries one archive per platform and `SHA256SUMS`.
- On npm, `@supolka/agent-notebook` and its five platform packages are at 0.2.0 with `latest` pointing at it, each with a provenance attestation, the first release published from the public repository.
- From an empty directory, `npm install @supolka/agent-notebook@0.2.0` resolves the platform package beside it and `anb --version` answers `anb 0.2.0`; `npx -y @supolka/agent-notebook@0.2.0 --version` answers the same. A run of `npx` within a minute of the publish failed to see the platform package at 0.2.0 and left that miss in the local `npx` cache; the registry had every package a minute later, and the check passed once the cache was cleared.
- The three release checks on the merged commit: `check-tag.sh v2026.09.06.1` ok, `check-versions.sh` ok at 0.2.0, `changelog-notes.sh v2026.09.06.1` extracts the entry.
- `scripts/check.sh` and `pnpm docs:check` green on both pull requests, and the smoke check on #40 found no defect; its one ordering note, the tag check placed late in the publish job, was taken and the check moved to the top of the job.
