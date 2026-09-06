---
id: task.status-json-debt-fields
type: task
state: open
title: status --json carries Debt as fields
by: Maksim Yaromin
via: claude-code
from: task.release-0-3-0
link: issue https://github.com/maksimyaromin/agent-notebook/issues/48
priority: 1
created: 2026-09-06
updated: 2026-09-06
---

Each Debt row in the JSON Status carries the variant's own fields beside code and line: the ids of a pair, the id and the days of a clock, the target of a dangling mention. Rendering only: the model and the text output stay as they are. Evidence: the JSON tests assert the fields; the Status reference names the shape.
