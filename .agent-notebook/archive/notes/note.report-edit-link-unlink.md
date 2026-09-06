---
id: note.report-edit-link-unlink
type: note
state: retired
title: Report: edit declares and erases a link
by: Maksim Yaromin
from: task.edit-link-unlink
created: 2026-09-06
updated: 2026-09-06
---


# edit declares and erases a link

A live Decision citing another live Decision was permanent `may-conflict` Debt unless one envelope declared an edge to the other. The Debt heuristic already honoured a `link` naming the id, but a link could only be written at `add`, so a Decision could not declare a relationship after the fact, and the citations the method itself asks for, a drift naming the rule it departs from or a rule that is part of a wider one, were resolved by backticking the ids and losing the graph edge.

## What changed

- `edit --link "<kind> <target>"` and `edit --unlink "<kind> <target>"`, repeatable, on any live record. Links splice at set granularity like tags, matched as the grammar reads a line rather than byte for byte: a link already present, or one already absent, changes no byte and the reply answers `already`.
- A link whose target is shaped like a record id must name a record, here or in the user's notebook, which a link reaches as `check` reads it; `add --link` and `edit --link` refuse a dangling one as `dangling-ref` before any byte moves, as `--from` does. A target of any other shape points outside the notebook and is taken as given.
- The skill's execution section teaches the sentence: a Decision that cites another as context declares the relationship once, `within` for a rule that is part of a wider one, `departs-from` for a drift, on `add` or later with `edit`, and `may-conflict` then names only the pair nobody has judged. The knowledge guide shows the edit and the pair leaving the signal; the records reference and the tasks guide name the flags.
- The optional default link for `add decision --kind drift` stays out: a rule the kind implies is better stated by the author than minted by the tool, and the skill's own table already asks for it.

## Evidence

- Core tests: links are added and removed by their line and `changed` names `link`; a link already present or already absent changes no byte. CLI tests: an undeclared citation between live Decisions is Debt, one `edit --link` takes the pair out of `may-conflict`; a link naming no record is refused on `edit` and on `add` with nothing written; a link without a target is a recovery payload. Each was shown red once.
- On this notebook, `edit --link` attached issue 46 to the held import Task, the friction noted when this set of Tasks was opened.
- `scripts/check.sh` green, the regenerated skill and reference pages included; `pnpm docs:check` green.
- Smoke check: no functional regression. Should-fix, taken: a link was matched byte for byte against its line, so a hand-written line with two spaces between its halves was never recognised; links are now matched as the grammar reads them. Should-fix, taken: the cited record declaring the edge was covered by the heuristic but by no test; a CLI test and the knowledge guide's example now exercise that side. Should-fix, taken: the skill sentence carried three clauses before its semicolon and is now two sentences with the mechanism after a colon. Nits taken: a doc comment that restated the grammar deleted, the trimming named on the line renderer, three unused derives dropped, the guide's example made continuous with the paragraph before it. Noted for a later issue, not this change: a record carrying a hand-written dangling link is closed to nothing, while a dangling origin closes it to unrelated edits, though check reports both as errors.

Pull request: https://github.com/maksimyaromin/agent-notebook/pull/53, squash-merged on a green CI check.
