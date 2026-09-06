---
title: Decisions, Notes and Questions
description: 'Record what the project knows, distinguish current rules from old ones, and resolve open questions.'
---

Choose the record by how the information changes. A ruling needs an explicit replacement, a fact needs correction, and a question needs an answer. Keeping them separate lets the notebook show what still applies. Keep a local observation in the Task log when it only helps finish that Task. Give it a Note when another piece of work should be able to find and use it. A Decision needs the constraint behind the choice, so a later reader can judge whether it still holds.

## Decisions

Record a Decision when a choice should guide later work. Include the reason: the next reader needs to know the constraint behind the choice, especially when that constraint changes.

```text
$ anb add decision "Fences never nest" --kind rule --tag parser --tag grammar --body "A fence closes at the first closing marker."
ok: add decision.fences-never-nest — decisions/decision.fences-never-nest.md
```

| Kind | Use it for | Include |
|---|---|---|
| `rule` | An agreed behavioral or working rule | Scope, requirement and reason |
| `shape` | A design choice between alternatives | Chosen approach, alternatives and the constraint that decided it |
| `drift` | An accepted exception | The rule it departs from, its consequences and the condition for revisiting it |

A proposed choice belongs in an idea or Question until it is agreed. A `drift` can use `review-by` when a review date is known; neither the kind nor the date automatically retires it. Status includes active `rule` Decisions by id and title, subject to its display limits. Open the record to read the reason.

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

Notes hold maintained knowledge, including ideas that have not become work yet. Choose the kind by what the document is for:

| Kind | Contents |
|---|---|
| `fact` | An observation or research result, with its evidence and limits |
| `term` | A definition, its domain context and words easily confused with it |
| `guide` | A repeatable procedure, including when to use it and how to check the outcome |
| `idea` | A problem to explore, intended outcome, constraints, alternatives and open questions |
| `model` | Domain boundaries, concepts, relationships, ownership and invariants |
| `spec` | Expected behavior, scope, exclusions and acceptance criteria |

Correct a Note with `anb edit <id> --body "<text>"` as understanding changes, or with `--body-file <path>` when the body is a document rather than a sentence; `-` reads standard input. Retire it when it is no longer useful. An active idea or spec is not necessarily accepted: its agreement status belongs in its body. The CLI validates the kind and lifecycle, not the truth of its contents or the user's agreement.

[Developing an idea](ideas.md) explains the route from a ticket or conversation to delivery. [Domain modeling](domain.md) explains when a glossary needs a model as well.

A report imported with `anb close --note` is also a Note, linked to the Task it documents. This keeps the evidence accessible through the same commands as the work.

## Questions

Record a Question when work exposes an uncertainty that deserves its own resolution:

```text
$ anb add question "Do fences nest?" --from task.parser-accepts-fenced-bodies
ok: add question.do-fences-nest — questions/question.do-fences-nest.md
```

Use `anb close <question> --resolved-by <decision-or-task>` when a record answers it. Use `anb close <question> --reason "<why>"` when it closes without such a record. A Question cannot close without one of these outcomes.

Open Questions become Debt after their age threshold. If the origin Task closes first, the Question surfaces immediately. The reminder asks you to resolve the uncertainty; it does not assume that finishing the Task answered it.

## Record who is acting

The supplied skill asks agents to pass `--via` on `add` and `comment`, using a consistent tool name such as `codex` or `claude-code`. On creation, `by` names the accountable person, normally from git identity, while `via` names the tool. On a comment, `--via` labels that log entry and leaves the record's creation fields unchanged. Other commands do not accept `--via`.

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
