---
id: note.report-link-to-a-record-id-is-an-edge
type: note
state: retired
title: Report: A link to a record id is an edge the tool walks
by: Maksim Yaromin
from: task.link-to-a-record-id-is-an-edge
created: 2026-09-08
updated: 2026-09-08
---

# Report: a link to a record id is an edge the tool walks

Closes [issue 64](https://github.com/maksimyaromin/agent-notebook/issues/64). The full audit of this change beside its sibling, the addressee of issue 79, is the report of task.a-question-and-a-review-name-whom-they-wait-on.

## Result

A `link` line whose target is a record id relates the two records under the link's kind, from the record carrying the line into the record it names. `Record::linked_records` is the one home of the edge, and every walk reads it:

- `show <id>` lists the live records linking it as `linked-by[N]: <id> (<kind>), …`, by kind then id, and JSON as `linked-by: {count, rows: [{id, kind}]}`.
- `graph` draws the edge under the link's own word, after `waits` and `born` and before `mentions`; the graph document is at `v` 4.
- `list --for <id>` and `graph --for <id>` reach the records that link a record, as far as the links go, and `--focus` walks the same edge.

A link kind may be any token but the three words the graph draws itself, and a record cannot link itself; `edit` refuses both, and a hand-written self-link draws no edge. `--match` over link lines was priced and dropped: git already ties the notebook's records to the code they shipped with.

## Evidence

- `./scripts/check.sh` and `pnpm docs:check` green.
- Six tests, each proven red by cutting the edge at its source: the `linked-by` block, the edge and its precedence over a mention, the scope and the focus through a link, the refused kinds and the self-link, and the two shell renderings of `show` and `graph`.
- The book, both skills and the commands reference follow; the repository's own notebook links the new rule Decision `within` the identity ruling and reads it back.
