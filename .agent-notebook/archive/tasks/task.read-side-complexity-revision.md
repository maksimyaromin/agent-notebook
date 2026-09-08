---
id: task.read-side-complexity-revision
type: task
state: closed
title: Status is where the work continues; reads narrow by kind, tag and identity; the read side gets a complexity revision
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
link: issue https://github.com/maksimyaromin/agent-notebook/issues/71
link: note note.report-read-side-complexity-revision
priority: 1
created: 2026-09-08
updated: 2026-09-08
closed: 2026-09-08
---

Issue 71: a team on one notebook needs Status to be the work (Tasks by state, open Questions, own first under the team scope, only the caller's under --mine or config scope: mine), one listing surface narrowed the same way everywhere (--type, --kind, --tag, --for, --by, --mine, --team, --match, --archive), and a complexity revision of the read side before another flag is added: an inventory, a decision per duplicate, one narrowing rule.
- 2026-09-08 Maksim Yaromin/claude-code: inventory: 8 read verbs, 22 flags, 5 row printers. Duplicates and rulings: graph --ready deleted (a ready Task has no live blocker, so the lens is a listing drawn as tiles; ready answers it and a node carries ready); graph --type kept and list gains it (one Filter for both); overview deleted (list --all is the page, list --type <t> a section); search deleted, list --match <text> with --archive for history; --mine kept as --by with the identity filled in, --team widens one call, config scope: mine sets the default. Status keeps Tasks by state and open Questions, drops rules and epics, keeps Debt as one count line; anb debt is the Debt read.
- 2026-09-08 Maksim Yaromin/claude-code: implemented: Core Filter in request.rs, Admission in query.rs, one guard_filter; Status sections active/review/held/ready/questions + debt count with by/mine/team and config scope; Notebook::debt; graph on the Filter (JSON contract v3); shell: Whose and Narrowing under one help heading, debt verb, search and overview removed; commands reference renders the narrowing once; docs, atlas skill, AGENTS.md updated; check.sh and docs:check green. Next: Sonnet smoke check, then submit.
- 2026-09-08 Maksim Yaromin/claude-code: smoke check by Sonnet 5: no behaviour defect, four text findings fixed (a false fixture comment, a leftover search in the skill, a history-narrating comment, the scope refusal text); 13 new tests proven red by inverted expectations; check.sh and docs:check green; report at .tmp/docs/report-read-side-complexity-revision.md
- 2026-09-08 Maksim Yaromin/claude-code: owner's ruling: ready takes only what a queue can answer; --type, --kind and --archive moved to an Extent struct flattened into list and graph alone; clap refuses them on ready (test added); decision, skill, docs and report updated; gate green
