---
id: note.report-cli-help-and-reply-texts-pass-the
type: note
state: retired
title: Report: CLI help and reply texts read as plain prose, so the commands reference reads well
by: Maksim Yaromin
from: task.cli-help-and-reply-texts-pass-the
created: 2026-09-05
updated: 2026-09-05
---

# Help strings read well (2026-09-05)

Report for task.cli-help-and-reply-texts-pass-the, filed when the maintainer ruled that the skills must read pleasantly for a human: the commands reference is rendered from the clap help strings, so the rule reaches them.

## What changed

The rendered commands reference carried four em dashes, all from five help strings that used a dash as a parenthetical or a colon: the `--global` flag, the `add`, `comment` and `search` verbs, and the `--report` proof. Refusal and finding messages carried nine more of the same shape, one sentence each with a dash as the connective ("nothing to change — pass --title", "is archived — add --archive"), and the priority range was written `0–4`. Each is now a plain sentence with the same content: a comma, a colon or a semicolon where the dash stood, the clauses reordered where that read better ("Find records by substring, the archive included"), and the range spelled out ("priority: 9 is outside 0 to 4"). The skill was regenerated and its committed copy matches.

What stays: the em dash in the reply format, `ok: <verb> <id> — <what changed>`, and in the `dangling-mention` nudge. There it is structure an agent parses, the settled shape of every reply, and the plain-prose rule is about prose. Code comments were left alone too; they are the engineer's text, not the skill's.

## Tests

No behaviour changed. The gate is green, the drift check included, and no snapshot moved: the help text is not snapshotted, which is right, since its wording is the kind of thing that should be free to improve.

## Review

One review pass: the six strings first rewritten held their meaning and rendered cleanly in `--help` and live; the check then found eight more refusal and finding messages in the Core with the same prose dash and the en dash in the priority range, all taken in the same pass. Left as they were, on the check's own reading: the struct-level doc comment on the add arguments, which never renders in help, and the two test fixtures that hold a dash on purpose.
