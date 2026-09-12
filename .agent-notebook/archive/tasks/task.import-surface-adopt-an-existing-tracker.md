---
id: task.import-surface-adopt-an-existing-tracker
type: task
state: closed
title: "Import surface: adopt an existing tracker's history"
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
from: decision.envelope-dates-mean-notebook-time
link: issue https://github.com/maksimyaromin/agent-notebook/issues/46
priority: 4
created: 2026-08-29
updated: 2026-09-12
closed: 2026-09-12
---

A team switching to anb from another tracker needs to bring finished work in without replaying it through the live verbs and without lying about envelope dates. A dedicated import surface is where backdating is explicit and fenced: it writes records with their true historical dates, marked as imported, while ordinary verbs keep vouching for notebook time. Adoption-horizon work — scope it when the first external adopter appears.
- 2026-09-06 claude-code: the first adopter appeared on 2026-09-06 and filed https://github.com/maksimyaromin/agent-notebook/issues/46 with what its migration lost: every envelope date reads the migration day, the log lines carry two dates, the Debt clocks start today; two shapes are offered, an import verb that stamps imported records, or the documented hand-fix procedure, and the criterion is whether check must tell an imported date from a vouched one
- 2026-09-12 Maksim Yaromin/codex: Import validates the prospective notebook before writing, preserves source IDs, dates, attribution, record bodies and archive locations, and safely retries interrupted additive work. It refuses conflicting IDs and dangling references. decision.source-history settles the provenance rule: source history stays source history; an imported flag or replacement author would add no reliable information. Core and real-process transfer tests verify preview, import, migration and crash recovery. The full code and documentation gates pass.
