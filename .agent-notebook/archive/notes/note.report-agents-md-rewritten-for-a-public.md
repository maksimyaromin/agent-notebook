---
id: note.report-agents-md-rewritten-for-a-public
type: note
state: retired
title: Report: AGENTS.md rewritten for a public repository, free of local dependencies
by: Maksim Yaromin
from: task.agents-md-rewritten-for-a-public
created: 2026-09-05
updated: 2026-09-05
---

# AGENTS.md for a public repository (2026-09-05)

Report for task.agents-md-rewritten-for-a-public. Before the repository goes public, the instruction file a stranger's agent reads had to say only what an agent needs to work here from a clean clone, and nothing that depends on a path outside the repository.

## What shipped

`AGENTS.md` is rewritten in the shape of the sibling project's public file, about 850 words: what the project is and who the file is for; how a change is verified (`./scripts/check.sh`, `pnpm install && pnpm docs:check`, the two renderings and how to regenerate them); the notebook protocol, through the shipped skill rather than a diary of rulings (open from Status, one Task in flight, log, close with a proof and archive, Questions closed, holds reasoned, `check` green, friction filed as a finding); the layout as a table; the conventions as a table naming the page that owns each rule (behaviour tests, comments that state facts, the text law, self-contained committed text, English, conventional commits without agent attribution, main by pull request only, unwrapped markdown, the sidebar rule, the nearest-page rule); and a reading order into the book and the notebook itself. `CLAUDE.md` remains the link to it.

## Acceptance

No path outside the repository is named in `AGENTS.md`. Every command in the file was run from a fresh clone with the new file in place. The file reads as documentation: it tells the reader what is true and where each rule is owned, and it dates nothing.

## Review

One review pass, from two fresh clones with the new file in place: every command ran (the gate, the docs check, the notebook reads, both regenerations without a diff, `cargo install`), every layout row and reading-order claim held, no maintainer-private content remained. Two must-fix, both taken: the comment rule and the text rule were attributed to pages that did not state them; the development page now states both (a comment is a fact a stranger can check; the words, sentence-case headings, no dash connectives) and the table points there. One should-fix taken: "one Task is in flight at a time" read as a tool invariant and is the skill's practice; the sentence now says so. Two nits taken: the development page now names both docs-check failure modes, and the old "never `/tmp`" rule, which had fallen through both files, is in the maintainer's local file.
