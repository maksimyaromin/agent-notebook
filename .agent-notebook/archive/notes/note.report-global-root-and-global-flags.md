---
id: note.report-global-root-and-global-flags
type: note
state: retired
title: Report: Global root and --global flags
by: Maksim Yaromin
from: task.global-root-and-flags
created: 2026-08-30
updated: 2026-08-30
---

# Global root and --global flags — what shipped

`--global` names a second notebook root in the user's home: `.agent-notebook` in the home directory, served by the same binary and the same Core through the Storage seam. No new envelope field, no second grammar, no second output contract — a record is global by residence.

## The two halves, and where each lives

**Where the root is** — `fs_storage::notebook_root`. `--notebook` and `--global` name a root two ways and share one precedence rung, so a call using both is refused rather than ranked; the rung outranks `ANB_NOTEBOOK`, which outranks the project default. A home is exported once and outlives every `cd`, exactly like `ANB_NOTEBOOK`, so a non-absolute one is refused: read from the working directory it would make `--global` a different notebook in every directory, and would file the user's private records inside whatever repository they happened to stand in.

**What may be done there** — the new `scope` module. A verb refuses `--global` when it can only create or move a Task or a Question: `add`, `start`, `submit`, `close`, `return`, `reopen`, `hold`, `unhold`, `block`, `unblock`, `comment`, `ask`, `answer`. Everything else — the knowledge writes, every read, the residence moves — behaves the same in either scope. The refusal is a structured `invalid-argument` payload naming the verb, so it reaches an agent as a reply rather than as usage text.

## The rule is derived, not enumerated

The task named six accepting verbs (`decide`, `note`, `retire`, `view`, `list`, `search`) and one invariant (task and question verbs refuse), which together leave `ready`, `check`, `archive`, `expunge`, `edit`, `overview` and `status` unplaced. The shipped rule places them by the invariant rather than by a list, so the accepting set is the six plus those seven.

Three reasons: without `edit` a global Note can never be corrected in place, and knowledge addressed by name is the whole use case; without `check` the one tool that can name a corruption cannot look at the user's notebook; and `ready` answering an empty queue is exactly as truthful as `status` and `overview` printing `0 tasks` in that scope, which they do either way — refusing one while the other two answer zero gives two answers to one question.

`question.should-the-global-scope-accept-more-than` puts this to the owner with the narrower alternative.

## The residual hole, named

`--global` is a scope declaration, not a path check. `anb add "x" --notebook <the home notebook>` does file work in the user's notebook, and nothing can flag it: a notebook carries no mark saying it is the global one, and a path guard would be both incomplete (it cannot see through a symlinked root, and `ANB_NOTEBOOK` offers no flag to inspect) and wrong (it would refuse a legitimate root in a container where `HOME` is the repository). The knowledge-only invariant is enforceable at the flag or nowhere.

No `ANB_GLOBAL_NOTEBOOK`: a user whose notebook is not at `~/.agent-notebook` reaches it with `ln -s ~/dotfiles/notebook ~/.agent-notebook`, which the seam already serves — `unusable_root` refuses a linked *record directory*, never a linked root.

## What the review found

An independent Opus 5 review returned 13 items. The serious ones were mine.

1. **A relative `HOME` was unguarded.** Reproduced: the personal note landed inside the git repository, and the same `--global` from a subdirectory read an empty notebook — both silently. This is the failure `notebook_root`'s own doc comment forbids for `ANB_NOTEBOOK`, and my defence ("consistent with the flag rung") did not transfer: a relative `--notebook` is typed by a caller who knows their cwd, while `HOME` is not typed at all. Now refused.
2. **`chosen_root` was a shallow module.** It computed half a decision and left its single caller to compose the rest, and nothing stopped a later caller from calling `notebook_root` alone and dropping `--global`. Folded in: one interface, one `Result`, the whole precedence in one place.
3. **`refused_globally` took no scope.** Its name promised a condition only its call site evaluated, so a second caller would have refused every task verb in every scope. It now takes `global`. It also moved out of `reply.rs`, whose header promises renderer-shared dispatch decisions: the `chosen_proof` precedent I cited is a *private* helper of `execute`, not a public pre-dispatch guard.
4. **`ready` refused while `status` and `overview` printed zeros.** The output contradicted the rule. `ready` is now accepted and the rule is writes-only.
5. **The classification tests transcribed the implementation.** `KNOWLEDGE`/`WORK` were a line-for-line copy of the match arms, so moving `edit` between them and flipping the table stayed green — no independent oracle. Split into the six the criteria name and the ones the derived rule adds, each carrying its reason.
6. **Four comment claims were false.** Chief among them "every other verb reads or writes records of any type" — `retire` loads only Decisions and Notes.
7. **Untested contract claims.** Added: `--global` outranks `ANB_NOTEBOOK` end to end; the same Note filed in both scopes renders and stores byte-identically; the record actually lands in the home root; a refused `add` leaves nothing in *either* notebook; and the same verb without the flag is still the project's to serve.

Also fixed from the review: message dialect (record types lowercase like every other refusal, one prefix per function, no clause restating what the previous one implies), `--global`'s help now carrying its precedence and its restriction, `#[cfg(unix)]` on the module that poses `HOME`, unused `TempDir` fixtures dropped from tests of a function that touches no filesystem, and the duplicated project fixture hoisted.

## Proof

`./scripts/check.sh` green. 15 new tests, each proved red by mutation before counting. Exercised against the real binary: a Decision recorded globally from one repository read from another, `ready`/`overview`/`status` served in the global scope, `add` refused, both flags refused, a relative `HOME` refused, and `--global` outranking an exported `ANB_NOTEBOOK`.
