---
id: task.status-json-debt-fields
type: task
state: closed
title: status --json carries Debt as fields
by: Maksim Yaromin
via: claude-code
from: task.release-0-3-0
link: issue https://github.com/maksimyaromin/agent-notebook/issues/48
link: note note.report-status-json-carries-debt-as
priority: 1
created: 2026-09-06
updated: 2026-09-06
closed: 2026-09-06
---

Each Debt row in the JSON Status carries the variant's own fields beside code and line: the ids of a pair, the id and the days of a clock, the target of a dangling mention. Rendering only: the model and the text output stay as they are. Evidence: the JSON tests assert the fields; the Status reference names the shape.
- 2026-09-06 claude-code: each Debt row in status --json carries the variant's fields beside code and line; cited records reuse the add reply's {id,by,via} shape; a CLI test asserts the fields for a clock, a dangling mention, a pair and an invalid file and was proven red once; the Status reference gains a JSON fields column; gate and docs check next, then the smoke check
- 2026-09-06 claude-code: smoke check: no defect; one should-fix taken (the row renderer's doc comment kept only its contract clause); gate green; submitting
