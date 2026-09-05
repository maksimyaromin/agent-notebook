---
title: Decisions, Notes and Questions
description: 'Rulings that stand until replaced, knowledge kept current in place, doubts that close by naming what settled them, and how citations are checked.'
---

Three record types hold what a project knows rather than what it does.

## Decisions

A Decision is a ruling. It carries a kind: `rule` for how we work, `shape` for a design ruling, `drift` for a deviation that stands until superseded.

```
$ anb add decision "Fences never nest" --kind rule --tag parser --tag grammar --body "A fence closes at the first closing marker."
ok: add decision.fences-never-nest — decisions/decision.fences-never-nest.md
```

Rules print in every Status, so the project's standing rules apply without anyone reading the log. A Decision is never edited into a different ruling; it is replaced: `anb add decision "<title>" --supersedes <old>` writes the new one and flips the old to `superseded` in the same move, and `anb retire <id>` ends one that has no successor. A dead rule cannot be read as live.

When a new Decision shares ground with a live one, the reply says so:

```
$ anb add decision "A fence body is opaque" --kind rule --tag parser --tag grammar
ok: add decision.a-fence-body-is-opaque — decisions/decision.a-fence-body-is-opaque.md
may-conflict[1]: decision.fences-never-nest (Alex)
```

The nudge fires at write time when the new Decision shares two or more tags with a live one, or cites it in its body, and it stops there: the writing agent, holding the full context, is the cheapest judge that will ever see the pair. Read the named Decision before going on, and supersede it if yours replaces it. Status keeps a `may-conflict` Debt line only for a pair where one cites the other and neither supersedes, since a shared tag is a hint and a citation is a claim.

## Notes

A Note is curated knowledge, kinds `fact`, `term` and `guide`. Unlike a Decision it is corrected in place with `anb edit`, because a fact that changed is the same fact, updated. A `term` defines a word of the project's domain language, so an agent names things the way the project does. A report ingested by `anb close --note` is a Note too, born from the Task it closes.

## Questions

A Question is a doubt parked without widening the Task that met it:

```
$ anb add question "Do fences nest?" --from task.parser-accepts-fenced-bodies
ok: add question.do-fences-nest — questions/question.do-fences-nest.md
```

It closes by naming what settled it, `anb close <question> --resolved-by <decision-or-task>`, or by saying why it no longer applies, `anb close <question> --reason "<why>"`. A Question cannot close any other way, so it cannot rot open unnoticed: an open Question older than its clock surfaces as Debt, and a Question whose origin Task closed surfaces at once.

## Citations

In a body or a comment, a bare id is a reference and a backticked one is a quotation. The tool scans prose for id-shaped tokens on demand and derives read-only value from them: `anb show` lists what a record mentions and what mentions it, and a citation of an id that exists nowhere comes back at write time:

```
$ anb comment task.grammar-parser-accepts-fences "see task.typo-in-the-id"
ok: comment task.grammar-parser-accepts-fences — logged
dangling-mention[1]: task.typo-in-the-id — backtick to quote, or create the record
```

A forward reference is legal, since the record may come; a typo is not. A dangling mention that stays becomes a Debt line in Status until the record exists or the id is quoted.

## Archiving knowledge

A superseded or retired Decision, a retired Note, a closed Question: each is archived with `anb archive <id>`, keeps its id and bytes under `archive/`, and comes back with `anb restore <id>` if it was filed too early. Nothing is deleted by a lifecycle move. `anb delete <id>` exists for a record born by mistake, and refuses while anything cites it.
