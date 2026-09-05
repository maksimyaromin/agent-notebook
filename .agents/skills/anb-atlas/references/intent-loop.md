---
name: anb atlas intent loop
description: How a reader's decisions on a drawn notebook come back as comments and become anb commands. Open before collecting or executing comments from a page.
metadata:
  managed-by: anb
---

# The intent loop

The page is where the reader decides, and the CLI is where the notebook changes. Nothing in between writes a file.

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

`*` addresses the slice. The page sends nothing on its own.

## A batch becomes commands

For each comment, in the order given:

1. Read the record it names with `anb show <id>`, so the instruction is judged against the record as it is now, and not as the page drew it.
2. Choose the command that carries the intent and no more. "Start this" is `anb start <id>`. "Hold until X" is `anb hold <id> --reason "<X>"`. "Duplicate of Y" is `anb close <id> --reason "duplicate of Y"` followed by `anb archive <id>`. "This decision is wrong" is a question back to the reader, since a Decision is replaced by a new one and only the reader can say what it should say.
3. Run it. A refusal's `try:` line is the next command. A refusal that means the intent no longer applies is reported, never forced.
4. Report per comment: the comment, the command run, and the reply's first line. A comment you could not turn into a command is reported as such, with the question it raises.

Then run `anb check`, and draw a fresh page if the reader wants to see the result. The picture is regenerated, never patched.

## What never happens

- The page does not write the notebook, and you do not edit record files after reading the page.
- A comment is not executed twice. A batch already acted on is done, and a second review starts from a fresh page.
- An instruction wider than its record ("restructure the epic") becomes a Task or a Question in the notebook, born from the record it was left on, rather than a chain of guesses.
