---
id: task.setup-snippet-paragraph
type: task
state: closed
title: setup appends its snippet as its own paragraph
by: Maksim Yaromin
via: claude-code
from: task.release-0-3-0
link: issue https://github.com/maksimyaromin/agent-notebook/issues/43
link: note note.report-setup-appends-its-snippet-as-its
priority: 1
created: 2026-09-06
updated: 2026-09-06
closed: 2026-09-06
---

A guide ending with a list item or a paragraph gets the snippet as a separate paragraph, so CommonMark does not read it as a lazy continuation of the user's last line. Preserve: a re-run reports already present; removal takes out the snippet and the blank line it added. Evidence: a test that fails on the joined line before the fix.
- 2026-09-06 claude-code: an appended snippet now stands behind a blank line and removal takes that blank line back out when a blank line or the end of the file follows; three unit tests went red before the fix; scripts/check.sh green; the agents guide names the paragraph; smoke check running
- 2026-09-06 claude-code: smoke check: one must-fix (the round trip was claimed byte-exact for every guide, holds only for a guide ending on one newline; the contract is now stated and tested as it is), one should-fix taken (one situation per test), one declined (a bare LF in a CRLF guide, as every line the tool writes), one nit taken; gate green; submitting
