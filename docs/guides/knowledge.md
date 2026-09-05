---
title: Decisions, Notes and Questions
description: 'Record what the project knows, distinguish current rules from old ones, and resolve open questions.'
---

Choose the record by how the information changes. A ruling needs an explicit replacement, a fact needs correction, and a question needs an answer. Keeping them separate lets the notebook show what still applies.

## Decisions

Record a Decision when a choice should guide later work. Include the reason: the next reader needs to know the constraint behind the choice, especially when that constraint changes.

```text
$ anb add decision "Fences never nest" --kind rule --tag parser --tag grammar --body "A fence closes at the first closing marker."
ok: add decision.fences-never-nest — decisions/decision.fences-never-nest.md
```

Use `rule` for working rules, `shape` for design decisions, and `drift` for an accepted deviation. Status includes active `rule` Decisions by id and title, subject to its display limits. Open the record to read the reason.

When the ruling changes, use `anb add decision "<title>" --supersedes <old>`. The command links the records and marks the predecessor `superseded`. Use `anb retire <id>` when a Decision no longer applies and has no replacement. Editing a Decision is for corrections to the same ruling, not for replacing it with another.

### Possible conflicts

A new Decision may overlap an existing one:

```text
$ anb add decision "A fence body is opaque" --kind rule --tag parser --tag grammar
ok: add decision.a-fence-body-is-opaque — decisions/decision.a-fence-body-is-opaque.md
may-conflict[1]: decision.fences-never-nest (Alex)
```

The tool reports a possible conflict when Decisions share at least two tags or the new one cites an active Decision. It does not compare their meaning. Read the named record and decide whether the new ruling replaces it.

Status keeps a `may-conflict` Debt signal for a citation between active Decisions without a supersession relationship. Shared tags alone trigger the write-time hint, not persistent Debt.

## Notes

Use a Note for knowledge you want to maintain: `fact` for an observation, `term` for a domain definition, `guide` for a practice. Correct it with `anb edit <id> --body "<text>"` as your understanding changes. Retire it when it is no longer useful.

A report imported with `anb close --note` is also a Note, linked to the Task it documents. This keeps the evidence accessible through the same commands as the work.

## Questions

Record a Question when work exposes an uncertainty that deserves its own resolution:

```text
$ anb add question "Do fences nest?" --from task.parser-accepts-fenced-bodies
ok: add question.do-fences-nest — questions/question.do-fences-nest.md
```

Use `anb close <question> --resolved-by <decision-or-task>` when a record answers it. Use `anb close <question> --reason "<why>"` when it closes without such a record. A Question cannot close without one of these outcomes.

Open Questions become Debt after their age threshold. If the origin Task closes first, the Question surfaces immediately. The reminder asks you to resolve the uncertainty; it does not assume that finishing the Task answered it.

## Citations

Write a bare record id in a body or comment to reference it. Put the id in backticks when you are discussing its spelling or using it as an example. `anb show` derives incoming and outgoing mentions from the prose.

A reference to an unknown id produces a hint without rejecting the write:

```text
$ anb comment task.grammar-parser-accepts-fences "see task.typo-in-the-id"
ok: comment task.grammar-parser-accepts-fences — logged
dangling-mention[1]: task.typo-in-the-id — backtick to quote, or create the record
```

This allows a forward reference. If it was a typo, correct it; if it was an example, quote it. Until it resolves, the reference also appears as Debt. Envelope relationships such as `--from` are stricter and require an existing target.

## Archive settled knowledge

Archive a superseded or retired Decision, a retired Note, or a closed Question with `anb archive <id>`. `show` and `search` still find archived records. `anb restore <id>` returns one to the working set without changing its state or contents.

Use `anb delete <id>` only for a record created by mistake. It removes the file and frees the id, but refuses while another record references it.
