---
id: note.report-readme-the-page-a-stranger-reads
type: note
state: retired
title: Report: README: the page a stranger reads first
by: Maksim Yaromin
from: task.readme-the-page-a-stranger-reads-first
created: 2026-09-05
updated: 2026-09-05
---

# README: the page a stranger reads first (2026-09-05)

Report for task.readme-the-page-a-stranger-reads-first.

## What shipped

`README.md` at the repository root, in the shape of the sibling project's page: a centred title and tagline, the documentation link, the CI and license badges, then prose that shows the tool working and reads each reply back to the stranger. Every command block is the literal output of the binary at HEAD, captured from one run in a scratch project with the git identity `Alex`: a Task added, started and logged; a Question filed from it and closed into a Decision; Status; the close with a report ingested as a Note and the archive that carried it; the clean check; one refusal with its `try:` line; a hub with a child born from it, the block edge, the queue, a hold and the quiet Status. The record file is shown as the tool wrote it. Then the setup table (what each written file does), where the generated skill comes from, the user's own notebook, and development.

Install is shown two ways, `npx -y @supolka/agent-notebook` and `cargo install --git`. The npm package publishes at release; the README describes the released state.

Two files came with it. `LICENSE` is MIT with the owner's name, copied from codemode-executor: the repository had none, and a public repository needs one. The owner confirms the choice at the final review. The documentation link points at `agent-notebook.supolka.dev`, following the sibling's `codemode-executor.supolka.dev`; the owner sets the DNS record, or the docs task adjusts the address.

The prose follows the writing rule the skills follow: no dashes as connectives, plain sentences, sentence-case headings. The em dash inside the tool's own output stays, since the blocks are literal.

## Findings met on the way

The quiet Status line prints "1 tasks, 1 decisions", a plural on one, and it is the first reply a stranger reads closely. Filed as task.status-counts-read-as-english-one-task, born from this task. A log entry written without a git identity renders its author as `-`; the README runs with an identity, so it does not show, and it is not filed: an author-less entry has nothing else honest to print.

## Smoke check

Sonnet 5, once, running every command of the README in a fresh project and holding every claim to the code. One must-fix: the close step read `report.md`, and nothing in the walkthrough had written it, so a reader following the page could not close the Task. The page now writes the report first. One should-fix: "ids are never reused" was stated as a blanket rule, and `anb delete` frees an id; the sentence now scopes the promise to lifecycle moves and names `delete` for what it is. One nit: the gate's rustdoc build was missing from the development paragraph. Every other block reproduced byte for byte, and every claim held.
