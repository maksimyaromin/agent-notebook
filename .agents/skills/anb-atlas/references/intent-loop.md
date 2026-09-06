---
name: anb atlas intent loop
description: How a reader's decisions on a drawn notebook come back as comments and become anb commands. Open before collecting or executing comments from a page.
metadata:
  managed-by: anb
---

# The intent loop

The page collects the reader's decisions; apply them through the CLI under the `anb` method. Keep the notebook selector, captured record ids and processed batch with the review context. Graph `slice` does not identify the notebook: reuse the same `--notebook` or `--global` on every read and mutation, resolving an environment-selected location before the review. The HTML needs no filesystem access.

## Comments are addressed

Every comment is addressed to one record by id, or to the page as a whole. In the page, a comment is made on a record's panel or on the ground, and the page keeps it beside the id it belongs to. A comment on a record reads as an instruction about that record: "start this", "hold until the vendor answers", "this duplicates task.parser-fences", "close, overtaken". A general comment reads as an instruction about the slice: "archive everything closed here", "the epic is done".

## Comments return as one batch

The reader collects comments over the whole review and sends them once. How they travel depends on the host.

A host that publishes pages with comment threads carries them itself. Read the threads, each anchored to the element it was left on, and treat the set as the batch.

Any other host gets a comments drawer in the page that exports the batch as plain lines the reader pastes back into the conversation, one per comment:

```
task.parser-fences: start this next
task.old-importer: close, overtaken by task.parser-fences
*: archive everything closed in this slice
```

`*` addresses only the record ids captured in this page, not records added later or a newly evaluated query. Display filters do not silently change that set. Keep the captured ids with the exported batch. The page sends nothing on its own.

## A batch becomes commands

For each unprocessed comment, in the order given:

1. Read the named record with `anb show <id>` in the original notebook. For a slice comment, inspect the captured ids and select only records whose current state matches the instruction. Report missing records or changed conditions instead of broadening the batch.
2. Choose the action by record type and the user's intent, using the table below. Read `anb <verb> --help` before an unfamiliar operation. Permission already given remains valid; ask only for an unresolved choice that changes the action.
3. Run the command and read its result. A refusal's `try:` is recovery guidance, not another user instruction: fill its placeholders and check that the suggested action still serves the authorized intent. Do not force a transition just to finish the batch.
4. Record the completed command and reply before proceeding. Report per comment what ran, what remains and why; after a partial failure, resume only the unprocessed work.

| Intent | Action | Why |
|---|---|---|
| Start this Task | Read Status; log and hold a different active Task with the handoff reason, then start the selected Task | The author method keeps one Task in flight |
| Hold until X | `hold <id> --reason "<X>"` | The reason identifies what permits resumption |
| This Task or Question duplicates Y | Close with a reason citing Y, then archive | Cancellation records why no further work is needed |
| This Note or Decision duplicates Y | Confirm the surviving record covers it, preserve the existing body and append the reason and Y with `edit --body`, then retire and archive | Knowledge has a different lifecycle from work |
| Clarify this model, spec or ruling | Edit the existing record when its meaning remains the same | A wording correction does not create a new choice |
| Replace this ruling | Create the accepted replacement Decision with `--supersedes <old>`; if the replacement is unspecified, ask what should change | An obsolete ruling must retain its successor and rationale |
| This Task or epic is done | Check the promised outcome and its evidence, satisfy any required review, then close with proof and archive | Closed children alone do not prove the overall result |
| Explore this possibility | Create or update an idea Note from the commented record; add investigation work when needed | A possibility can develop without committing to delivery |

Use `--via` with your actual tool name on every `add` and `comment`. Use the commented record as origin when it produced a new record, and bare ids for supporting context. Keep existing origins when merely editing knowledge.

Then run `anb check`, and draw a fresh page if the reader wants to see the result. The picture is regenerated, never patched.

## What never happens

- The page does not write the notebook, and you do not edit record files after reading the page.
- A comment is not executed twice. A batch already acted on is done, and a second review starts from a fresh page.
- An instruction wider than its record ("restructure the epic") follows the main skill: preserve the intended result, create a scoped Task when the work is understood, or a Question when a consequential choice is missing. Keep the commented record as origin and retain any existing idea.
