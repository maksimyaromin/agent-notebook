---
id: task.readable-id-by-default
type: task
state: closed
title: A record gets a readable id by default
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
from: task.records-live-with-their-task
link: issue https://github.com/maksimyaromin/agent-notebook/issues/92
created: 2026-09-30
updated: 2026-09-30
closed: 2026-09-30
---

Issue #92: add derives the slug from the title, suffixes a collision and says so, refuses a title with no usable slug; --id wins and import keeps source ids. An id-only edge list under decision.one-graph-read-through-show-start-recall is legible only with readable ids.
- 2026-09-30 Maksim Yaromin/claude-code: Implemented. The CLI no longer allocates random ids: the Core mints `<type>.<slug>` from the title (existing slugify, 40-character cap at a word boundary), a taken id gets the two-character suffix and the add reply names the holder as `collision`, a title with no ASCII letter or digit is refused with invalid-argument asking for --id. --id wins; import keeps source ids. getrandom left the workspace. Docs: records (ids), replies (creation), quickstart, development; the worked session drops its explicit ids. The rename open point of #92 is not addressed.
- 2026-09-30 Maksim Yaromin/claude-code: Follow-up from the field test: slugify drops an apostrophe (ASCII or typographic) instead of turning it into a hyphen, so `A Decision's kind` mints `a-decisions-kind`. Covered in creation tests; records reference updated.
- 2026-09-30 Maksim Yaromin/claude-code: After the skill test (agents guessed ids from long titles and hit the cut): the slug cap rose 40 -> 64 characters and the id grammar cap 64 -> 96 bytes, with a compile-time assert that the longest type word, the slug and a suffix fit. The limit and the rule 'choose --id when a later command in the same script names the record' now stand where an agent reads before its first write: add --help, the skill's intent section and the records reference (which also notes that older binaries reject ids over 64 bytes). Cap tests restated for 64; the too-long corpus id is now 105 bytes.
- 2026-09-30 Maksim Yaromin/claude-code: add derives <type>.<slug> from the title, cut at a word boundary within 64 characters (ids up to 96 bytes), drops apostrophes, suffixes a taken id and names it as collision, and refuses a title with no ASCII word; add --help and the skill say when to choose --id. PR #96.
