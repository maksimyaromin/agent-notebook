---
id: task.cli-help-and-reply-texts-pass-the
type: task
state: closed
title: CLI help and reply texts pass the humanizer, so the commands reference reads well
by: Maksim Yaromin
from: task.skills
tags: cli
link: note note.report-cli-help-and-reply-texts-pass-the
created: 2026-09-05
updated: 2026-09-05
closed: 2026-09-05
---

Owner ruling (2026-09-05): skills must read pleasantly for a human, so every skill text passes the humanizer's rules (no em or en dashes, no bold-header bullets, no forced triples, plain sentences). The commands reference is rendered from the clap help strings, which lean on em dashes as parentheticals, and the worked session shows the replies as printed. Sweep the help strings and, where a reply's prose carries the same tells, the reply texts, without changing any word the vocabulary decision settled; regenerate the skill and its committed copy.
- 2026-09-05 Maksim Yaromin: Inventory: the rendered commands reference carried four dashes, all from five help strings (--global, add, comment, search, --report); one refusal message carried a sixth. Rewritten as plain sentences, skill regenerated, gate green. The dash in the reply format (ok: <verb> <id> — <what changed>) and in the dangling-mention nudge is structure, not prose, and stays.
- 2026-09-05 Maksim Yaromin: Smoke check found eight more Core refusal messages with the prose dash and the 0–4 range; all rewritten, gate green. Report at .tmp/docs/report-help-strings.md.
