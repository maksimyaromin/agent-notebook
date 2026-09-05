---
id: note.report-a-task-ends-without-work-stating
type: note
state: retired
title: Report: A Task ends without work, stating why
by: Maksim Yaromin
from: task.close-drop-ends-a-task-without-work
created: 2026-09-05
updated: 2026-09-05
---

# A Task ends without work, stating why (2026-09-05)

Report for task.close-drop-ends-a-task-without-work. Shipped inside the vocabulary audit's change, because the word the flag carries was the audit's to settle: `drop` and `withdraw` were both refused by the maintainer, and the audit's Decision (decision.the-cli-speaks-one-plain-word-per) chose `--reason`, the one word for "why" that `hold` already spoke.

## What shipped

`anb close <id> --reason "<why>"` ends a Task from `open`, `active` or `review` without work. The move writes `state: closed`, stamps `closed: <date>`, writes the reason into the envelope as `reason: <why>`, and writes no proof link: a proof would vouch for work that did not happen. `--no-proof` keeps its meaning — done with nothing to show — and keeps refusing an open Task. The state is the same `closed`, so dependents unblock, epics count the Task and `archive` and `reopen` need no new rule; the envelope carries the distinction, where `check` can read it.

The same flag ends a Question without a record, which is where the original `answer --drop` went.

## Where the reason lives, and why not the log

The task body asked for the reason in the log. It landed as an envelope field instead: the body is opaque to the parser by design, so a log line is a fact `check` cannot verify, while a field is one it does — a `reason` on a record that is not closed is a `bad-value` finding, and a closed Question with neither `resolved-by` nor `reason` is a `missing-field`. The field also mirrors `hold`: the reason a Task pauses and the reason it ends are read the same way, from the envelope.

## Shape

- `TaskAction::CloseWithReason` sits beside `Close` in the state machine with the wider source set (`open` included); its word is `close --reason`, which is also the key of its retry shape, so an invalid move from `open` answers `valid: start, close --reason` and prints `try: anb close <id> --reason "<why>"` — a line that runs.
- `Notebook::close_with_reason` dispatches on the id's type: a Task goes through the shared task transition, a Question through the shared Question close; both then build the one `Closed` reply every way of closing shares (the still-open Questions born from the record, the Tasks it was the last blocker of, the citations in the reason a reader is nudged about).
- The CLI folds `--reason` into the one-of-seven exclusivity the proofs already had; two flags are refused naming the whole set.
- The reason is guarded once, in `write::guarded_reason`: trimmed, one line, never empty.

## Tests

Core, through `Notebook`: an open Task closes with the reason in the envelope and no link, and unblocks its dependents; the move is legal from every live state; a waived close still refuses an open Task; a replay changes no byte; an empty or multi-line reason is refused before any byte moves; the same promises on a Question. CLI: the reply line, the envelope written, the conflict with a proof, the missing-flag refusal with the three runnable shapes, the valid-move listing from `open` and from `review`. Every new test was made to fail before it counted.

## Left open

question.can-a-task-die-without-ever-being was resolved into this Task before the marathon; nothing else.
