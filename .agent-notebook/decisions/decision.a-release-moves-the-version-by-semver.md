---
id: decision.a-release-moves-the-version-by-semver
type: decision
state: active
kind: rule
title: A release moves the version by semver, and only the maintainer moves the major
by: Maksim Yaromin
via: claude-code
from: task.a-narrowed-listing-says-whose-it-is
tags: release
created: 2026-09-08
updated: 2026-09-08
---

The package version moves by semantic versioning. Patch: a fix that changes no documented reply shape, flag or field. Minor: anything added, a flag, a field, a line in a reply, a finding, a changed default. Major: a removal, a rename or a changed meaning of a command, a flag, a field, a state word or a reply shape; the major never moves without the maintainer's consent. The rule decides the number before the release Task is written, so a release is never a minor by habit: a narrowed listing that gains a by line and a JSON field is a minor because a documented shape grew, and it would be a patch only if no documented shape had changed. Recorded because no committed text or ruling said which number a release moves.
