---
id: task.erase-an-optional-envelope-field-through
type: task
state: open
title: Erase an optional envelope field through the CLI
by: Maksim Yaromin
via: claude-code
from: decision.check-names-an-origin-cycle
tags: cli
priority: 3
created: 2026-08-30
updated: 2026-08-30
---

Every optional envelope field can be written and rewritten, and none can be removed. `edit --from` takes a record id and refuses an empty one, so a false Origin can only be repointed at some other record — which invents a birth — never erased; the same holds for a priority, a review-by, and a kind once set. The one edge with an erasing verb is the dependency edge, and unblock exists precisely because a false blocker must be removable rather than redirected. This surfaced as a Check finding with no repair: a lineage loop is an error, and nothing in the CLI can clear the line that closes it. Acceptance: a removal shape chosen and applied to every optional field the same way, refused where the field is not optional, idempotent on a field already absent, and a Check finding that names a bad field points at a command that can clear it.
