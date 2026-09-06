---
id: note.report-setup-appends-its-snippet-as-its
type: note
state: retired
title: Report: setup appends its snippet as its own paragraph
by: Maksim Yaromin
from: task.setup-snippet-paragraph
created: 2026-09-06
updated: 2026-09-06
---

# setup appends its snippet as its own paragraph

`anb setup` appended its marker-bounded line right after the last line of `AGENTS.md` or `CLAUDE.md`, ensuring only a newline before it. CommonMark reads a non-blank line after a list item or a paragraph as a lazy continuation, so a guide ending with a bullet rendered the snippet as the tail of that bullet, and the user's last rule read as if it ended with "Project working memory: …".

## What changed

- An appended snippet stands behind a blank line: `snippet_applied` pads the text until it ends with a blank line before it appends, and a guide that already ends on one gets no second. A snippet patched in place between existing markers is untouched.
- `setup --remove` takes one blank line ahead of the snippet back out when a blank line or the end of the file follows it. A guide that ended on one newline, the shape every editor leaves, comes back as it was; a snippet the writer moved between two paragraphs leaves one paragraph break behind, never two; a snippet followed directly by text keeps the blank line ahead of it, since that line is the break between the user's own paragraphs. A guide that ended on blank lines of its own comes back one shorter, and one without a final newline gains one: removal cannot tell the break setup added from one the guide had, and the contract says so.
- The agents guide names the appended block as a paragraph of its own.

## Evidence

- Three unit tests on `snippet_applied` and `snippet_removed` went red before the fix: the appended snippet after a list item, the guide already ending blank, and the round trip. The existing end-to-end test that asserts `setup --remove` returns `AGENTS.md` byte for byte holds through the new shape.
- `scripts/check.sh` green: format, clippy, tests, doctests, rustdoc, the rendered skill and the reference pages.
- Smoke check by another model, once: four findings. Must-fix, taken: the removal claimed a byte-exact round trip for every guide and delivered it only for a guide ending on one newline; a stateless removal cannot tell the break setup added from a blank line the guide already ended on, so the contract now says what holds and a test covers each case: a guide ending on one newline comes back as it was, one ending on blank lines comes back one blank line shorter, one without a final newline gains one. Should-fix, taken: each new test specifies one situation. Should-fix, declined: the blank line is a bare LF in a CRLF guide; every line the tool writes is LF, as the grammar leaves a CRLF file when it splices, and the snippet's own line already was. Nit, taken: an assertion message overstated what the mid-file case keeps.

Pull request: https://github.com/maksimyaromin/agent-notebook/pull/49, squash-merged on a green CI check.
