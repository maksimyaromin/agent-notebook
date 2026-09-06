---
id: note.report-anb-v1
type: note
state: retired
title: Report: anb v1
by: Maksim Yaromin
from: task.anb-v1
created: 2026-09-06
updated: 2026-09-06
---

# anb v1

The founding idea shipped. anb keeps a project's working memory as typed records in plain files: Tasks, Decisions, Notes and Questions under `.agent-notebook/`, one markdown file each, with lifecycles the tool enforces, a Status that fits a token budget, a check that names the repair for anything it finds, a skill rendered from the binary, and a setup that wires Claude Code and Codex. It installs from npm with a native binary per platform, or with cargo, and the book at agent-notebook.supolka.dev is the manual.

The epic's twenty-five children closed, each with its report in the archive: the three spikes on the storage format, the record model and the agent interaction, then the Core, the command line, setup and the skills, the distribution, the book, and the release gate last. The repository is public since 2026-09-06, releases are named by their date, and the packages are at 0.1.1.

What stays open: task.import-surface-adopt-an-existing-tracker holds until a first adopter scopes it. No Question is open.
