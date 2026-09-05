---
id: note.report-shadow-surfacing-in-status-debt
type: note
state: retired
title: Report: Shadow surfacing in Status/Debt
by: Maksim Yaromin
from: task.global-shadow-surfacing
created: 2026-08-30
updated: 2026-08-30
---

# Shadow surfacing in Status/Debt — what shipped

A project's Status now reads the user's notebook behind its own and names each cross-scope pair: a live project Decision whose prose cites a standing Decision of the user's, both ids and both authors, nothing resolved.

```
debt[1]:
  shadow: decision.spaces (Teammate) <-> global decision.tabs (Reader)
```

## The discriminator, and why

A project Decision cannot supersede one in the user's home — a repository does not mutate it — and an envelope reference to a record this notebook does not hold is a dangling reference that `check` refuses. So a citation in prose is the only edge that survives, and a typed id in prose is deliberate: the same construction the undeclared-pair signal already rests on. The shared-tag heuristic used for the write-time nudge was rejected: across scopes it would fire on coincidence and nag every session.

The criteria's "names the pair — both ids, both authors" also rules out same-id shadowing, which would name one id.

## Two rules that keep it honest without comparing roots

**Only what this notebook has no live answer for reaches across.** A citation this notebook answers with a live record of its own is about that record and belongs to the undeclared pair. That single rule buys three things: a notebook read behind itself pairs with nobody (so no root equality has to be guessed anywhere), a rule this project *retired* stops hiding the user's — which still stands — and a project id colliding with a global one resolves the way shadowing means.

**A citation names nothing only when neither notebook holds it.** The whole second notebook is read, through the same `Resolver` as the first, so an id it carries stops being reported as a dangling mention. Naming it missing would be false: a reader standing in the project can open it.

## Where the second root is read

The Core takes it as `Option<&dyn Storage>` on `status` — the one reader. `Notebook::new` and its 282 call sites are untouched, and the shared reference makes "read and never written" a fact the type system keeps rather than a promise a comment makes: `Storage::write` needs `&mut self`.

A second root is read for a hint on somebody else's dashboard, so a root that cannot be read leaves the hint out instead of taking that dashboard down. It is held to the same root rules as the first — `unusable_root` guards both — and its own health is what a `check` against it reports.

## What the review found

An independent review returned 13 items; seven were reproduced defects.

1. **A broken user notebook hard-failed every project `status`.** A file where the directory should be, or an unreadable one, exited 1 with a message pointing at the user's home. The criterion "absence of a global root is not an error" was honoured for literal absence only. Now the hint is dropped instead, and `unusable_root` guards the second root as it does the first.
2. **A shadow was silently lost when the project's own archive held the same id.** The guard asked `resolves`, which counts archived ids, so a rule this project retired hid the user's — and neither the pair nor the dangling signal picked it up. The criterion says *never silent*. The guard now asks for a live record of this notebook's own.
3. **`dangling-mention` made a false claim, asymmetrically.** Suppression covered only the user's live Decisions, so citing its Note, its Task, or a retired rule still said the id names nothing. Reading the whole second notebook fixed the behaviour and made the comment true.
4. **An invalid record of the user's notebook was named as half a pair** — under an id its own envelope disowns. The undeclared pair explicitly refuses this; the shadow now does too.
5. **`over global` asserted an override the trigger never establishes.** A citation can be agreement; the rendering printed a verdict as fact, in the one word a reader acts on. It now reads `<-> global`, parallel with the undeclared pair, and the precedence rule lives once in the variant's doc.
6. **A doc claimed the order the project records name them while the rank re-sorted alphabetically.** The rank now preserves that order, so the sentence is true.
7. **The tests transcribed too little to catch their own subject.** The end-to-end assertion used four `contains` checks and passed with the two sides swapped — which side shadows which is the whole criterion. The retired case asserted only the absence of one code and stayed green over defect 3. The JSON surface had no coverage at all despite the claim it was exercised.

Also fixed from the review: `Notebook` lost the `user` field and the `beside_the_user` constructor — a field one method read, on the handle that also runs `expunge` and `close`, under a name that promised "read beside"; `records_in` was a pure delegate that failed the deletion test and is gone; five comment claims were false or over-general; the tests moved into the `debt_signals` module that already owns the Debt contract and reuse its fixtures instead of duplicating them; and `process.rs`, `concurrency.rs` and `fs.rs`'s environment helper now pin `HOME`, which `status` reads on every run — a hermeticity regression this change introduced.

**Kept as shipped, against the review:** a shadow stays a `DebtSignal`. The reviewer's point is fair — a shadow is not decay, it is permanent and correct, and Debt is the section a reader is trained to clear — but the criteria name "Status/Debt", and moving it would invent a section the phase did not ask for. Worth the maintainer's attention.

## The residual hole, named

Two write-side surfaces still read this notebook alone, and one of them is a gate:

- `decide`/`comment`/`edit` probe a body's citations against the project, so recording the very rule that shadows a global one answers `dangling: decision.x` while Status answers `shadow: … <-> global decision.x`.
- `check` verifies a `link:` target that parses as an id, so declaring the edge — the natural reaction to a shadow row — makes `check` **exit failure** on an id that exists.

`question.should-the-write-paths-and-check-see-the-users-notebook` puts both to the maintainer with three options. Shipping the nudge inconsistency alone would be defensible; shipping it beside a red `check` is the part that deserves a decision.

## Proof

`./scripts/check.sh` green. 13 new tests, each proved red by mutation before counting — including one that poses a second root whose every read fails. Exercised against the real binary: the pair reached Status and the JSON debt rows, retiring the global rule removed both the shadow and the dangling line, and a user notebook replaced by a file left the project's dashboard standing.
