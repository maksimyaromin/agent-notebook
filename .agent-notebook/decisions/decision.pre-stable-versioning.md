---
id: decision.pre-stable-versioning
type: decision
state: active
kind: rule
title: Pre-stable releases may change interfaces in a minor version
by: Maksim Yaromin
via: codex
from: task.release-0-9-0
supersedes: decision.a-release-moves-the-version-by-semver
created: 2026-09-12
updated: 2026-09-12
---

Package versions stay synchronized across Cargo and npm. A compatible fix moves the patch; new behavior moves the minor. Before 1.0, the maintainer may choose a minor release for incompatible changes, with explicit upgrade instructions covering removed or changed commands and reply shapes. After the stable interface is released, incompatible changes require a major version. The major number never moves without approval from the maintainer. Release 0.9.0 is the explicitly requested pre-stable minor release.
