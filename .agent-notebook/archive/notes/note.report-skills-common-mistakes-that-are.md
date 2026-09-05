---
id: note.report-skills-common-mistakes-that-are
type: note
state: retired
title: Report: Skills: common mistakes that are real, and an atlas page that stays out of the map's way
by: Maksim Yaromin
from: task.skills-common-mistakes-that-are-real-and
created: 2026-09-05
updated: 2026-09-05
---

# Skills: mistakes that are real, and a page that stays out of the map's way (2026-09-05)

Report for task.skills-common-mistakes-that-are-real-and, filed on two rulings the owner made after reading the public skills and the page a fresh session drew by the atlas skill.

## The `anb` skill

The first Common mistakes table was symmetry: it repeated the sections above it as prohibitions (do not edit files, do not guess after a refusal). It is replaced by eight mistakes that are easy because of how the tool behaves, each with the cost the next session pays and the move that avoids it:

| The mistake | Why the tool makes it easy |
|---|---|
| a child Task added without `--from` and without a `block` edge | scope follows edges, so the epic's queue, listing and counter never see the child |
| a backticked id where a reference was meant, or a bare id as an example | the mention scan reads backticks as quotation and bare ids as references, so a typo hides or an example nags |
| a Question answered in a comment on its Task | nothing closes a Question but `close`, so it ages into `origin-closed` Debt with its answer buried in a log |
| a ruling changed with `edit --body` | `edit` is legal on a Decision, and nothing but `--supersedes` tells the notebook a rule changed |
| a collapsed Status section read as empty | under budget a section keeps its count and drops its rows |
| a held Task read as gone | a hold removes a Task from `active:` and `ready` and puts it in `held` only |
| `--no-proof` with a proof somewhere else | it is the shortest `try:` line offered, and it writes "nothing to show" for good |
| a record recreated because `list` did not show it | the working set hides the archive, and ids are never freed, so `duplicate-id` is the surprise |

## The atlas skill

The page a fresh session drew by the skill was faithful to its rules and wrong all the same: a header of controls, a rail of counters, a footer of provenance, a disclaimer, a panel opening on twenty envelope rows, and a ranked layout of the whole notebook drawn as a rope. The reference page had one narrow column and a map. The skill's page section now prescribes that restraint (one column: find, legends, arrangement; one line under the map: the command and the slice; nothing else), says that the page opens with names readable, that web is for a notebook and ranked for a branch, and that the panel opens on the body and the relations with the envelope folded last. Its Common mistakes are the ones that page showed, plus two from the encoding (two facts on one channel; the bounded plain text). `docs/guides/atlas.md` follows the skill.

## Verified

The gate is green, the skill drift check included, and the docs check passes.

## Smoke check

Sonnet 5, once, reproducing every row of the `anb` table on a scratch notebook: all eight held, and the check judged none banal. One should-fix taken: the last row led with the `duplicate-id` refusal, which fires only when `--id` names the taken id; the common path, `add` with the same title, mints `<id>-00` beside the archived record without a word, and the row now says so. Two nits taken: the collapsed-section row used a bracket notation the tool does not print for a collapsed section, and the atlas sentence about legends now says which facts get one and that degree, read from size, needs none. The atlas rules were held to the reference page's principles and to the ruling; every rule follows, and the one deviation (the panel opening on the body rather than the envelope) is the ruling itself. The gate and the docs check passed under the check as well.
