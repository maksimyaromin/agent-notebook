---
id: note.report-release-0-3-0
type: note
state: retired
title: Report: Release 0.3.0: the first adopter's findings
by: Maksim Yaromin
from: task.release-0-3-0
created: 2026-09-06
updated: 2026-09-06
---

# Release v2026.09.06.2 with the packages at 0.3.0

The third release of 2026-09-06 and the first shaped by an adopter: seven issues were filed after the tool was set up on an existing project and its history replayed through the live verbs, and six of them close in this release, one pull request each. The packages move to 0.3.0 because `anb setup` now refuses to wire an agent nobody named, so a script that ran it bare must name one.

## What shipped

- #49: an appended setup snippet is a paragraph of its own, and removal leaves a guide ending on one newline as it was (#43).
- #50: each Debt row of `status --json` carries its signal's fields beside the printed line (#48).
- #51: `close --note` mints `note.report-<task slug>`, every id through one minting seam (#44).
- #52: `add` and `edit` take `--body-file <path>`, `-` for standard input, and `--note -` reads standard input too (#47).
- #53: `edit --link` and `edit --unlink`; a link naming a record must name one that exists; a Decision declares a relationship once and `may-conflict` names only the pair nobody judged (#45).
- #54: `anb setup --agent <name>` wires only the agents named; a shared file goes only when every reader is named; a host directory setup emptied goes with its files (#42), with the ruling in decision.setup-wires-only-the-agents-named.
- #55: the versions, the changelog entry and the notebook records of the release.

Held: task.import-surface-adopt-an-existing-tracker waits on question.import-dates-vouched-or-stamped, the shape of the import surface (#46), taken off the milestone. Filed from the work: #56 (prose beginning with a hyphen is read as a flag), #57 (a dangling link is an error for check but closes the record to nothing), #58 (hold on a held Task keeps the old reason).

## Evidence

- Every pull request squash-merged on a green CI check; the tag `v2026.09.06.2` points at the merge of #55.
- The Release run for the tag: five builds, the GitHub release job and the npm publish job, every one a success.
- The GitHub release `anb v2026.09.06.2` is the latest and carries one archive per platform and `SHA256SUMS`.
- On npm, `@supolka/agent-notebook` and its five platform packages are at 0.3.0 with `latest` pointing at it; the darwin-arm64 package showed 0.2.0 for about a minute after the publish, as a platform package did in the previous release, and 0.3.0 once the registry caught up.
- From an empty directory with a fresh npm cache, `npx -y @supolka/agent-notebook@0.3.0 --version` answers `anb 0.3.0`; `setup` without an agent refuses with the three try lines, and `setup --agent claude-code` writes its nine files.
- `check-versions.sh` ok at 0.3.0, `check-tag.sh v2026.09.06.2` ok, `changelog-notes.sh v2026.09.06.2` extracts the entry; `scripts/check.sh` and `pnpm docs:check` green on #55.
