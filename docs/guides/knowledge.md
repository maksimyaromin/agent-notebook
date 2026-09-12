---
title: Decisions, Notes and Questions
description: 'Record what the project knows, distinguish current rules from old ones, and resolve open questions.'
---

Choose the record by how the information changes. A ruling needs an explicit replacement, a fact needs correction, and a question needs an answer. Keeping them separate lets the notebook show what still applies. Keep a local observation in the Task log when it only helps finish that Task. Give it a Note when another piece of work should be able to find and use it. A Decision needs the constraint behind the choice, so a later reader can judge whether it still holds.

## Decisions

Record a Decision when a choice should guide later work. Include the reason: the next reader needs to know the constraint behind the choice, especially when that constraint changes.

```sh
anb add decision "Fences never nest" --id decision.fences-never-nest --kind rule --tag parser --tag grammar --body "A fence closes at the first closing marker."
```

| Kind | Use it for | Include |
|---|---|---|
| `rule` | An agreed behavioral or working rule | Scope, requirement and reason |
| `shape` | A design choice between alternatives | Chosen approach, alternatives and the constraint that decided it |
| `drift` | An accepted exception | The rule it departs from, its consequences and the condition for revisiting it |

A proposed choice belongs in an idea or Question until it is agreed. A `drift` can use `review-by` when a review date is known; neither the kind nor the date automatically retires it. `anb list --type decision --kind rule` lists the standing rules by id and title; the supplied skill reads it before the work a rule binds. Open the record to read the reason.

When the ruling changes, use `anb add decision "<title>" --supersedes <old>`. The command links the records and marks the predecessor `superseded`. Use `anb retire <id>` when a Decision no longer applies and has no replacement. Editing a Decision is for corrections to the same ruling, not for replacing it with another.

### Related claims and contradictions

Shared tags and citations identify related material, not a contradiction. Read the claims, their scope and their evidence before deciding whether they conflict. A new exception can complement a standing rule; a replacement changes the ruling and should name its predecessor.

Use a typed link when the relationship deserves an explicit name:

```sh
anb edit decision.fence-body --link "within decision.fences-never-nest"
```

A link whose target looks like a record id must name an existing record in the same notebook. `show` exposes incoming links and `graph` draws their direction. `--unlink` removes the named relationship. Links provide context, not work membership; `list --for <id>` follows origin descendants instead. None of these operations decides whether the linked claims agree.

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

A finding from completed work becomes a Note when it needs independent maintenance. Create it with `anb add note --from <task>` and link its sources. The Task's outcome stays on the Task; archiving the Task does not retire its knowledge.

## Questions

Record a Question when work exposes an uncertainty that deserves its own resolution:

```sh
anb add question "Do fences nest?" --from task.parser-accepts-fenced-bodies
```

`--to <name>` puts the Question to the person who can settle it, and their Status shows it. Use `anb close <question> --resolved-by <decision-or-task>` when a record answers it. Use `anb close <question> --reason "<why>"` when it closes without such a record. A Question cannot close without one of these outcomes.

Open Questions become Debt after their age threshold. If the origin Task closes first, the Question surfaces immediately. The reminder asks you to resolve the uncertainty; it does not assume that finishing the Task answered it.

## Record who is acting

Use a consistent `--via` value such as `codex` or `claude-code`. On creation, `by` names the accountable person, normally from git identity, while `via` names the tool. `comment`, `close --body` and `retire --body` sign their appended entry without changing the record's creation fields.

## Comments and outcomes

`comment` appends context to any live Task, Decision, Note or Question. It preserves the existing body and accepts the same text inputs as `add` and `edit`: `--body "<text>"` or `--body-file <path>`. A `-` path reads standard input. The positional form `anb comment <id> "<text>"` also works. Use one text input per command.

Each entry records its date, accountable person and optional agent tool. Markdown continuation lines are indented under the entry. An immediate repeat with the same text, date and signature changes nothing. Comments can explain a settled record until it is archived; use `restore` before correcting an archived one.

## Citations

Write a bare record id in a body or comment to reference it. Put the id in backticks when you are discussing its spelling or using it as an example. `anb show` derives incoming and outgoing mentions from the prose. A mention is context; a relation the record should carry as a fact, such as the schema a document follows, is a `link` in the envelope, which the tool walks back from the other end.

A reference to an unknown id produces a hint without rejecting the write:

```sh
anb comment task.grammar-parser-accepts-fences "See task.typo-in-the-id."
```

This allows a forward reference. If it was a typo, correct it; if it was an example, quote it. Until it resolves, the reference also appears as Debt. Envelope relationships such as `--from` are stricter and require an existing target.

## Archive settled knowledge

Archive a superseded or retired Decision, a retired Note, or a closed Question with `anb archive <id>`. `show` still reads an archived record, and `list --archive` still lists it. `anb restore <id>` returns one to the working set without changing its state or contents.

When an idea ends or a ruling stops applying, record why before archiving it. `retire --body` appends the outcome and retires the Note or Decision in one write; `--body-file` reads a longer outcome. No separate report is required:

```sh
anb retire note.domain-language --body "The four names are confirmed in the domain map." --via codex
anb archive note.domain-language
```

An identical retirement leaves the outcome unchanged. `retire` without text remains available when the body already explains why the record ended. For a Question, append any detail with `comment`, then close it with `--resolved-by` or `--reason`.

Use `anb delete <id>` only for a record created by mistake. It removes the file and frees the id, but refuses while another record references it.
