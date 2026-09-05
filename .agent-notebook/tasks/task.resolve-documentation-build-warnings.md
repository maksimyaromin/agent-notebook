---
id: task.resolve-documentation-build-warnings
type: task
state: open
title: Resolve documentation build warnings
by: Maksim Yaromin
from: task.write-the-readme-and-book-for-engineers
created: 2026-09-05
updated: 2026-09-05
---

The documentation build completes and link checks pass, but reports an oversized JavaScript chunk and Starlight warnings for the absent i18n collection and 404 entry. Trace the build configuration and bundled integrations; remove the causes without suppressing useful warnings. Verify with pnpm docs:check.
