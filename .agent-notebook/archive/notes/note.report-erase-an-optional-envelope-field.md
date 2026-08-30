---
id: note.report-erase-an-optional-envelope-field
type: note
state: retired
title: Report: Erase an optional envelope field through the CLI
by: Maksim Yaromin
from: task.erase-an-optional-envelope-field-through
created: 2026-08-30
updated: 2026-08-30
---

# A record nothing could repair

## The gap

`check` reported error classes that no command could clear.

**A dangling origin.** A record carrying `from: task.ghost` earns a `dangling-ref` error finding. The mutation gate refuses to write any record carrying an error finding, so `edit --from <other>` was refused too, and so was every other verb. The record was frozen, `check` named it on every run, and the only repair was a hand edit of the file the CLI exists to own.

**A lineage loop, and a value set by mistake.** `from` could be repointed at some other record — which invents a birth the record never had — but not erased. The same held for a `priority` and a `review-by`. The one edge with an eraser was the dependency edge, and `unblock` exists precisely because a false blocker must be removable rather than redirected.

**A half-repair that reported success.** The gate already had one exception: `unblock` runs over an error finding sitting on the record's own `blocked-by` lines, so a corrupted edge cannot freeze the verb that erases it. That exception was stated in terms of lines, and lines are not what a splice acts on. A record with `blocked-by: task.ghost` and `blocked-by: task.b`, unblocked on `task.b`, passed the gate on the ghost's finding, erased the *other* edge, and wrote a record that was still invalid — `ok`, exit 0. This one predates this work.

## The shape

**`edit --clear <field>`** (repeatable; `from`, `priority`, `review-by`) erases an optional envelope field. The set is the optional fields `edit` can also write, minus those with an eraser of their own: a body is replaced whole by an empty `--body`, a tag leaves through `--untag`. Clearing a field a record does not carry changes no byte and answers `unchanged (already)`. A field both written and cleared in one call is refused. A field the record's type does not allow — a `priority` on a Decision — is erasable all the same, because erasing it is the repair; only writing one is refused.

**The gate's exception became a postcondition.** A verb that repairs — `edit` and `unblock` — reads the record over its error findings and is judged on the bytes it produces: if any error finding survives the splice, the call is refused and nothing is written. The line the splice aims at stopped mattering, which is what makes the rule true rather than nearly true:

- `set_field` rewrites the *first* line of a key, so a record with two `from:` lines and an `--from` edit repaired one and left the duplicate standing.
- `retagged` re-serializes the existing `tags:` value, so a malformed tag list survived a `--tag` edit.
- `unblock` erases by value, so it could erase a sound edge while a corrupted one stood.
- A finding with no line at all — a missing required field — could not be admitted under a line rule, yet `edit --title` on a record that has lost its `title:` line produces exactly the missing line. Under the postcondition it is allowed, and the record comes out clean.

A file with no envelope stays outside all of this: there is no line to correct, only a file to write again, and the splicing methods are not defined on one.

## What it deleted

`query::borne_by` and `write::edited_keys` — the line bookkeeping — are gone, along with the per-verb admission closure. `resolve_live_kept` takes an `Admission` of two words, and `Notebook::guard_repaired` states the rule once.

## The boundary, stated

The optional envelope fields with no CLI repair are now the machine-written ones: `kind`, `by`, `via`, `link`, `supersedes`, `superseded-by`, `routed-to`, `updated`, `closed`. Only a hand edit can corrupt them, and clearing one without a way to write it back would be a one-way door — `edit` cannot set a `kind`, so `--clear kind` would strip a Decision out of the rules block permanently. Whether every envelope line should be repairable through the CLI, or a hand-made corruption is a hand-repaired one, is a design question, and it is filed as one rather than answered by accretion.

The other declined item: the filed task asked that a `check` finding "point at a command that can clear it". No finding in this tool carries a repair hint, and making one class the exception would be a wart; adding hints to all of them is a reply-shape decision with a token cost on every row. Filed as a Question.

## Verification

- Twelve tests in `crates/anb-core/tests/notebook/edit.rs` and `dependencies.rs` covering both sides of the postcondition: the two repairs of a dangling origin, the three half-repairs that are refused (duplicate key, malformed tag list, an unrelated edit), the missing line that an edit supplies, the file with no envelope, and `unblock`'s sound-edge-beside-a-corrupted-one. Each was run against the code with the postcondition disabled and observed to fail.
- The suite is 458 → 478 tests, green through `./scripts/check.sh`.
- A 28-command mutation script over four seed notebooks, run against a build from before this change: replies and trees byte-identical.
