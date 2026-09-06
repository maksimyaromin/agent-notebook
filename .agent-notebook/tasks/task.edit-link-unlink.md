---
id: task.edit-link-unlink
type: task
state: review
title: edit declares and erases a link
by: Maksim Yaromin
via: claude-code
from: task.release-0-3-0
link: issue https://github.com/maksimyaromin/agent-notebook/issues/45
priority: 1
created: 2026-09-06
updated: 2026-09-06
---

edit --link and edit --unlink so a live record can declare a relationship after the fact; a Decision that cites another as context declares it with a link and may-conflict is for the pair that has none. The Debt heuristic already honours a link naming the id. The skill and the knowledge guide teach the sentence. Evidence: core and CLI tests; the regenerated references; a run of anb status on this notebook shows no pair that was judged.
- 2026-09-06 claude-code: edit --link and --unlink, repeatable, spliced at set granularity like tags; a link whose target is id-shaped must name a record, here or in the user's notebook, on add and edit alike, as --from does and as check already demanded; the skill and the knowledge guide teach within and departs-from; two core tests and four CLI tests shown red once; used on this notebook to link the held import Task to its issue; gate and docs check green; smoke check next
- 2026-09-06 claude-code: smoke check: no regression; three should-fix taken (links matched as the grammar reads them, the cited side tested, the skill sentence split), four nits taken, one asymmetry noted for a later issue; gate green; submitting
