---
id: task.readme-demo-0-9-0
type: task
state: closed
title: Refresh the README terminal demo for 0.9.0
by: Maksim Yaromin
via: codex
taken-by: Maksim Yaromin
from: task.release-0-9-0
created: 2026-09-12
updated: 2026-09-12
closed: 2026-09-12
---

Restore the README animation with real output from the published CLI. Preserve the existing terminal appearance and private recording configuration, update obsolete commands, and inspect the rendered frames for readable output. Keep the recording visible below the README badges.
- 2026-09-12 Maksim Yaromin/codex: Also refine the README header spacing: inspect the logo bounds and align the logo, tagline, navigation and badges into a compact vertical rhythm.
- 2026-09-12 Maksim Yaromin/codex: Restored the README animation with real output from the published 0.9.0 CLI. The 36.52-second recording shows task creation, progress, a shared rule, session continuation, completion and archive; the rule remains readable and the final check has no findings. Inspected all three scenes for readable, uncropped output. Trimmed the light and dark logo viewports, stabilized wordmark width across fallback fonts, and grouped the header into a compact vertical rhythm. Verified the README with GitHub Markdown styles in both themes and at a 390-pixel viewport. The docs build and all 20 content pages pass their link checks; git diff --check passes.
