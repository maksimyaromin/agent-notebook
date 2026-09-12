---
id: decision.release-date-version
type: decision
state: active
kind: rule
title: Release tags contain the date and package version
by: Maksim Yaromin
via: codex
from: task.release-0-9-0
supersedes: decision.same-day-release-tags
created: 2026-09-12
updated: 2026-09-12
---

New release tags use vYYYY.MM.DD.MAJOR.MINOR.PATCH. The final three components must equal the Cargo workspace and npm package version. The GitHub release title is anb followed by the tag, and asset names retain that tag. Another release on the same day carries its actual package version rather than a sequence counter. Existing published tags, release titles and historical changelog entries remain unchanged. The release check refuses malformed tags and version mismatches before publication.
