---
id: task.import-surface-adopt-an-existing-tracker
type: task
state: open
title: Import surface: adopt an existing tracker's history
by: Maksim Yaromin
via: claude-code
from: decision.envelope-dates-mean-notebook-time
priority: 4
hold: waits for question.import-dates-vouched-or-stamped: the shape of the import surface is a product ruling
created: 2026-08-29
updated: 2026-09-06
---

A team switching to anb from another tracker needs to bring finished work in without replaying it through the live verbs and without lying about envelope dates. A dedicated import surface is where backdating is explicit and fenced: it writes records with their true historical dates, marked as imported, while ordinary verbs keep vouching for notebook time. Adoption-horizon work — scope it when the first external adopter appears.
- 2026-09-06 claude-code: the first adopter appeared on 2026-09-06 and filed https://github.com/maksimyaromin/agent-notebook/issues/46 with what its migration lost: every envelope date reads the migration day, the log lines carry two dates, the Debt clocks start today; two shapes are offered, an import verb that stamps imported records, or the documented hand-fix procedure, and the criterion is whether check must tell an imported date from a vouched one
