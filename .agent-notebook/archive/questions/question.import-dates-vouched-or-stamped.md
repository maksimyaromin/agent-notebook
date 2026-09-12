---
id: question.import-dates-vouched-or-stamped
type: question
state: closed
title: Does check need to tell an imported date from a vouched one?
by: Maksim Yaromin
via: claude-code
from: task.import-surface-adopt-an-existing-tracker
resolved-by: decision.source-history
created: 2026-09-06
updated: 2026-09-12
closed: 2026-09-12
---

The criterion issue 46 names between its two shapes. If yes, the import surface is a verb: it takes a directory of record files or a manifest, runs every record through the invariants add and the lifecycle verbs enforce, trusts created, updated, closed and the dates in log lines, and stamps the envelope so check and Status can tell an imported record from a vouched one; ordinary verbs keep refusing to backdate. If no, the sanctioned migration is the procedure this project used once: replay through the CLI, hand-fix the envelope dates, run check, record the exception as a drift Decision. Facts in hand: the first adopter replayed 53 records on 2026-09-06 and every envelope date reads that day, every log line carries two dates, and the Debt clocks started that day; decision.envelope-dates-mean-notebook-time already names a fenced import surface as the future route and the hand-fix as a one-time exception. Recommendation: yes, the verb. The hand-fix asks the adopter to break the rule the tool exists to hold, and the exception was approved once for the tool's own migration, not as a procedure. What the ruling must settle before the verb is scoped: the input shape (record files as they would be written, or a manifest), whether a dated log line of the form the adopter produced is accepted as input, and what check reports for a stamped record. The Task stays held until this is answered.
